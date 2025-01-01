#![allow(non_snake_case)]
#![allow(dead_code)]

use std::env;

mod audio;
mod data;
mod graphics;
mod math;
mod modes;
mod visualizers;

use data::*;

// Audio lib
use audio::get_source;
use cpal::traits::StreamTrait;
use reader::eprintln_red;
use visualizers::VisFunc;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    let prog = Program::new().eval_args(&mut args.iter());

    let stream = get_source();
    stream.play().unwrap();

    match prog.mode() {
        #[cfg(not(feature = "console_only"))]
        modes::Mode::Win => modes::windowed_mode::winit_main(prog).unwrap(),

        _ => {
            #[allow(unused_mut, unused_assignments)]
            let mut terminal = false;

            #[cfg(not(feature = "window_only"))]
            {
                terminal = true;
                modes::console_mode::con_main(prog).unwrap();
            }

            if !terminal {
                eprintln_red!("Unable to determine display mode!");
            }
        }
    }

    stream.pause().unwrap();
}
