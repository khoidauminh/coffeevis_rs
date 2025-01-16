use softbuffer::{Context, Surface};

use qoi::{Decoder, Header};

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{self, ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    platform::{
        modifier_supplement::KeyEventExtModifierSupplement, wayland::WindowAttributesExtWayland,
    },
    window::{Icon, Window, WindowId, WindowLevel},
};

use std::{
    num::NonZeroU32,
    sync::mpsc::{self, SyncSender},
    thread::{self, sleep},
    time::{Duration, Instant},
};

use crate::graphics::blend::Blend;
use crate::{audio::get_no_sample, data::*};

type WindowSurface = Surface<&'static Window, &'static Window>;

struct WindowState {
    pub prog: Program,
    pub window: Option<&'static Window>,
    pub surface: Option<WindowSurface>,
    pub exit_sender: Option<SyncSender<()>>,
    pub final_buffer_size: PhysicalSize<u32>,
    pub poll_deadline: Instant,
    pub refresh_rate_check_deadline: Instant,
    pub already_resumed: bool,
}

impl ApplicationHandler for WindowState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Since we are leaking the window into a static
        // reference, resumed() is not allowed to be
        // called again as it would cause the build up
        // of leaked windows and potentially flood RAM.
        assert!(!self.already_resumed);

        self.already_resumed = true;

        self.prog.print_startup_info();

        let scale = self.prog.scale() as u32;
        let mut size = (self.prog.pix.width() as u32, self.prog.pix.height() as u32);
        size.0 *= scale;
        size.1 *= scale;

        let resizeable = self.prog.is_resizable();
        let de = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        let is_gnome = de == "GNOME";
        let is_wayland = self.prog.is_wayland();
        let gnome_workaround = is_gnome && is_wayland;

        let win_size = LogicalSize::<u32>::new(size.0, size.1);

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
            .with_decorations(!(is_gnome && is_wayland))
            .with_resizable(self.prog.is_resizable())
            .with_name("coffeevis", "cvis")
            .with_window_icon(icon);

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
            self.prog.print_message(format!(
                "Running in the {} desktop (XDG_CURRENT_DESKTOP).\n",
                de
            ));
        }

        if gnome_workaround {
            sleep(Duration::from_millis(50));
            window.set_decorations(true);
        }

        if !resizeable {
            window.set_min_inner_size(Some(win_size));
            window.set_max_inner_size(Some(win_size));
        }

        let (exit_send, exit_recv) = mpsc::sync_channel(1);

        self.exit_sender = Some(exit_send);

        // Coffeevis needs to wind down as much as
        // possble when hidden or there's no input
        // received. However, sleeping in the main
        // thread will block the program from properly
        // processing other events.
        //
        // Secondly, coffeevis only does a request_redraw after
        // successfully rendering a frame, so if it's on idle
        // mode, no frame is drawn, no request is made and thus
        // it will be stuck idling. This thread will send a
        // request to kick start it.
        let _ = thread::Builder::new().stack_size(1024).spawn(move || {
            while exit_recv.recv_timeout(IDLE_INTERVAL).is_err() {
                window.request_redraw();
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

                if window.is_minimized().unwrap_or(false)
                    || self.prog.is_hidden()
                    || get_no_sample() >= STOP_RENDERING
                {
                    return;
                }

                if self.prog.get_rr_mode() != RefreshRateMode::Specified {
                    let now = Instant::now();
                    if now > self.refresh_rate_check_deadline {
                        Self::check_refresh_rate(window, &mut self.prog);
                        self.refresh_rate_check_deadline = now + Duration::from_secs(1);
                    }
                }

                self.prog.update_vis();
                self.prog.render();

                if self.prog.is_display_enabled() {
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

                window.request_redraw();

                self.wait();
            }

            _ => {}
        }
    }
}

impl WindowState {
    fn wait(&mut self) {
        let now = Instant::now();
        if now < self.poll_deadline {
            sleep(self.poll_deadline - now);
        }

        self.poll_deadline = Instant::now() + self.prog.get_rr_interval(get_no_sample());
    }

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

    let poll_deadline = Instant::now() + prog.get_rr_interval(0);
    let refresh_rate_check_deadline = Instant::now() + Duration::from_secs(1);

    let mut state = WindowState {
        prog,
        window: None,
        surface: None,
        exit_sender: None,
        final_buffer_size: PhysicalSize::<u32>::new(0, 0),
        poll_deadline,
        refresh_rate_check_deadline,
        already_resumed: false,
    };

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut state).unwrap();
    state.exit_sender.as_ref().map(|x| x.send(()));
}
