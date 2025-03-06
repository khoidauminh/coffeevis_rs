use softbuffer::{Context, Surface};

use qoi::{Decoder, Header};

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{self, ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    platform::{
        modifier_supplement::KeyEventExtModifierSupplement, wayland::WindowAttributesExtWayland,
    },
    window::{Icon, Theme, Window, WindowId, WindowLevel},
};

use std::{
    num::NonZeroU32,
    sync::mpsc::{self, SyncSender},
    thread,
};

use crate::graphics::Pixel;
use crate::{audio::get_no_sample, data::*};

type WindowSurface = Surface<&'static Window, &'static Window>;

struct WindowState {
    pub prog: Program,
    pub window: Option<&'static Window>,
    pub surface: Option<WindowSurface>,
    pub exit_sender: Option<SyncSender<()>>,
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
                .inspect_err(|_| self.prog.print_message("Failed to create window icon.\n"))
                .ok()
        });

        let window_attributes = Window::default_attributes()
            .with_title("cvis")
            .with_inner_size(win_size)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_transparent(false)
            .with_resizable(self.prog.is_resizable())
            .with_name("coffeevis", "cvis")
            .with_theme(Some(Theme::Dark))
            .with_window_icon(icon);

        // Since we are leaking the window into a static
        // reference, resumed() is not allowed to be
        // called again as it would cause the build up
        // of leaked windows and potentially flood RAM.
        match self.window {
            None => {
                self.window = Some(Box::leak(Box::new(
                    event_loop.create_window(window_attributes).unwrap(),
                )))
            }

            Some(_) => panic!("Resume being called the 2nd time!"),
        }

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
            self.prog.print_message(format!(
                "Running in the {} desktop (XDG_CURRENT_DESKTOP).\n",
                de
            ));
        }

        // On XFCE this is needed to lock the size of the window.
        if !self.prog.is_resizable() {
            window.set_min_inner_size(Some(win_size));
            window.set_max_inner_size(Some(win_size));
        }

        let (exit_send, exit_recv) = mpsc::sync_channel(1);

        self.exit_sender = Some(exit_send);

        if self.prog.get_rr_mode() != RefreshRateMode::Specified {
            Self::check_refresh_rate(window, &mut self.prog);
        }

        let intervals = self.prog.get_rr_intervals();

        // Thread to control requesting redraws.
        let _ = thread::Builder::new().stack_size(1024).spawn(move || {
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
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::MouseInput { button, .. } => {
                if button == event::MouseButton::Left {
                    if let Some(err) = self.window.as_ref().and_then(|f| f.drag_window().err()) {
                        self.prog
                            .print_message(format!("Error dragging window: {err}"));
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
                    self.prog
                        .print_message("Coffeevis is unable to resize the buffer!\n");
                    return;
                };

                let scale = self.prog.scale() as u16;

                let w = u16::min(MAX_WIDTH, width as u16);
                let h = u16::min(MAX_HEIGHT, height as u16);

                if w == MAX_WIDTH || h == MAX_HEIGHT {
                    self.prog.print_message(format_red!(
                        "You are hitting the resolution limit of Coffeevis!\n"
                    ));
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
                    Key::Named(NamedKey::Escape) => event_loop.exit(),

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

                // if window.is_minimized().unwrap_or(false)
                //     || self.prog.is_hidden()
                //     || get_no_sample() >= STOP_RENDERING
                // {
                //     return;
                // }

                self.prog.update_vis();
                self.prog.render();

                if !self.prog.is_display_enabled() {
                    return;
                }

                if let Some(Ok(mut buffer)) = self.surface.as_mut().map(|s| s.buffer_mut()) {
                    self.prog.pix.scale_to(
                        self.prog.scale() as usize,
                        &mut buffer,
                        Some(self.final_buffer_size.width as usize),
                        Some(u32::mix),
                        self.prog.is_crt(),
                    );

                    window.pre_present_notify();
                    if let Err(e) = buffer.present() {
                        self.prog.print_message(format!(
                            "Coffeevis is failing to present buffers to the window: {e}.\n"
                        ));
                    }
                }
            }

            _ => {}
        }
    }
}

impl WindowState {
    fn resize_surface(surface: &mut WindowSurface, w: u32, h: u32) {
        surface
            .resize(
                NonZeroU32::new(w).expect("Surface width is zero"),
                NonZeroU32::new(h).expect("Surface height is zero"),
            )
            .expect("Failed to resize surface buffer");
    }

    fn check_refresh_rate(window: &Window, prog: &mut Program) {
        let Some(Some(mut milli_hz)) = window
            .current_monitor()
            .map(|m| m.refresh_rate_millihertz())
        else {
            prog.print_message("Coffeevis is unable to query your monitor's refresh rate.\n");
            return;
        };

        if milli_hz == prog.get_milli_hz() {
            return;
        }

        prog.print_message(format!(
            "\
        Detected rate to be {}hz.\n\
        Note: Coffeevis relies on Winit to detect refresh rates.\n\
        Run with --fps flag if you want to lock fps.\n\
        Refresh rate changed.\n\
        ",
            milli_hz as f32 / 1000.0
        ));

        if milli_hz > CAP_MILLI_HZ {
            milli_hz = CAP_MILLI_HZ;
            prog.print_message(format!(
                "(Refresh rate has been capped to {}.)\n",
                CAP_MILLI_HZ
            ));
        }

        prog.change_fps_frac(milli_hz);
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
        final_buffer_size: PhysicalSize::<u32>::new(0, 0),
    };

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut state).unwrap();
    state.exit_sender.as_ref().map(|x| x.send(()));
}
