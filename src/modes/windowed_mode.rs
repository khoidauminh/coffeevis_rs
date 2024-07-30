#![allow(warnings)]

use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
        Arc,
    },
    thread,
};

use crate::data::*;

#[cfg(feature = "minifb")]
use minifb::{Window, WindowOptions};

#[cfg(feature = "minifb")]
pub fn init_window(prog: &Program) -> Result<Window, minifb::Error> {
    std::env::set_var("GDK_BACKEND", "x11");

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

    if cfg!(not(feature = "benchmark")) {
        win.limit_update_rate(Some(prog.REFRESH_RATE));
    }

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
            //let s = (s.0 / WIN_SCALE, s.1 / WIN_SCALE);
            prog.update_size_win(s);
        }

        control_key_events_win_legacy(&mut win, &mut prog);

        // prog.update_timer();

        // prog.update_state();

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
    dpi,
    event::{DeviceEvent, DeviceId, ElementState, Event, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{
        Key,
        NamedKey::{Escape, Space},
    },
    platform::modifier_supplement::KeyEventExtModifierSupplement,
    window::WindowId,
};

use std::num::NonZeroU32;
use std::sync::RwLock;
use std::time::Duration;

use crate::data::Command;
use crate::graphics::Canvas;

fn set_exit(b: Arc<AtomicBool>) {
    b.store(false, Relaxed);
}

