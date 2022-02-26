
//use core::time::Duration;
#![allow(warnings)]


mod constants;
use crate::constants::*;

// Audio lib
mod audio_input;
use cpal;
use cpal::traits::{StreamTrait};
use audio_input::input_stream::get_source;

mod math;

mod graphics;
use graphics::{graphical_fn, visualizers::{oscilloscope, spectrum, lazer, vol_sweeper, shaky_coffee, ring, bars}};

//mod assets;

use minifb::{WindowOptions, Window, Key, KeyRepeat};

static mut buf : [(f32, f32); SAMPLE_SIZE] = [(0.0f32, 0.0f32); SAMPLE_SIZE];

use std::env;

static mut i : usize = 0;
static mut switch_incremeter: u64 = 0;
//static mut visualizer : (dyn Fn(Vec<u32>, Vec<(f32, f32)>)) = &oscilloscope::draw_vectorscope;

fn main() {
	let coffee_pixart_file = include_bytes!("coffee_pixart_2x.ppm");

    let mut stream;

    unsafe {
        stream = get_source(read_samples::<f32>);
        let status = stream.play();

        let mut win = match Window::new(
            "Coffee Visualizer",
            WIN_W, WIN_H,
            WindowOptions {
                    scale : minifb::Scale::X1,
                    resize : false,
                    topmost : true,
                    borderless : false,
                    ..WindowOptions::default()
            }
        ) {
            Ok(win) => win,
            Err(err) => {
                println!("Unable to create window {}", err);
                return;
            }
        };

        win.limit_update_rate(Some(std::time::Duration::from_micros(FPS_ITVL)));

        let mut pix_buf: [u32; WIN_R] = [0u32 ; WIN_R];

        graphics::visualizers::shaky_coffee::prepare_img(coffee_pixart_file);
        spectrum::prepare_index();
        bars::prepare_index_bar();
        //crate::constants::prepare_table();

        while win.is_open() && !win.is_key_down(Key::Escape) {

            if switch_incremeter == AUTO_SWITCH_ITVL {
                change_visualizer();
                switch_incremeter = 0;
            }

            if (VIS_IDX != SWITCH) {
                pix_buf = [0u32; WIN_R];
                SWITCH = VIS_IDX;
            }

            match VIS_IDX {
                1 => shaky_coffee::draw_shaky(&mut pix_buf, &buf),
                2 => vol_sweeper::draw_vol_sweeper(&mut pix_buf, &buf),
                3 => spectrum::draw_spectrum_pow2_std(&mut pix_buf, &buf),
                4 => oscilloscope::draw_oscilloscope(&mut pix_buf, &buf),
                5 => lazer::draw_lazer(&mut pix_buf, &buf),
                6 => ring::draw_ring(&mut pix_buf, &buf),
                7 => bars::draw_bars(&mut pix_buf, &buf),
                _ => oscilloscope::draw_vectorscope(&mut pix_buf, &buf),
            }

            win.update_with_buffer(&pix_buf, WIN_W, WIN_H);

//			println!("{:?}", buf[0]);

            control_key_events(&win);

            usleep(FPS);

            if AUTO_SWITCH {
                switch_incremeter += 1;
            }
        }
        stream.pause();
    }
}

fn read_samples<T: cpal::Sample>(data : &[T], _ : &cpal::InputCallbackInfo) {
    unsafe {
	//buf = data.iter().map(|x| (x.to_f32(), 0.0f32)).collect();
		for sample in 0..SAMPLE_SIZE {
			buf[sample].0 = data[sample%data.len()].to_f32();
		}
//		println!("{}", data.len());
		usleep(FPS_ITVL);
    }
}

unsafe fn change_visualizer() {
    VIS_IDX = (VIS_IDX+1)%VISES;
}

unsafe fn control_key_events(win : &minifb::Window) {

    if win.is_key_pressed(Key::Space, KeyRepeat::Yes) {
        change_visualizer();
    }

    if win.is_key_pressed(Key::Minus, KeyRepeat::Yes) {
        VOL_SCL = (VOL_SCL/1.2).clamp(0.0, 10.0);
    } else if win.is_key_pressed(Key::Equal, KeyRepeat::Yes) {
        VOL_SCL = (VOL_SCL*1.2).clamp(0.0, 10.0);
    }

    if win.is_key_pressed(Key::LeftBracket, KeyRepeat::Yes) {
        SMOOTHING = (SMOOTHING-0.05).clamp(0.0, 0.95);
    } else if win.is_key_pressed(Key::RightBracket, KeyRepeat::Yes) {
        SMOOTHING = (SMOOTHING+0.05).clamp(0.0, 0.95);
    }

    if win.is_key_pressed(Key::Semicolon, KeyRepeat::Yes) {
        WAV_WIN = (WAV_WIN-3).clamp(3, 50);
    } else if win.is_key_pressed(Key::Apostrophe, KeyRepeat::Yes) {
        WAV_WIN = (WAV_WIN+3).clamp(3, 50);
    }
    
    if win.is_key_pressed(Key::Backslash, KeyRepeat::No) {
        AUTO_SWITCH ^= true;
    } 

    if win.is_key_pressed(Key::Slash, KeyRepeat::Yes) {
        VOL_SCL = 0.8;
        SMOOTHING = 0.65;
        WAV_WIN = 30;
    }
}
