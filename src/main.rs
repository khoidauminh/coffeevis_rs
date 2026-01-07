#![forbid(unsafe_code)]

use std::env;
mod audio;
mod data;
mod graphics;
mod math;
mod modes;
mod visualizers;

use audio::get_source;
use cpal::traits::StreamTrait;

use crate::audio::AudioBuffer;
use crate::data::Program;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    let prog = data::Program::new().eval_args(&mut args.iter());

    let stream = get_source();
    stream.play().unwrap();

    match prog.mode() {
        modes::Mode::Win => modes::windowed_mode::winit_main(prog),
        _ => modes::console_mode::con_main(prog).unwrap(),
    }

    stream.pause().unwrap();

    crate::data::log::info!("Bye!")
}
