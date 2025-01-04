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
    sync::Arc,
    thread::sleep,
    time::{Duration, Instant},
};

use crate::data::*;
use crate::graphics::blend::Blend;

struct WindowState {
    pub window: Option<Arc<winit::window::Window>>,
    pub surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    pub prog: Program,
    pub poll_deadline: Instant,
    pub refresh_rate_check_deadline: Instant,
    pub final_buffer_size: PhysicalSize<u32>,
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
            eprintln!("Running in the {} desktop (XDG_CURRENT_DESKTOP).", de);
        }

        if gnome_workaround {
            sleep(Duration::from_millis(50));
            window.set_decorations(true);
        }

        if !resizeable {
            window.set_min_inner_size(Some(win_size));
            window.set_max_inner_size(Some(win_size));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::MouseInput { button, .. } => {
                if button == event::MouseButton::Left {
                    if let Err(err) = self.window.as_ref().unwrap().drag_window() {
                        eprintln!("Error starting window drag: {err}");
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

                let scale = self.prog.scale() as u32;

                let w = width;
                let h = height.min(MAX_PIXEL_BUFFER_SIZE / w);

                self.final_buffer_size.width = w;
                self.final_buffer_size.height = h;

                surface
                    .resize(NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap())
                    .unwrap();

                self.prog.update_size((w / scale, h / scale));

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

                let Some(surface) = self.surface.as_mut() else {
                    return;
                };

                window.request_redraw();

                let no_sample = crate::audio::get_no_sample();

                if window.is_minimized().unwrap_or(false)
                    || self.prog.is_hidden()
                    || no_sample >= crate::data::STOP_RENDERING
                {
                    sleep(IDLE_INTERVAL);
                    return;
                }

                if self.prog.refresh_rate_mode != crate::RefreshRateMode::Specified {
                    let now = Instant::now();
                    if now > self.refresh_rate_check_deadline {
                        check_refresh_rate(window, &mut self.prog);
                        self.refresh_rate_check_deadline = now + Duration::from_secs(1);
                    }
                }

                self.prog.update_vis();
                self.prog.render();

                if self.prog.is_display_enabled() {
                    if let Ok(mut buffer) = surface.buffer_mut() {
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
                }

                let now = Instant::now();
                if now < self.poll_deadline {
                    sleep(self.poll_deadline - now);
                }

                self.poll_deadline = Instant::now() + self.prog.get_rr_interval(no_sample);
            }

            _ => {}
        }
    }
}

pub fn read_icon() -> (u32, u32, Vec<u8>) {
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

pub fn check_refresh_rate(window: &Window, prog: &mut Program) {
    let Some(monitor) = window.current_monitor() else {
        return;
    };

    let Some(milli_hz) = monitor.refresh_rate_millihertz() else {
        return;
    };

    if milli_hz == prog.milli_hz {
        return;
    }

    eprintln!(
        "\
    Detected rate to be {}hz.\n\
    Note: Coffeevis relies on Winit to detect refresh rates.\n\
    Run with --fps flag if you want to lock fps.\n\
    Refresh rate changed.\
    ",
        milli_hz as f32 / 1000.0
    );

    prog.change_fps_frac(milli_hz);
}

pub fn winit_main(prog: Program) {
    let event_loop = EventLoop::new().unwrap();

    let mut state = WindowState {
        window: None,
        surface: None,
        poll_deadline: Instant::now() + prog.refresh_rate_intervals[0],
        refresh_rate_check_deadline: Instant::now() + Duration::from_secs(1),
        prog,
        final_buffer_size: PhysicalSize::<u32>::new(0, 0),
    };

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut state).unwrap();
}