struct WindowState {
    pub thread_main_running: Arc<AtomicBool>,
    pub window: Arc<winit::window::Window>,
    pub commands: Arc<RwLock<Command>>,
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
                set_exit(self.thread_main_running.clone());
                event_loop.exit();
            }

            WindowEvent::MouseInput { button: button, .. } => {
                if button == winit::event::MouseButton::Left {
                    if let Err(err) = self.window.drag_window() {
                        eprintln!("Error starting window drag: {err}");
                    }
                }
            }

            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                match self.commands.try_write() {
                    Ok(mut cmd) => {
                        *cmd = match event.key_without_modifiers().as_ref() {
                            Key::Named(Escape) => {
                                set_exit(self.thread_main_running.clone());
                                event_loop.exit();

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
                        };
                    }

                    Err(_) => {}
                }
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

    size.0 *= prog.scale() as u32;
    size.1 *= prog.scale() as u32;

    let lock_fps = prog.REFRESH_RATE_MODE == crate::RefreshRateMode::Specified
        || prog.REFRESH_RATE_MODE == crate::RefreshRateMode::Unlimited;

    let report_fps = prog.HZ_REPORT;

    std::env::set_var("WINIT_X11_SCALE_FACTOR", prog.scale().to_string());

    if prog.transparency < 255 {
        eprintln!(
            "WARNING: transparency doesn't work for now.
		\rSee https://github.com/rust-windowing/softbuffer/issues/215\n"
        );
    }

    let win_size = dpi::PhysicalSize::<u32>::new(size.0, size.1);

    let icon = {
        let (w, h, v) = read_icon();

        winit::window::Icon::from_rgba(v, w, h).expect("Failed to create window icon.")
    };

    let window_attributes = winit::window::Window::default_attributes()
        .with_title("cvis")
        .with_inner_size(win_size)
        .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
        .with_transparent(false)
        .with_decorations(true)
        .with_resizable(prog.is_resizable())
        .with_window_icon(Some(icon));

    let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

    let inner_size = window.clone().inner_size();

    let mut commands: Arc<RwLock<Command>> = Arc::new(RwLock::new(Command::Blank));

    let mut state = WindowState {
        window: window.clone(),
        thread_main_running: Arc::new(AtomicBool::new(true)),
        commands: commands.clone(),
    };

    let thread_main_running = state.thread_main_running.clone();
    let thread_main_running_prog = state.thread_main_running.clone();

    let window = window.clone();
    let thread_main_running_draw = state.thread_main_running.clone();
    let thread_main_running_report = state.thread_main_running.clone();

    let window_updates = window.clone();
    let commands_updates = commands.clone();
    let thread_updates = thread::spawn(move || {
        if lock_fps {
            return;
        }

        loop {
            thread::sleep(Duration::from_millis(621));

            if let Some(monitor) = window_updates.current_monitor() {
                if let Some(milli_hz) = monitor.refresh_rate_millihertz() {
                    let Ok(mut cmd) = commands_updates.try_write() else {
                        continue;
                    };

                    eprintln!("Detected rate to be {}hz", milli_hz as f64 / 1000.0);
                    eprintln!(
                        "Note: Coffeevis relies on Winit to detect rates and it can be wrong."
                    );
                    eprintln!("Refresh rate changed.\n");
                    *cmd = Command::FPSFrac(milli_hz);

                    return;
                }
            }
        }
    });

    const REPORT_RATE: usize = 4;
    let fps_report = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let average_render_time = Arc::new(RwLock::new(Duration::from_millis(0)));
    let average_render_time_draw = average_render_time.clone();
    let fps_report_draw = fps_report.clone();
    let thread_report = thread::spawn(move || {
        if !report_fps {
            return;
        }

        eprintln!("Reporting FPS at {}Hz.", REPORT_RATE);

        const DURATION_ZERO: Duration = Duration::from_millis(0);

        while thread_main_running_report.load(Relaxed) {
            thread::sleep(Duration::from_millis(1000 / REPORT_RATE as u64));

            let frames = fps_report.swap(0, Relaxed);
            let frames = frames * REPORT_RATE;

            eprint!("FPS: {:3.}, ", frames);

            if let Ok(mut time) = average_render_time.try_write() {
                let average_time = if frames > 0 {
                    *time / frames as u32
                } else {
                    DURATION_ZERO
                };
                eprint!("Render time: {:0>10?}. ", average_time);
                eprint!("{:*<1$}", "", average_time.as_micros().min(100) as usize);
                *time = DURATION_ZERO;
            }

            eprintln!();
        }
    });

    // This is the main drawing thread;
    let thread_draw = thread::spawn(move || {
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

        let window = window.clone();

        while thread_main_running_draw.load(Relaxed) {
            let no_sample = crate::audio::get_no_sample();
            let mut draw = false;

            if let Ok(mut cmd) = commands.try_write() {
                draw |= prog.eval_command(&cmd);
                *cmd = Command::Blank;
            }

            prog.update_vis();

            if no_sample >= crate::data::STOP_RENDERING && !draw {
                thread::sleep(Duration::from_millis(333));
                continue;
            }

            let render_begin = std::time::Instant::now();

            let new_size = window.inner_size();

            if size.width != new_size.width || size.height != new_size.height {
                size = new_size;

                let mut size_down = size;

                size_down.width /= prog.scale() as u32;
                size_down.height /= prog.scale() as u32;

                surface
                    .resize(
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    )
                    .unwrap();

                prog.update_size((size_down.width as u16, size_down.height as u16));

                if let Ok(mut buffer) = surface.buffer_mut() {
                    buffer.fill(0x0);
                }

                thread::sleep(Duration::from_millis(20));
            }

            if let Ok(mut buffer) = surface.buffer_mut() {
                prog.force_render();

                let slice = prog.pix.as_slice();

                let len = buffer.len().min(slice.len());

                prog.pix.scale_to(
                    &mut buffer,
                    prog.scale() as usize,
                    Some(size.width as usize),
                );

                window.pre_present_notify();
                let _ = buffer.present();
            }

            let sleep = prog.DURATIONS[(no_sample >> 6) as usize];

            if report_fps {
                let _ = fps_report_draw.fetch_add(1, Relaxed);

                let render_end = std::time::Instant::now();

                if let Ok(mut time) = average_render_time_draw.try_write() {
                    *time += render_end - render_begin;
                }
            }

            if prog.REFRESH_RATE_MODE != crate::RefreshRateMode::Unlimited {
                thread::sleep(sleep);
            }
        }
    });

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let _ = event_loop.run_app(&mut state);
    let _ = thread_updates.join();
    let _ = thread_draw.join();

    Ok(())
}
