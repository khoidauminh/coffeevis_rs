#![allow(warnings)]

use std::{sync::Arc, thread};

use crate::data::*;

use fps_clock::FpsClock;
#[cfg(feature = "minifb")]
use minifb::{Window, WindowOptions};

#[cfg(feature = "minifb")]
pub fn init_window(prog: &Program) -> Result<Window, minifb::Error> {
    std::env::set_var("GDK_BACKEND", "x11");
    std::env::set_var("LANG", "en_US-UTF-8");

    let mut win = Window::new(
        "kvis",
        prog.pix.width() * prog.scale() as usize,
        prog.pix.height() * prog.scale() as usize,
        WindowOptions {
            // resize: prog.RESIZE,
            topmost: true,
            borderless: false,
            transparency: false,
            scale_mode: minifb::ScaleMode::UpperLeft,
            ..WindowOptions::default()
        },
    )?;

    win.limit_update_rate(Some(prog.REFRESH_RATE));

    Ok(win)
}

#[cfg(feature = "minifb")]
pub fn control_key_events_win_legacy(win: &mut minifb::Window, prog: &mut Program) {
    use minifb::{Key, KeyRepeat};

    let fps_change = false;

    prog.update_vis();

    win.get_keys_pressed(KeyRepeat::No)
        .iter()
        .for_each(|key| match key {
            Key::Space => prog.change_visualizer(true),

            //~ Key::Key1 =>  { change_fps(prog, 10, true); fps_change = true; },
            //~ Key::Key2 =>  { change_fps(prog, 20, true); fps_change = true; },
            //~ Key::Key3 =>  { change_fps(prog, 30, true); fps_change = true; },
            //~ Key::Key4 =>  { change_fps(prog, 40, true); fps_change = true; },
            //~ Key::Key5 =>  { change_fps(prog, 50, true); fps_change = true; },
            //~ Key::Key6 =>  { change_fps(prog, 60, true); fps_change = true; },

            //~ Key::Key7 =>  { change_fps(prog, -5, false); fps_change = true; },
            //~ Key::Key8 =>  { change_fps(prog,  5, false); fps_change = true; },
            Key::Minus => prog.VOL_SCL = (prog.VOL_SCL / 1.2).clamp(0.0, 10.0),
            Key::Equal => prog.VOL_SCL = (prog.VOL_SCL * 1.2).clamp(0.0, 10.0),

            Key::LeftBracket => prog.SMOOTHING = (prog.SMOOTHING - 0.05).clamp(0.0, 0.95),
            Key::RightBracket => prog.SMOOTHING = (prog.SMOOTHING + 0.05).clamp(0.0, 0.95),

            Key::Semicolon => prog.WAV_WIN = (prog.WAV_WIN - 3).clamp(3, 50),
            Key::Apostrophe => prog.WAV_WIN = (prog.WAV_WIN + 3).clamp(3, 50),

            Key::Backslash => prog.toggle_auto_switch(),

            Key::Slash => {
                prog.VOL_SCL = DEFAULT_VOL_SCL;
                prog.SMOOTHING = DEFAULT_SMOOTHING;
                prog.WAV_WIN = DEFAULT_WAV_WIN;
                // change_fps(prog, 144, true);
            }

            _ => {}
        });

    if fps_change {
        win.limit_update_rate(Some(prog.REFRESH_RATE));
    }
}

#[cfg(feature = "minifb")]
pub fn minifb_main(mut prog: Program) -> Result<(), minifb::Error> {
    let mut win = init_window(&prog)?;
    win.topmost(true);

    let scale = prog.scale() as usize;

    while win.is_open() && !win.is_key_down(minifb::Key::Q) {
        let s = win.get_size();

        let s = (s.0 / scale, s.1 / scale);

        if s.0 != prog.pix.width() || s.1 != prog.pix.height() {
            prog.update_size_win(s);
        }

        control_key_events_win_legacy(&mut win, &mut prog);

        if crate::audio::get_no_sample() > 64 {
            win.update();
            continue;
        }

        prog.force_render();

        let winw = prog.pix.width() * scale;
        let winh = prog.pix.height() * scale;

        if scale == 1 {
            let _ = win.update_with_buffer(prog.pix.as_slice(), winw, winh);
            continue;
        }

        let mut buffer = vec![0u32; winw * winh];

        prog.pix.scale_to(&mut buffer, scale, None);

        let _ = win.update_with_buffer(&buffer, winw, winh);
    }

    Ok(())
}

