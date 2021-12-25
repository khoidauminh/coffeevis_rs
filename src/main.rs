mod constants;
use crate::constants::*;

// Audio lib
mod audio_input;
use cpal;
use cpal::traits::{StreamTrait};
use audio_input::input_stream::get_source;

mod math;

mod graphics;
use graphics::{graphical_fn, visualizers::{oscilloscope, spectrum, lazer, vol_sweeper, shaky_coffee}};

mod assets;

use minifb::{WindowOptions, Window, Key, KeyRepeat};

static mut buf : Vec<(f32, f32)> = Vec::new();

use std::env;

static mut i : usize = 0;
//static mut visualizer : (dyn Fn(Vec<u32>, Vec<(f32, f32)>)) = &oscilloscope::draw_vectorscope;

fn main() {

    let mut stream;

    unsafe {
        stream = get_source(read_samples::<i16>);
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

        let mut pix_buf = vec![0u32 ; WIN_W*WIN_H];

        graphics::visualizers::shaky_coffee::prepare_img_default();
        spectrum::prepare_index();
        //crate::constants::prepare_table();

        while win.is_open() && !win.is_key_down(Key::Escape) {

            if (VIS_IDX != SWITCH) {
                pix_buf = vec![0u32; pix_buf.len()];
                SWITCH = VIS_IDX;
            }

            match VIS_IDX {
                2 => vol_sweeper::draw_vol_sweeper(&mut pix_buf, buf.clone()),
                3 => spectrum::draw_spectrum_pow2_std(&mut pix_buf, buf.clone()),
                4 => oscilloscope::draw_oscilloscope(&mut pix_buf, buf.clone()),
                5 => lazer::draw_lazer(&mut pix_buf, buf.clone()),
                1 => shaky_coffee::draw_shaky(&mut pix_buf, buf.clone()),
                _ => oscilloscope::draw_vectorscope(&mut pix_buf, buf.clone()),
            }

            win.update_with_buffer(&pix_buf, WIN_W, WIN_H);

            control_key_events(&win);

            usleep(FPS);
        }
        stream.pause();
    }
}

fn read_samples<T: cpal::Sample>(data : &[T], _ : &cpal::InputCallbackInfo) {
    unsafe {
        buf = data.iter().map(|x| (x.to_f32(), 0.0f32)).collect();
        usleep(FPS_ITVL << 1);
    }
}

fn change_visualizer() {

}

unsafe fn control_key_events(win : &minifb::Window) {

    if win.is_key_pressed(Key::Space, KeyRepeat::Yes) {
        VIS_IDX = (VIS_IDX+1)%VISES;
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

    if win.is_key_pressed(Key::Slash, KeyRepeat::Yes) {
        VOL_SCL = 0.8;
        SMOOTHING = 0.65;
        WAV_WIN = 30;
    }
}
