use minifb::{self};

use std::{
	thread,
	sync::{Arc, atomic::{Ordering::Relaxed, AtomicBool}}
};

use crate::{
	data::*,
	controls,
};

//~ use fps_clock;

use winit::{
	event::{
		Event,
		WindowEvent::{self},
		DeviceEvent::{self},
		RawKeyEvent
	},
	keyboard::{PhysicalKey::Code, KeyCode},
	event_loop::{EventLoop},
	window::{WindowBuilder},
	dpi::{LogicalSize}
};

use std::num::NonZeroU32;

const MOTION_BLUR_INDEX: u8 = 2;

pub fn win_legacy_main(mut prog: Program) -> Result<(), minifb::Error> {

    let mut win = controls::init_window(&prog)?;
    win.topmost(true);

    let scale = prog.SCALE as usize;

    while win.is_open() && !win.is_key_down(minifb::Key::Q) {
        let s = win.get_size();

         let s = (s.0 / scale, s.1 / scale);

        if s.0 != prog.pix.width() || s.1 != prog.pix.height() {
            //let s = (s.0 / WIN_SCALE, s.1 / WIN_SCALE);
            prog.update_size_win(s);
        }

        controls::control_key_events_win(&mut win, &mut prog);

        prog.update_timer();

        prog.update_state();

        if !prog.render_trigger() {
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

pub fn win_main_winit(mut prog: Program) -> Result<(), &'static str> {

	// let mut prog = Program::new().from_conf_file(conf).as_win();

	pub fn to_u8_vec(buf: &[u32]) -> Vec<u8> {
		buf.iter().flat_map(|x| { // buf stores pixel samples as u32 of [a, r, g, b]
			let mut p = x.to_be_bytes();
			p.rotate_left(1);
			p
		}).collect::<Vec<u8>>()
	}

	let mut size = (
		prog.pix.width()  as u32,
		prog.pix.height() as u32
	);

	if prog.WAYLAND {
		size.0 *= prog.SCALE as u32;
		size.1 *= prog.SCALE as u32;
	}

	std::env::set_var("WINIT_X11_SCALE_FACTOR", prog.SCALE.to_string());

	let event_loop = EventLoop::new().unwrap();

	let window = Arc::new(
			WindowBuilder::new()
			.with_title("kvis")
			.with_inner_size(LogicalSize::<u32>::new(size.0, size.1))
			.with_window_level(winit::window::WindowLevel::AlwaysOnTop)
			.with_transparent(prog.transparency < 255)
			.with_resizable(false)
			.build(&event_loop)
			.expect("Failed to init window")
	);

	let inner_size = window.clone().inner_size();

    let context = unsafe { softbuffer::Context::new(&window.clone()).unwrap() };
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window.clone()).unwrap() };

	surface
	.resize(
		NonZeroU32::new(inner_size.width ).unwrap(),
		NonZeroU32::new(inner_size.height).unwrap()
	)
	.unwrap();

	let thread_main_running  = Arc::new(AtomicBool::new(true));


	#[cfg(not(feature = "benchmark"))]
	{
		use std::time::Duration;

		let thread_main_running = thread_main_running.clone();
		let _active_frame_duration = prog.REFRESH_RATE;
		let _idle_frame_duration = Duration::from_millis(333);

		let durations: [Duration; 4] = [
			prog.REFRESH_RATE,
			prog.REFRESH_RATE*4,
			prog.REFRESH_RATE*16,
			Duration::from_millis(250)
		];

		let window = window.clone();

		thread::spawn(move || {
			while thread_main_running.load(Relaxed) {

				let no_sample = crate::audio::get_no_sample();

				if no_sample < 255 {
					window.request_redraw();
				}

				thread::sleep(durations[(no_sample >> 6) as usize]);
			}
		});
	}

	// let mut clock = fps_clock::FpsClock::new(prog.FPS as u32);

	fn set_exit(b: Arc<AtomicBool>) {
		b.store(false, Relaxed);
	}

	event_loop.run(move |event, elwt| {
		prog.update_vis();

		let perform_draw =
			|
				prog: &mut Program,
				surface: &mut softbuffer::Surface
			|
		{
			let mut buffer = surface.buffer_mut().unwrap();

			prog.force_render();

			prog.pix.scale_to(&mut buffer, prog.SCALE as usize);

			let _ = buffer.present();
		};

		match event {
			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				..
			} => {
				set_exit(thread_main_running.clone());
				elwt.exit()
			},

			Event::WindowEvent {
				event: WindowEvent::RedrawRequested,
				..
			} => {
				perform_draw(&mut prog, &mut surface);

				#[cfg(feature = "benchmark")]
				window.request_redraw();
			},


			//~ Event::AboutToWait => {
				//~ let no_sample = crate::audio::get_no_sample();

				//~ if no_sample < 128 {
					//~ perform_draw(&mut prog, &mut surface);
					//~ window.request_redraw();
					//~ clock.tick();
				//~ } else {
					//~ thread::sleep(Duration::from_millis(250));
				//~ }

			//~ },

			Event::DeviceEvent{event: DeviceEvent::Key(RawKeyEvent{physical_key: Code(code), state }), .. } => {

				if state.is_pressed() {
					return
				}

				match code {
					KeyCode::Escape => {
						set_exit(thread_main_running.clone());
						elwt.exit()
					},

					KeyCode::Space => {
						prog.change_visualizer(true);
						perform_draw(&mut prog, &mut surface);
					},

					KeyCode::KeyB => {
						prog.change_visualizer(false);
						perform_draw(&mut prog, &mut surface);
					},

					KeyCode::Minus 			=>   prog.decrease_vol_scl(),
					KeyCode::Equal 			=>  prog.increase_vol_scl(),

					KeyCode::BracketLeft 	=>  prog.decrease_smoothing(),
					KeyCode::BracketRight 	=>  prog.increase_smoothing(),

					KeyCode::Semicolon 		=>  prog.decrease_smoothing(),
					KeyCode::Quote 			=> prog.increase_smoothing(),

					KeyCode::Backslash 		=> prog.toggle_auto_switch(),

					KeyCode::Slash 			=> prog.reset_parameters(),

					_ => {},
				}

			},

			_ => {},
		}
	}).unwrap();

	Ok(())
}

