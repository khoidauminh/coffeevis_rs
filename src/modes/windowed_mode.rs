use minifb::{self, Scale};

use std::{  
	sync::{Arc, RwLock, atomic::{Ordering, AtomicU64}},
	time::{Instant, Duration}
};

use crate::{
	audio::get_buf,
	data::*,
	controls,
	graphics,
	visualizers::VisFunc,
};

const MOTION_BLUR_INDEX: u8 = 2;

pub fn win_main(mut prog: Program) -> Result<(), minifb::Error> {

	// let mut prog = Program::new().from_conf_file(conf).as_win();
	/*
	let scale = match prog.SCALE {
		2 => Scale::X2,
		4 => Scale::X4,
		8 => Scale::X8,
		_ => Scale::X1,
	};*/

    let mut win = controls::init_window(&prog)?;
   // let mut visualizer: VisFunc = prog.vis_array[0];
    win.topmost(true);

    let scale = prog.SCALE as usize;

    while win.is_open() && !win.is_key_down(minifb::Key::Q) {
        let s = win.get_size();

         let s = (s.0 / scale, s.1 / scale);

        if s.0 != prog.pix.width || s.1 != prog.pix.height {
            //let s = (s.0 / WIN_SCALE, s.1 / WIN_SCALE);
            prog.update_size_win(s);
        }

        //println!("{:?}", win.get_position());

        // win.set_position(100, 100);

        controls::control_key_events_win(&mut win, &mut prog);

        prog.update_timer();

        prog.update_state();

        /*use crate::data::{State, IDLE_REFRESH_RATE};
        let state = prog.set_state();
        match state {
            State::Waiting => {
                win.limit_update_rate(Some(IDLE_REFRESH_RATE));
                println!("{:?}", );
            },
            State::Waken   => {
                win.limit_update_rate(Some(prog.REFRESH_RATE));
                println!("I WOKE");
            }
            _ => {},
        }*/
        

        if !prog.render_trigger() {
            win.update();
            continue;
        }

        prog.force_render();

        prog.print_err_win();
        
        let winw = prog.pix.width*scale;
        let winh = prog.pix.height*scale;

        if (scale == 1) {
	        win.update_with_buffer(prog.pix.as_slice(), winw, winh);
	        continue;
	    }

	    let mut buffer = vec![0u32; winw*winh];

        //
        /*
	    for y in 0..winh {
            let xbase = y*winw;

            let xbase_scaled = y / scale * prog.pix.width;
            // println!("{} {}", xbase_scaled, xbase / scale / winw * prog.pix.width);

	        for x in 0..winw {
	            buffer[xbase + x] = prog.pix.pixel(xbase_scaled + x/scale);
	        }
	    }*/

	    /*let jump = winw - scale;
        let scale2 = scale.pow(2);
        let jumprow = winw*scale2;*/


        /*for yibase in (0..prog.pix.sizel()).step_by(prog.pix.width) {
            let ybase = yibase * scale2;

            let bound = ybase + jumprow + scale;

            for xi in 0..prog.pix.width {
                let pixel = prog.pix.pixel(yibase + xi);

                for y in (ybase..bound).step_by(jump) {
                    for i in y..y+scale {
                        buffer[i] = pixel;
                    }
                }
            }
        }*/

        prog.pix.scale_to(&mut buffer, scale);

	    win.update_with_buffer(&buffer, winw, winh);
        /*

	    use std::iter::repeat;
	    let buffer =
            prog.pix
            .as_slice()
            .chunks_exact(prog.pix.width)
            .flat_map(|line|
                line.iter().map(|pixel| repeat(*pixel).take(scale)).cycle().take(winw)
            )
            .flatten()
            .collect::<Vec<u32>>();
	    ;
	    win.update_with_buffer(&buffer, winw, winh);
        */
    }

    Ok(())
}

#[cfg(feature = "winit")]
pub mod winit_mode {
	use crate::{
		audio::get_buf,
		data::*,
		controls,
		graphics,
		visualizers::VisFunc
	};
	
	use fps_clock;

	use winit::{
		event::{
			self,
			Event, 
			WindowEvent, 
			KeyboardInput, 
			DeviceEvent::Key, 
			VirtualKeyCode,
			ElementState::Released
		},
		event_loop::{EventLoop, ControlFlow},
		window::{Window, WindowBuilder},
		dpi::{PhysicalSize, LogicalSize}
	};
	use std::time::Instant;
	
	use std::num::NonZeroU32;

	pub fn win_main_winit(mut prog: Program) -> Result<(), &'static str> {

		// let mut prog = Program::new().from_conf_file(conf).as_win();

		pub fn to_u8_vec(buf: &[u32]) -> Vec<u8> {
			buf.iter().map(|x| { // buf stores pixel samples as u32 of [a, r, g, b]
				let mut p = x.to_be_bytes(); 
				p.rotate_left(1);
				p
			}).flatten().collect::<Vec<u8>>()
		}
		
