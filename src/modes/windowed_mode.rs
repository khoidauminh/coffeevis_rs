#![allow(warnings)]

use std::{
	thread,
	sync::{Arc, atomic::{Ordering::Relaxed, AtomicBool, AtomicUsize}}
};

use crate::data::*;

use minifb::{Window, WindowOptions};

pub fn init_window(prog: &Program) -> Result<Window, minifb::Error> {
	std::env::set_var("GDK_BACKEND", "x11");

	let mut win = Window::new(
        "kvis",
        prog.pix.width()*prog.scale() as usize,
        prog.pix.height()*prog.scale() as usize,
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

pub fn control_key_events_win_legacy(
    win: &mut minifb::Window,
    prog: &mut Program,
) {
	use minifb::{Key, KeyRepeat};
	
    let fps_change = false;

    prog.update_vis();
    
    win.get_keys_pressed(KeyRepeat::No).iter().for_each(|key|
        match key {
            Key::Space => prog.change_visualizer(true),

            //~ Key::Key1 =>  { change_fps(prog, 10, true); fps_change = true; },
            //~ Key::Key2 =>  { change_fps(prog, 20, true); fps_change = true; },
            //~ Key::Key3 =>  { change_fps(prog, 30, true); fps_change = true; },
            //~ Key::Key4 =>  { change_fps(prog, 40, true); fps_change = true; },
            //~ Key::Key5 =>  { change_fps(prog, 50, true); fps_change = true; },
            //~ Key::Key6 =>  { change_fps(prog, 60, true); fps_change = true; },

            //~ Key::Key7 =>  { change_fps(prog, -5, false); fps_change = true; },
            //~ Key::Key8 =>  { change_fps(prog,  5, false); fps_change = true; },

            Key::Minus =>   prog.VOL_SCL = (prog.VOL_SCL / 1.2).clamp(0.0, 10.0),
            Key::Equal =>   prog.VOL_SCL = (prog.VOL_SCL * 1.2).clamp(0.0, 10.0),

            Key::LeftBracket =>   prog.SMOOTHING = (prog.SMOOTHING - 0.05).clamp(0.0, 0.95),
            Key::RightBracket =>   prog.SMOOTHING = (prog.SMOOTHING + 0.05).clamp(0.0, 0.95),

            Key::Semicolon =>   prog.WAV_WIN = (prog.WAV_WIN - 3).clamp(3, 50),
            Key::Apostrophe =>  prog.WAV_WIN = (prog.WAV_WIN + 3).clamp(3, 50),

            Key::Backslash =>  prog.toggle_auto_switch(),

            Key::Slash => {
                prog.VOL_SCL = DEFAULT_VOL_SCL;
                prog.SMOOTHING = DEFAULT_SMOOTHING;
                prog.WAV_WIN = DEFAULT_WAV_WIN;
                // change_fps(prog, 144, true);
            }

            _ => {},
        }
    );

    if fps_change {
        win.limit_update_rate(Some(prog.REFRESH_RATE));
    }
}

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

        let winw = prog.pix.width()*scale;
        let winh = prog.pix.height()*scale;

        if scale == 1 {
	        let _ = win.update_with_buffer(prog.pix.as_slice(), winw, winh);
	        continue;
	    }

	    let mut buffer = vec![0u32; winw*winh];


        prog.pix.scale_to(&mut buffer, scale);

	    let _ = win.update_with_buffer(&buffer, winw, winh);

    }

    Ok(())
}


use winit::{
	event::{
        ElementState,
		Event,
		WindowEvent,
		DeviceEvent,
		DeviceId
	},
	
	application::ApplicationHandler,
	
	platform::modifier_supplement::KeyEventExtModifierSupplement,
	keyboard::{Key, NamedKey::{Escape, Space}},
	
	event_loop::{EventLoop, ActiveEventLoop},
	
	window::WindowId,
	
	dpi::LogicalSize
};

use std::num::NonZeroU32;
use std::time::Duration;
use std::sync::RwLock;

use crate::data::Command;

fn set_exit(b: Arc<AtomicBool>) {
	b.store(false, Relaxed);
}

struct WindowState { 
	pub thread_main_running: Arc<AtomicBool>,
	pub window: Arc<winit::window::Window>,
	pub commands: Arc<RwLock< Vec<Command> >>,
}

