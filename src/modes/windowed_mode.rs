use softbuffer::{Context, Surface};

use crate::data::log::{alert, error, info};
use qoi::{Decoder, Header};

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{self, ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Icon, Theme, Window, WindowId, WindowLevel},
};

#[cfg(target_os = "linux")]
use winit::platform::{
    modifier_supplement::KeyEventExtModifierSupplement, wayland::WindowAttributesExtWayland,
};

#[cfg(target_os = "windows")]
use winit::platform::{
    modifier_supplement::KeyEventExtModifierSupplement, windows::WindowAttributesExtWindows,
};

use std::{
    num::NonZeroU32,
    sync::mpsc::{self, SyncSender},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::graphics::Pixel;
use crate::{audio::get_no_sample, data::*};

type WindowSurface = Surface<&'static Window, &'static Window>;

pub struct WindowProps {
    pub width: u16,
    pub height: u16,
}

impl WindowProps {
    pub fn set_size<T: TryInto<u16>>(&mut self, s: (T, T)) {
        self.width = s.0.try_into().ok().unwrap();
        self.height = s.1.try_into().ok().unwrap();
    }

    pub fn getu(&self) -> (usize, usize) {
        (self.width as usize, self.height as usize)
    }
}

struct WindowState {
    pub prog: Program,
    pub window: Option<&'static Window>,
    pub surface: Option<WindowSurface>,
    pub exit_sender: Option<SyncSender<()>>,
    pub thread_control_draw_id: Option<JoinHandle<()>>,
    pub final_buffer_size: PhysicalSize<u32>,
}

impl ApplicationHandler for WindowState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.prog.print_startup_info();

        let scale = self.prog.scale() as u32;
        let win_size = PhysicalSize::<u32>::new(
            self.prog.pix.width() as u32 * scale,
            self.prog.pix.height() as u32 * scale,
        );

        let de = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();

        let icon = read_icon().and_then(|(w, h, v)| {
            Icon::from_rgba(v, w, h)
                .inspect_err(|_| error!("Failed to create window icon."))
                .ok()
        });

        let window_attributes = Window::default_attributes()
            .with_title("cvis")
            .with_inner_size(win_size)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_transparent(false)
            .with_resizable(self.prog.is_resizable())
            .with_theme(Some(Theme::Dark))
            .with_window_icon(icon);

        #[cfg(target_os = "linux")]
        let window_attributes = window_attributes.with_name("coffeevis", "cvis");

        #[cfg(target_os = "windows")]
        let window_attributes = window_attributes.with_class_name("coffeevis");

        // Since we are leaking the window into a static
        // reference, resumed() is not allowed to be
        // called again as it would cause the build up
        // of leaked windows and potentially flood RAM.
        assert!(self.window.is_none());

        self.window = Some(Box::leak(Box::new(
            event_loop.create_window(window_attributes).unwrap(),
        )));

        let window = self
            .window
            .expect("Window unwraps to none. This error should never happen!");

        let size = window.inner_size();
        self.final_buffer_size = size;

        self.surface = {
            let context = Context::new(window).unwrap();
            let mut surface = Surface::new(&context, window).unwrap();

            Self::resize_surface(&mut surface, size.width, size.height);

            Some(surface)
        };

        if !de.is_empty() {
            info!("Running in the {} desktop (XDG_CURRENT_DESKTOP).", de);
        }

        // On XFCE this is needed to lock the size of the window.
        if !self.prog.is_resizable() {
            window.set_min_inner_size(Some(win_size));
            window.set_max_inner_size(Some(win_size));
        }

        let (exit_send, exit_recv) = mpsc::sync_channel(1);

        self.exit_sender = Some(exit_send);

        let mut intervals = self.prog.get_rr_intervals();
        let mut milli_hz = self.prog.get_milli_hz();
        let rr_mode = self.prog.get_rr_mode();

        // Thread to control requesting redraws.
        self.thread_control_draw_id = thread::Builder::new()
            .name("coffeevis draw control".into())
            .spawn(move || {
                window.request_redraw();

                if rr_mode == RefreshRateMode::Sync {
                    // Wait for a little before querying for monitor refresh rate.
                    thread::sleep(Duration::from_millis(100));
                    Self::check_refresh_rate(window, &mut milli_hz);
                    intervals = Program::construct_intervals(milli_hz);
                }

                loop {
                    let s = get_no_sample();

                    let itvl = intervals[(s > SLOW_DOWN_THRESHOLD) as usize];

                    if exit_recv.recv_timeout(itvl).is_ok() {
                        break;
                    }

                    if !window.is_minimized().unwrap_or(false) && s < STOP_RENDERING {
                        window.request_redraw();
                    }
                }
            })
            .ok();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.call_exit(event_loop),

            WindowEvent::MouseInput { button, .. } => {
                if button == event::MouseButton::Left {
                    if let Some(err) = self.window.as_ref().and_then(|f| f.drag_window().err()) {
                        error!("Error dragging window: {}", err);
                    }
                }
            }

            WindowEvent::Focused(_) => {
                if let Some(w) = self.window.as_ref() {
                    w.request_redraw()
                }
            }

            WindowEvent::Occluded(b) => {
                self.prog.set_hidden(b);
            }

            WindowEvent::Resized(PhysicalSize { width, height }) => {
                let Some(surface) = self.surface.as_mut() else {
                    error!("Coffeevis is unable to resize the buffer!");
                    return;
                };

                let scale = self.prog.scale() as u16;

                let w = u16::min(MAX_WIDTH, width as u16);
                let h = u16::min(MAX_HEIGHT, height as u16);

                if w == MAX_WIDTH || h == MAX_HEIGHT {
                    alert!("You are hitting the resolution limit of Coffeevis!");
                }

                self.prog.update_size((w / scale, h / scale));

                let (w, h) = (w as u32, h as u32);

                self.final_buffer_size.width = w;
                self.final_buffer_size.height = h;

                Self::resize_surface(surface, w, h);

                if let Ok(mut buffer) = surface.buffer_mut() {
                    buffer.fill(0x0);
                }
            }

            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                match event.key_without_modifiers().as_ref() {
                    Key::Character("q") => self.call_exit(event_loop),

                    Key::Named(NamedKey::Space) => self.prog.change_visualizer(true),

                    Key::Character("b") => self.prog.change_visualizer(false),

                    Key::Character("f") => self.prog.pix.toggle_running_foreign(),

                    Key::Character("n") => self.prog.change_vislist(),

                    Key::Character("-") => self.prog.decrease_vol_scl(),
                    Key::Character("=") => self.prog.increase_vol_scl(),

                    Key::Character("[") => self.prog.decrease_smoothing(),
                    Key::Character("]") => self.prog.increase_smoothing(),

                    Key::Character(";") => self.prog.decrease_wav_win(),
                    Key::Character("\'") => self.prog.increase_wav_win(),

                    Key::Character("\\") => self.prog.toggle_auto_switch(),

                    Key::Character("/") => self.prog.reset_parameters(),

                    _ => {}
                }
            }

            WindowEvent::RedrawRequested => {
                let Some(window) = self.window.as_ref() else {
                    return;
                };

                self.prog.update_vis();
                self.prog.render();

                if !self.prog.is_display_enabled() {
                    return;
                }

                let Some(Ok(mut buffer)) = self.surface.as_mut().map(|s| s.buffer_mut()) else {
                    return;
                };

                self.prog.pix.scale_to(
                    self.prog.scale() as usize,
                    &mut buffer,
                    Some(self.final_buffer_size.width as usize),
                    Some(u32::mix),
                    self.prog.is_crt(),
                );

                window.pre_present_notify();
                if let Err(e) = buffer.present() {
                    error!(
                        "Coffeevis is failing to present buffers to the window: {}!",
                        e
                    );
                }
            }

            _ => {}
        }
    }
}

