use softbuffer::{Context, Surface};

use crate::data::log::{alert, error, info};

use qoi;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{self, ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::Key,
    window::{Icon, Window, WindowButtons, WindowId, WindowLevel},
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
    cell::LazyCell,
    num::NonZeroU32,
    sync::mpsc::{self, RecvTimeoutError, SyncSender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::data::*;

const ICON_FILE: &[u8] = include_bytes!("../../assets/coffeevis_icon_128x128.qoi");

thread_local! {
    static ICON: LazyCell<(u32, u32, Vec<u8>)> = LazyCell::new(|| {
        use qoi::{Decoder, Header};

        let mut icon = Decoder::new(ICON_FILE)
            .map(|i| i.with_channels(qoi::Channels::Rgba))
            .ok()
            .unwrap();

        let &Header { width, height, .. } = icon.header();

        (width, height, icon.decode_to_vec().unwrap())
    });
}

pub struct WindowProps {
    pub width: u16,
    pub height: u16,
}

impl WindowProps {
    pub fn set_size<T: TryInto<u16>>(&mut self, s: (T, T)) {
        self.width = s.0.try_into().ok().unwrap();
        self.height = s.1.try_into().ok().unwrap();
    }
}

enum LoopCode {
    Continue(u8),
    Exit,
}

struct Renderer {
    pub window: &'static Window,
    pub surface: Surface<&'static Window, &'static Window>,
    pub thread_control_draw_id: JoinHandle<()>,
    pub loopcode_sender: SyncSender<LoopCode>,
}

struct WindowState {
    pub prog: Program,
    pub renderer: Option<Renderer>,
    pub final_buffer_size: PhysicalSize<u32>,
}

impl ApplicationHandler for WindowState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Prevents re-initialization.
        assert!(self.renderer.is_none());

        let scale = self.prog.scale() as u32;
        let win_size = PhysicalSize::<u32>::new(
            self.prog.pix.width() as u32 * scale,
            self.prog.pix.height() as u32 * scale,
        );

        let de = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();

        let icon = ICON.with(|icon| {
            Icon::from_rgba(icon.2.to_vec(), icon.0, icon.1)
                .inspect_err(|_| error!("Failed to create window icon."))
                .ok()
        });

        let window_attributes = Window::default_attributes()
            .with_title("cvis")
            .with_inner_size(win_size)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_transparent(self.prog.is_transparent())
            .with_resizable(self.prog.is_resizable())
            .with_visible(false)
            .with_enabled_buttons(WindowButtons::CLOSE)
            .with_window_icon(icon);

        #[cfg(target_os = "linux")]
        let window_attributes = window_attributes.with_name("coffeevis", "cvis");

        #[cfg(target_os = "windows")]
        let window_attributes = window_attributes.with_class_name("coffeevis");

        let window = &*Box::leak(Box::new(
            event_loop.create_window(window_attributes).unwrap(),
        ));

        let size = window.inner_size();
        self.final_buffer_size = size;

        let surface = Surface::new(&Context::new(window).unwrap(), window).unwrap();

        if !de.is_empty() {
            info!("Running in the {} desktop (XDG_CURRENT_DESKTOP).", de);
        }

        // On XFCE this is needed to lock the size of the window.
        if !self.prog.is_resizable() {
            window.set_min_inner_size(Some(win_size));
            window.set_max_inner_size(Some(win_size));
        }

        let (loopcode_sender, loopcode_receiver) = mpsc::sync_channel::<LoopCode>(1);

        let mut milli_hz = self.prog.get_milli_hz();
        let rr_mode = self.prog.get_rr_mode();

        let mut intervals = self.prog.get_rr_intervals().to_vec();

        window.set_visible(true);
        window.set_window_level(WindowLevel::AlwaysOnTop);

        // Thread to control requesting redraws.
        let thread_control_draw_id = thread::Builder::new()
            .name("coffeevis draw control".into())
            .spawn(move || {
                window.request_redraw();

                if rr_mode == RefreshRateMode::Sync {
                    // Wait for a little before querying for monitor refresh rate.
                    thread::sleep(Duration::from_millis(100));
                    Self::check_refresh_rate(window, &mut milli_hz);
                    intervals = Program::construct_intervals(milli_hz).to_vec();
                }

                let mut last = Instant::now();

                loop {
                    match loopcode_receiver.recv_timeout(Duration::from_millis(750)) {
                        Ok(LoopCode::Continue(s)) => {
                            if s < STOP_RENDERING {
                                window.request_redraw();
                            }

                            let itvl = intervals[(s > SLOW_DOWN_THRESHOLD) as usize];

                            let now = Instant::now();
                            let next = last + itvl;
                            thread::sleep(next.duration_since(now));
                            last = next;
                        }

                        Err(RecvTimeoutError::Timeout) => window.request_redraw(),
                        _ => break,
                    }
                }
            })
            .unwrap();

        self.renderer = Some(Renderer {
            window,
            surface,
            thread_control_draw_id,
            loopcode_sender,
        })
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                let Some(Renderer {
                    window,
                    surface,
                    loopcode_sender,
                    ..
                }) = self.renderer.as_mut()
                else {
                    return;
                };

                let Ok(mut buffer) = surface.buffer_mut() else {
                    return;
                };

                let silent = {
                    let mut buf = crate::audio::get_buf();
                    self.prog.autoupdate_visualizer();
                    self.prog.render(&mut buf);
                    buf.silent()
                };

                if !self.prog.is_display_enabled() {
                    return;
                }

                self.prog.pix.scale_to(
                    self.prog.scale() as usize,
                    &mut buffer,
                    Some(self.final_buffer_size.width as usize),
                    self.prog.get_win_render_effect(),
                );

                window.pre_present_notify();

                if let Err(e) = buffer.present() {
                    error!(
                        "Coffeevis is failing to present buffers to the window: {}!",
                        e
                    );
                }

                let _ = loopcode_sender.send(LoopCode::Continue(silent));
            }

            WindowEvent::CloseRequested => self.call_exit(event_loop),

            WindowEvent::MouseInput { button, .. } => {
                if button == event::MouseButton::Left
                    && let Some(err) = self
                        .renderer
                        .as_ref()
                        .and_then(|f| f.window.drag_window().err())
                {
                    error!("Error dragging window: {}", err);
                }
            }

            WindowEvent::Focused(_) => {
                if let Some(r) = self.renderer.as_ref() {
                    r.window.request_redraw()
                }
            }

            WindowEvent::Resized(PhysicalSize { width, height }) => {
                let Some(Renderer { surface, .. }) = self.renderer.as_mut() else {
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

                surface
                    .resize(
                        NonZeroU32::new(w).expect("Surface width is zero"),
                        NonZeroU32::new(h).expect("Surface height is zero"),
                    )
                    .expect("Failed to resize surface buffer");

                self.prog.pix.clear_out_buffer();
            }

            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                match event.key_without_modifiers().as_ref() {
                    Key::Character("q") => self.call_exit(event_loop),
                    Key::Character("n") => self.prog.change_visualizer(true),
                    Key::Character("b") => self.prog.change_visualizer(false),

                    Key::Character("\\") => self.prog.toggle_auto_switch(),
                    Key::Character("/") => self.prog.reset_parameters(),

                    _ => {}
                }
            }

            _ => {}
        }
    }
}

impl WindowState {
    fn call_exit(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(t) = self.renderer.take() {
            t.loopcode_sender.send(LoopCode::Exit).unwrap();
            t.thread_control_draw_id.join().unwrap()
        }

        event_loop.exit();
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

pub fn winit_main(prog: Program) {
    let event_loop = EventLoop::new().unwrap();

    let mut state = WindowState {
        prog,
        renderer: None,
        final_buffer_size: PhysicalSize::<u32>::new(0, 0),
    };

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut state).unwrap();
}
