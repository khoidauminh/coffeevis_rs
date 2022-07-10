#![allow(warnings)]

use std::env;

use cpal;
use minifb;

mod graphics;
mod audio_input;
mod math;
mod config;
mod controls;
mod console_mode;

use config::*;

// Audio lib
use audio_input::{get_source, read_samples};
use cpal::traits::StreamTrait;
use graphics::{graphical_fn, visualizers::{oscilloscope::draw_vectorscope, VisFunc}};
use minifb::{Key, KeyRepeat, Window, WindowOptions};

/// Global buffer used for audio input data storage.
static mut buf: [(f32, f32); config::SAMPLE_SIZE] = [(0.0f32, 0.0f32); config::SAMPLE_SIZE];
static IMAGE: &[u8; 449509] = include_bytes!("coffee_pixart_2x.ppm");

fn main() {
    let mut stream;
    let mut args = env::args().collect::<Vec<String>>();

    stream = get_source(read_samples::<f32>);
    stream.play().unwrap();

    let mut err = Ok(());

    match args.pop().unwrap().as_str() {
        "--con" => console_mode::console_main().unwrap(),
        _       => err = crate::graphics::graphics_main(),
    }

    if err.is_err() {
        println!("Cannot initialize window: {:?}", err);
    }

    stream.pause();
}
