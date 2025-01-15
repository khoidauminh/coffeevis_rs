use softbuffer::Surface;

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{self, ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{
        Key,
        NamedKey::{Escape, Space},
    },
    platform::{
        modifier_supplement::KeyEventExtModifierSupplement, wayland::WindowAttributesExtWayland,
    },
    window::{Icon, Window, WindowId, WindowLevel},
};

use std::{
    num::NonZeroU32,
    sync::{mpsc, Arc},
    thread::{self, sleep},
    time::{Duration, Instant},
};

use crate::graphics::blend::Blend;
use crate::{audio::get_no_sample, data::*};

struct WindowState {
    pub window: Option<Arc<Window>>,
    pub surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    pub prog: Program,
    pub poll_deadline: Instant,
    pub refresh_rate_check_deadline: Instant,
    pub final_buffer_size: PhysicalSize<u32>,
    pub exit_sender: Option<mpsc::SyncSender<()>>,
}

impl ApplicationHandler for WindowState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
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

        let icon = {
            let (w, h, v) = read_icon();
            Icon::from_rgba(v, w, h).expect("Failed to create window icon.")
        };

        let window_attributes = Window::default_attributes()
            .with_title("cvis")
            .with_inner_size(win_size)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_transparent(false)
            .with_decorations(!(is_gnome && is_wayland))
            .with_resizable(self.prog.is_resizable())
            .with_name("coffeevis", "cvis")
            .with_window_icon(Some(icon));

        self.window = Some(Arc::new(
            event_loop.create_window(window_attributes).unwrap(),
        ));

        let window = self.window.as_ref().unwrap().clone();
        let size = window.inner_size();
        self.final_buffer_size = size;

        self.surface = {
            let context = softbuffer::Context::new(window.clone()).unwrap();
            let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

            surface
                .resize(
                    NonZeroU32::new(size.width).unwrap(),
                    NonZeroU32::new(size.height).unwrap(),
                )
                .unwrap();

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
            while !exit_recv.recv_timeout(IDLE_INTERVAL).is_ok() {
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
                    if let Err(err) = self.window.as_ref().unwrap().drag_window() {
                        self.prog
                            .print_message(format!("Error starting window drag: {err}"));
                    }
                }
            }

            WindowEvent::Occluded(b) => {
                self.prog.set_hidden(b);
            }

            WindowEvent::Resized(PhysicalSize { width, height }) => {
                let Some(surface) = self.surface.as_mut() else {
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

                surface
                    .resize(NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap())
                    .unwrap();

                if let Ok(mut buffer) = surface.buffer_mut() {
                    buffer.fill(0x0);
                }
            }

            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                match event.key_without_modifiers().as_ref() {
                    Key::Named(Escape) => event_loop.exit(),

                    Key::Named(Space) => self.prog.change_visualizer(true),

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

                let no_sample = crate::audio::get_no_sample();

                if window.is_minimized().unwrap_or(false)
                    || self.prog.is_hidden()
                    || no_sample >= STOP_RENDERING
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

                'render: {
                    if !self.prog.is_display_enabled() {
                        break 'render;
                    }

                    let Some(surface) = self.surface.as_mut() else {
                        break 'render;
                    };

                    let Ok(mut buffer) = surface.buffer_mut() else {
                        break 'render;
                    };

                    self.prog.pix.scale_to(
                        self.prog.scale() as usize,
                        &mut buffer,
                        Some(self.final_buffer_size.width as usize),
                        Some(u32::mix),
                        self.prog.is_crt(),
                    );

                    window.pre_present_notify();
                    let _ = buffer.present();
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

    fn check_refresh_rate(window: &Window, prog: &mut Program) {
        let Some(monitor) = window.current_monitor() else {
            return;
        };

        let Some(mut milli_hz) = monitor.refresh_rate_millihertz() else {
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

fn read_icon() -> (u32, u32, Vec<u8>) {
    let icon_file = include_bytes!("../../assets/coffeevis_icon_128x128.qoi");

    let mut icon = qoi::Decoder::new(icon_file)
        .expect("Failed to parse qoi image")
        .with_channels(qoi::Channels::Rgba);

    let header = icon.header();
    let width = header.width;
    let height = header.height;

    let Ok(vec) = icon.decode_to_vec() else {
        panic!("Failed to decode qoi to vec.")
    };

    (width, height, vec)
}

pub fn winit_main(prog: Program) {
    let event_loop = EventLoop::new().unwrap();

    let mut state = WindowState {
        window: None,
        surface: None,
        poll_deadline: Instant::now() + prog.get_rr_interval(0),
        refresh_rate_check_deadline: Instant::now() + Duration::from_secs(1),
        prog,
        final_buffer_size: PhysicalSize::<u32>::new(0, 0),
        exit_sender: None,
    };

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut state).unwrap();
    state.exit_sender.as_ref().map(|x| x.send(()));
}
