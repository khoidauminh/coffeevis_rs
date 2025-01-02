use winit::{
    application::ApplicationHandler,
    dpi::{self, LogicalSize, PhysicalSize},
    event::{self, DeviceEvent, DeviceId, ElementState, Event, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{
        Key,
        NamedKey::{Escape, Space},
    },
    platform::{
        modifier_supplement::KeyEventExtModifierSupplement, wayland::WindowAttributesExtWayland,
    },
    window::{Icon, Window, WindowButtons, WindowId, WindowLevel},
};

use std::{
    num::NonZeroU32,
    sync::{mpsc, Arc, RwLock},
    thread::{self, current},
    time::{Duration, Instant},
};

use crate::data::*;
use crate::graphics::blend::Blend;
use crate::graphics::Canvas;

struct WindowState {
    pub window: Arc<winit::window::Window>,
    pub commands_sender: mpsc::SyncSender<Command>,
}

impl WindowState {
    fn request_close(&self) {
        self.commands_sender.send(Command::Close);
    }
}

impl ApplicationHandler for WindowState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.request_close();
                event_loop.exit();
            }

            WindowEvent::MouseInput { button, .. } => {
                if button == event::MouseButton::Left {
                    if let Err(err) = self.window.drag_window() {
                        eprintln!("Error starting window drag: {err}");
                    }
                }
            }

            WindowEvent::Occluded(b) => {
                println!("Occluded: {}", b);
                self.commands_sender.send(Command::Hidden(b));
            }

            WindowEvent::Resized(PhysicalSize { width, height }) => {
                self.commands_sender
                    .send(Command::Resize(width as u16, height as u16));
            }

            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                self.commands_sender
                    .send(match event.key_without_modifiers().as_ref() {
                        Key::Named(Escape) => {
                            event_loop.exit();
                            self.request_close();
                            Command::Blank
                        }

                        Key::Named(Space) => Command::VisualizerNext,

                        Key::Character("b") => Command::VisualizerPrev,

                        Key::Character("n") => Command::SwitchVisList,

                        Key::Character("-") => Command::VolDown,
                        Key::Character("=") => Command::VolUp,

                        Key::Character("[") => Command::SmoothDown,
                        Key::Character("]") => Command::SmoothUp,

                        Key::Character(";") => Command::WavDown,
                        Key::Character("\'") => Command::WavUp,

                        Key::Character("\\") => Command::AutoSwitch,

                        Key::Character("/") => Command::Reset,

                        _ => Command::Blank,
                    });
            }

            _ => {}
        }
    }
}