		let size = (
			prog.pix.width as u32, //*prog.SCALE as u32, 
			prog.pix.height as u32 //*prog.SCALE as u32
		);

		let mut icon = 
		    winit::window::Icon::from_rgba(
		        to_u8_vec(prog.IMG.as_slice()),
		        prog.IMG.width as u32, 
		        prog.IMG.height as u32
		    ).unwrap();

		std::env::set_var("WINIT_X11_SCALE_FACTOR", prog.SCALE.to_string());

		let mut event_loop = EventLoop::new();
		let mut window = WindowBuilder::new()
			//~ .with_vsync(true)
			.with_title("kvis")
			.with_inner_size(LogicalSize::<u32>::new(size.0, size.1))
			.with_window_level(winit::window::WindowLevel::AlwaysOnTop)
			.with_transparent(false)
			.with_resizable(false)
			.with_window_icon(Some(icon))
			.build(&event_loop)
			.expect("Failed to init window");

		let inner_size = window.inner_size();
		
		let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
		let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();
		
		surface
		.resize(
			NonZeroU32::new(inner_size.width).unwrap(),
			NonZeroU32::new(inner_size.height).unwrap()
		)
		.unwrap();
		
		/*
		let surface_texture = pixels::SurfaceTexture::new(inner_size.width, inner_size.height, &window);
		let mut pixels_context = 
			pixels::PixelsBuilder::new(
				prog.pix.width as u32, 
				prog.pix.height as u32, 
				surface_texture
			)
			.request_adapter_options(RequestAdapterOptions {
				power_preference: PowerPreference::LowPower,
				force_fallback_adapter: false,
				compatible_surface: None,
			})
			.build()
			.expect("Failed to create Pixels context");
		*/

		let mut clock = fps_clock::FpsClock::new(prog.FPS as u32);
		
		event_loop.run(move |event, _, control_flow| {

			*control_flow = ControlFlow::Poll;

			prog.update_vis();

			match event {
				Event::WindowEvent {
					event: WindowEvent::CloseRequested,
					..
				} => control_flow.set_exit(),

				Event::DeviceEvent{ event: Key(KeyboardInput{virtual_keycode: Some(code), state: kstate, .. }), ..} => {
					//use VirtualKeyCode::*;
					if kstate == Released {
						//println!("{:?}", code.clone());

						match code {
							VirtualKeyCode::Q => control_flow.set_exit(),

							VirtualKeyCode::Space => {
								prog.change_visualizer(true);
							},

							//~ VirtualKeyCode::Key1 =>  change_fps(&mut prog, 10, true),
							//~ VirtualKeyCode::Key2 =>  change_fps(&mut prog, 20, true),
							//~ VirtualKeyCode::Key3 =>  change_fps(&mut prog, 30, true),
							//~ VirtualKeyCode::Key4 =>  change_fps(&mut prog, 40, true),
							//~ VirtualKeyCode::Key5 =>  change_fps(&mut prog, 50, true),
							//~ VirtualKeyCode::Key6 =>  change_fps(&mut prog, 60, true),

							//~ VirtualKeyCode::Key7 =>  change_fps(&mut prog, -5, false),
							//~ VirtualKeyCode::Key8 =>  change_fps(&mut prog,  5, false),

							VirtualKeyCode::Minus =>   prog.VOL_SCL = (prog.VOL_SCL / 1.2).clamp(0.0, 10.0),
							VirtualKeyCode::Equals =>   prog.VOL_SCL = (prog.VOL_SCL * 1.2).clamp(0.0, 10.0),

							VirtualKeyCode::LBracket =>   prog.SMOOTHING = (prog.SMOOTHING - 0.05).clamp(0.0, 0.95),
							VirtualKeyCode::RBracket =>   prog.SMOOTHING = (prog.SMOOTHING + 0.05).clamp(0.0, 0.95),

							VirtualKeyCode::Semicolon =>   prog.WAV_WIN = (prog.WAV_WIN - 3).clamp(3, 50),
							VirtualKeyCode::Apostrophe =>  prog.WAV_WIN = (prog.WAV_WIN + 3).clamp(3, 50),

							VirtualKeyCode::Backslash => prog.AUTO_SWITCH ^= true,

							VirtualKeyCode::Slash => {
								prog.VOL_SCL = DEFAULT_VOL_SCL;
								prog.SMOOTHING = DEFAULT_SMOOTHING;
								prog.WAV_WIN = DEFAULT_WAV_WIN;
							}


							_ => {},
						}
					}
				},

				Event::MainEventsCleared => {
					window.request_redraw();
					clock.tick();
				},

				Event::RedrawRequested(_) => {
					prog.update_timer();

					if prog.render_trigger() {
					
						let mut buffer = surface.buffer_mut().unwrap();
						
						prog.force_render();
						
						prog.pix.scale_to(&mut buffer, prog.SCALE as usize);

						buffer.present().unwrap();
					}
					
				},

				_ => (),
			}
		});

		Ok(())
	}
}