use winit::{
    application::ApplicationHandler,
    dpi::{self, LogicalSize},
    event::{DeviceEvent, DeviceId, ElementState, Event, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{
        Key,
        NamedKey::{Escape, Space},
    },
    platform::modifier_supplement::KeyEventExtModifierSupplement,
    window::{WindowButtons, WindowId},
};

use std::num::NonZeroU32;
use std::sync::{mpsc, RwLock};
use std::time::Duration;

use crate::data::Command;
use crate::graphics::Canvas;

struct WindowState {
    pub window: Arc<winit::window::Window>,
    pub commands_sender: std::sync::mpsc::SyncSender<Command>,
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
                if button == winit::event::MouseButton::Left {
                    if let Err(err) = self.window.drag_window() {
                        eprintln!("Error starting window drag: {err}");
                    }
                }
            }

            WindowEvent::Resized(winit::dpi::PhysicalSize { width, height }) => {
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

pub fn winit_main(mut prog: Program) -> Result<(), &'static str> {
    let event_loop = EventLoop::new().unwrap();

    let mut size = (prog.pix.width() as u32, prog.pix.height() as u32);
    let scale = prog.scale() as u32;

    size.0 *= scale;
    size.1 *= scale;

    let lock_fps = prog.REFRESH_RATE_MODE == crate::RefreshRateMode::Specified;

    let resizeable = prog.is_resizable();

    let de = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or(String::new());
    let is_gnome = de == "GNOME";
    let is_wayland = prog.is_wayland();
    let gnome_workaround = is_gnome && is_wayland;

    let win_size = dpi::PhysicalSize::<u32>::new(size.0, size.1);

    let icon = {
        let (w, h, v) = read_icon();

        winit::window::Icon::from_rgba(v, w, h).expect("Failed to create window icon.")
    };

    let mut window_attributes = winit::window::Window::default_attributes()
        .with_title("cvis")
        .with_inner_size(win_size)
        .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
        .with_transparent(false)
        .with_decorations(!(is_gnome && is_wayland))
        .with_resizable(prog.is_resizable())
        .with_window_icon(Some(icon));

    let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

    let inner_size = window.clone().inner_size();

    let (commands_sender, commands_receiver) = std::sync::mpsc::sync_channel::<Command>(8);

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

            // Tries to fetch monitor information for 5 times before giving up.
            for _ in 0..5 {
                thread::sleep(Duration::from_millis(500));

                if let Some(monitor) = window.current_monitor() {
                    if let Some(milli_hz) = monitor.refresh_rate_millihertz() {
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

                        return;
                    }
                }
            }
        }
    });

    // This is the main drawing thread;
    let thread_draw = thread::spawn({
        let commands_sender = commands_sender.clone();
        let window = window.clone();
        let mut size = inner_size;
        let mut ticker = fps_clock::FpsClock::new(prog.MILLI_HZ / 1000);
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

        move || loop {
            let mut draw = false;

            if let Ok(mut cmd) = commands_receiver.try_recv() {
                if cmd.is_close_requested() {
                    break;
                }

                if let Command::Resize(w, h) = cmd {
                    size.width = w as u32;
                    size.height = h as u32;

                    let wd = w as u32 / scale;
                    let hd = h as u32 / scale;

                    surface
                        .resize(
                            NonZeroU32::new(w as u32).unwrap(),
                            NonZeroU32::new(h as u32).unwrap(),
                        )
                        .unwrap();

                    prog.update_size((wd as u16, hd as u16));

                    if let Ok(mut buffer) = surface.buffer_mut() {
                        buffer.fill(0x0);
                    }
                }

                if let Command::FpsFrac(f) = cmd {
                    ticker = fps_clock::FpsClock::new(f / 1000);
                }

                draw |= prog.eval_command(&cmd);
            }

            prog.update_vis();
            let no_sample = crate::audio::get_no_sample();

            if no_sample >= crate::data::STOP_RENDERING && !draw {
                thread::sleep(Duration::from_millis(333));
                continue;
            }

            if let Ok(mut buffer) = surface.buffer_mut() {
                use crate::graphics::blend::Blend;

                prog.force_render();

                if prog.is_display_enabled() {
                    let len = buffer.len().min(prog.pix.sizel());

                    prog.pix.scale_to(
                        &mut buffer,
                        prog.scale() as usize,
                        Some(size.width as usize),
                        Some(u32::mix),
                    );

                    window.pre_present_notify();
                    let _ = buffer.present();
                }
            }

            let sleep_index = (no_sample >> 6) as usize;

            let sleep = prog.DURATIONS[sleep_index];

            if sleep_index == 0 {
                ticker.tick();
            } else {
                thread::sleep(sleep);
            }
        }
    });

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    event_loop.run_app(&mut state).unwrap();
    thread_updates.join().unwrap();
    thread_size.join().unwrap();
    thread_draw.join().unwrap();

    Ok(())
}