pub fn read_icon() -> (u32, u32, Vec<u8>) {
    let ICON_FILE = include_bytes!("../../assets/coffeevis_icon_128x128.qoi");

    let mut icon = qoi::Decoder::new(ICON_FILE)
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

pub fn winit_main(mut prog: Program) {
    prog.print_startup_info();

    let event_loop = EventLoop::new().unwrap();

    let scale = prog.scale() as u32;
    let mut size = (prog.pix.width() as u32, prog.pix.height() as u32);
    size.0 *= scale;
    size.1 *= scale;

    let lock_fps = prog.REFRESH_RATE_MODE == crate::RefreshRateMode::Specified;
    let resizeable = prog.is_resizable();
    let de = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or(String::new());
    let is_gnome = de == "GNOME";
    let is_wayland = prog.is_wayland();
    let gnome_workaround = is_gnome && is_wayland;

    let win_size = dpi::LogicalSize::<u32>::new(size.0, size.1);

    let icon = {
        let (w, h, v) = read_icon();
        Icon::from_rgba(v, w, h).expect("Failed to create window icon.")
    };

    let mut window_attributes = Window::default_attributes()
        .with_title("cvis")
        .with_inner_size(win_size)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_transparent(false)
        .with_decorations(!(is_gnome && is_wayland))
        .with_resizable(prog.is_resizable())
        .with_name("coffeevis", "cvis")
        .with_window_icon(Some(icon));

    let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

    let inner_size = window.clone().inner_size();

    let (commands_sender, commands_receiver) = mpsc::sync_channel::<Command>(8);

    let mut state = WindowState {
        window: window.clone(),
        commands_sender: commands_sender.clone(),
    };

    let thread_size = thread::spawn({
        let window = window.clone();

        move || {
            if !de.is_empty() {
                eprintln!("Running in the {} desktop (XDG_CURRENT_DESKTOP).", de);
            }

            if gnome_workaround {
                thread::sleep(Duration::from_millis(50));
                window.set_decorations(true);
            }

            if !resizeable {
                window.set_min_inner_size(Some(win_size));
                window.set_max_inner_size(Some(win_size));
            }
        }
    });

    let thread_updates = thread::spawn({
        let window = window.clone();
        let commands_sender = commands_sender.clone();
        move || {
            if lock_fps {
                return;
            }

            thread::sleep(Duration::from_millis(500));

            if let Some(monitor) = window.current_monitor() {
                if let Some(mut milli_hz) = monitor.refresh_rate_millihertz() {
                    eprintln!(
                        "\
                        Detected rate to be {}hz.\n\
                        Note: Coffeevis relies on Winit to detect refresh rates.\n\
                        Run with --fps flag if you want to lock fps.\n\
                        Refresh rate changed.\
                        ",
                        milli_hz as f32 / 1000.0
                    );

                    commands_sender.send(Command::FpsFrac(milli_hz));
                }
            }
        }
    });

    // This is the main drawing thread;
    let thread_draw = thread::spawn({
        let commands_sender = commands_sender.clone();
        let window = window.clone();
        let mut size = inner_size;

        let mut surface = {
            use winit::raw_window_handle;
            let context = softbuffer::Context::new(window.clone()).unwrap();
            let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

            surface
                .resize(
                    NonZeroU32::new(inner_size.width).unwrap(),
                    NonZeroU32::new(inner_size.height).unwrap(),
                )
                .unwrap();

            surface
        };

        let mut frame_deadline = Instant::now() + prog.DURATIONS[0];

        move || loop {
            let mut draw = false;

            if let Ok(mut cmd) = commands_receiver.try_recv() {
                if cmd.is_close_requested() {
                    break;
                }

                if let Command::Resize(w, h) = cmd {
                    let w = w as u32;
                    let h = (h as u32).min(MAX_PIXEL_BUFFER_SIZE / w);

                    size.width = w as u32;
                    size.height = h as u32;

                    surface
                        .resize(
                            NonZeroU32::new(w as u32).unwrap(),
                            NonZeroU32::new(h as u32).unwrap(),
                        )
                        .unwrap();

                    prog.update_size((w / scale, h / scale));

                    if let Ok(mut buffer) = surface.buffer_mut() {
                        buffer.fill(0x0);
                    }
                }

                draw |= prog.eval_command(&cmd);
            }

            prog.update_vis();
            let no_sample = crate::audio::get_no_sample();
            let sleep_index = no_sample as usize * prog.DURATIONS.len() / 256;

            if window.is_minimized().unwrap_or(false)
                || prog.is_hidden()
                || no_sample >= crate::data::STOP_RENDERING && !draw
            {
                thread::sleep(IDLE_INTERVAL);
                continue;
            }

            prog.force_render();

            if prog.is_display_enabled() {
                if let Ok(mut buffer) = surface.buffer_mut() {
                    let len = buffer.len().min(prog.pix.sizel());

                    prog.pix.scale_to(
                        prog.scale() as usize,
                        &mut buffer,
                        Some(size.width as usize),
                        Some(u32::mix),
                        prog.is_crt(),
                    );

                    window.pre_present_notify();
                    let _ = buffer.present();
                }
            }

            let now = Instant::now();
            if now < frame_deadline {
                thread::sleep(frame_deadline - now);
            }

            frame_deadline = Instant::now() + prog.DURATIONS[sleep_index];
        }
    });

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    event_loop.run_app(&mut state).unwrap();
    thread_updates.join().unwrap();
    thread_size.join().unwrap();
    thread_draw.join().unwrap();
}