impl WindowState {
    fn call_exit(&mut self, event_loop: &ActiveEventLoop) {
        self.exit_sender.as_ref().map(|x| x.send(()));
        self.thread_control_draw_id
            .take()
            .map(|t| t.join().unwrap());
        event_loop.exit();
    }

    fn resize_surface(surface: &mut WindowSurface, w: u32, h: u32) {
        surface
            .resize(
                NonZeroU32::new(w).expect("Surface width is zero"),
                NonZeroU32::new(h).expect("Surface height is zero"),
            )
            .expect("Failed to resize surface buffer");
    }

    fn check_refresh_rate(window: &Window, milli_hz_out: &mut u32) -> bool {
        let Some(Some(mut milli_hz)) = window
            .current_monitor()
            .map(|m| m.refresh_rate_millihertz())
        else {
            alert!("Coffeevis is unable to query your monitor's refresh rate.");
            return false;
        };

        if *milli_hz_out == milli_hz {
            return true;
        }

        info!(
            "\
        Detected rate to be {}hz.\n\
        Note: Coffeevis relies on Winit to detect refresh rates.\n\
        Run with --fps flag if you want to lock fps.\n\
        Refresh rate changed.\
        ",
            milli_hz as f32 / 1000.0
        );

        if milli_hz > CAP_MILLI_HZ {
            milli_hz = CAP_MILLI_HZ;
            info!("(Refresh rate has been capped to {}.)", CAP_MILLI_HZ);
        }

        info!("");

        *milli_hz_out = milli_hz;

        true
    }
}

fn read_icon() -> Option<(u32, u32, Vec<u8>)> {
    let icon_file = include_bytes!("../../assets/coffeevis_icon_128x128.qoi");

    let mut icon = Decoder::new(icon_file)
        .map(|i| i.with_channels(qoi::Channels::Rgba))
        .ok()?;

    let &Header { width, height, .. } = icon.header();

    icon.decode_to_vec().ok().map(|v| (width, height, v))
}

pub fn winit_main(prog: Program) {
    let event_loop = EventLoop::new().unwrap();

    let mut state = WindowState {
        prog,
        window: None,
        surface: None,
        exit_sender: None,
        thread_control_draw_id: None,
        final_buffer_size: PhysicalSize::<u32>::new(0, 0),
    };

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut state).unwrap();
}
