#![allow(non_snake_case)]
#![allow(dead_code)]

use std::env;

use scheduler::set_self_priority;

mod audio;
mod data;
mod graphics;
mod math;
mod modes;
mod visualizers;

use data::*;

use modes::{windowed_mode::*, Mode};

// Audio lib
use audio::get_source;
use cpal::traits::StreamTrait;
use visualizers::VisFunc;

fn main() {
    let _ = set_self_priority(scheduler::Which::Process, -11);

    let args = env::args().collect::<Vec<String>>();

    let prog = Program::new().eval_args(&mut args.iter());

    prog.print_startup_info();

    let stream = get_source();
    stream.play().unwrap();

    match prog.mode() {
        #[cfg(feature = "minifb")]
        Mode::WinLegacy => minifb_main(prog).unwrap(),

        Mode::Win => winit_main(prog).unwrap(),

        _ => {
            #[cfg(feature = "terminal")]
            modes::console_mode::con_main(prog).unwrap();
        }
    }

    stream.pause().unwrap();
}