impl ApplicationHandler for WindowState {

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {    	
    	
    	match event {
			WindowEvent::CloseRequested => {
			    set_exit(self.thread_main_running.clone());
			    event_loop.exit();
		    },
    	
			WindowEvent::RedrawRequested => {
				
				// self.window.request_redraw();
			}
			
			 WindowEvent::KeyboardInput {event, ..} => {
				
				match self.commands.try_write() {

					Ok(mut cmds) if event.state == ElementState::Pressed && !event.repeat => {
						
						let cmd = match event.key_without_modifiers().as_ref() {
							
							Key::Named(Escape) => {
								set_exit(self.thread_main_running.clone());
								event_loop.exit();
								
								Command::Blank
							},

							Key::Named(Space) => {
								Command::VisualizerNext
							},
							
							Key::Character("b") => {
								Command::VisualizerPrev
							},
							
							Key::Character("n") => {
								Command::SwitchVisList
							}
							
							Key::Character("-")		=> Command::VolDown,
							Key::Character("=")		=> Command::VolUp,

							Key::Character("[") 	=> Command::SmoothDown,
							Key::Character("]") 	=> Command::SmoothUp,

							Key::Character(";")		=> Command::WavDown,
							Key::Character("\'")	=> Command::WavUp,

							Key::Character("\\")	=> Command::AutoSwitch,

							Key::Character("/") 	=> Command::Reset,
	
							_ => Command::Blank ,
						};
						
						cmds.push(cmd);
					},
					
					Ok(_) => {},
					Err(_) => {}
				}
			},
			
			_ => {}
	    }
    }
    
    //fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {}
    
    //fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {}
}

pub fn winit_main(mut prog: Program) -> Result<(), &'static str> {
	
	let event_loop = EventLoop::new().unwrap();
	
	let mut size = (
		prog.pix.width()  as u32,
		prog.pix.height() as u32
	);
	
	if prog.is_wayland() {
		size.0 *= prog.scale() as u32;
		size.1 *= prog.scale() as u32;
	}

	std::env::set_var("WINIT_X11_SCALE_FACTOR", prog.scale().to_string());

	if prog.transparency < 255 {
		eprintln!(
		"WARNING: transparency doesn't work for now.
		\rSee https://github.com/rust-windowing/softbuffer/issues/215\n"
		);
	}

	let window_attributes = 
		winit::window::Window::default_attributes()
			.with_title("cvis")
			.with_inner_size(LogicalSize::<u32>::new(size.0, size.1))
			.with_window_level(winit::window::WindowLevel::AlwaysOnTop)
			.with_transparent(false)
			.with_resizable(false)
	;
	
	let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

    let inner_size 	= window.clone().inner_size();
	
	let durations = [
		prog.REFRESH_RATE,
		prog.REFRESH_RATE*4,
		prog.REFRESH_RATE*16,
		Duration::from_millis(250)
	];
	
	let mut commands: Arc<RwLock<Vec<Command>>> = Arc::new(RwLock::new(Vec::new()));

	let mut state = WindowState { 
		window: window.clone(), 
		thread_main_running: Arc::new(AtomicBool::new(true)),
		commands: commands.clone(),
	};
	
	let thread_main_running = state.thread_main_running.clone();
	
	let window = window.clone();
	
	let thread = thread::spawn(move || {
				
		let context 	= softbuffer::Context::new(window.clone()).unwrap();
		let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

		surface
		.resize(
			NonZeroU32::new(inner_size.width ).unwrap(),
			NonZeroU32::new(inner_size.height).unwrap()
		)
		.unwrap();

		let _active_frame_duration = prog.REFRESH_RATE;
		let _idle_frame_duration = Duration::from_millis(333);

		let durations: [Duration; 4] = [
			prog.REFRESH_RATE,
			prog.REFRESH_RATE*4,
			prog.REFRESH_RATE*16,
			Duration::from_millis(250)
		];

		let window = window.clone();
		
		let commands = commands.clone();
		
		let counter: usize = 0;
	
		while thread_main_running.load(Relaxed) {

			let no_sample = crate::audio::get_no_sample();			
			let mut redraw = false;
			
			if let Ok(mut cmd) = commands.try_write() {
				redraw = prog.eval_commands(&mut cmd);
			}

			if no_sample < 192 || redraw {
				
				let mut buffer = surface.buffer_mut().unwrap();
				prog.force_render();
				prog.pix.scale_to(&mut buffer, prog.scale() as usize);
				let _ = buffer.present();
				
				prog.update_vis();
				
				window.request_redraw();
			}

			thread::sleep(durations[(no_sample >> 6) as usize]);
		}
	});

	event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
	
	let _ = event_loop.run_app(&mut state);
	
	thread.join().unwrap();
	
	Ok(())
}
