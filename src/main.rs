#![allow(warnings)]
// #![forbid(unsafe_code)]

use std::env;

use cpal;
use minifb;

mod data;
mod audio;
mod math;
mod graphics;
mod modes;
mod visualizers;
mod controls;
mod misc;

use data::*;

use modes::{Mode, windowed_mode::*,console_mode::con_main};

// Audio lib
use audio::{get_source, read_samples};
use cpal::traits::StreamTrait;
use visualizers::VisFunc;
use minifb::{Key, KeyRepeat, Window, WindowOptions};

use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};

type WriteLock<T> = std::sync::RwLockWriteGuard<'static, T>;

fn main() {
    let mut stream;
    let mut args = env::args().collect::<Vec<String>>();

    stream = get_source();
    stream.play().unwrap();

	let conf = "";

	std::env::set_var("LC_CTYPE", "en_US.utf8");
	
	let mut prog = Program::new().eval_args(&mut args.iter());
	match &prog.mode
	{
		&Mode::WinLegacy    => win_legacy_main(prog).unwrap(),
		&Mode::Win          => win_main_winit(prog).unwrap(),
		&_                  => con_main(prog).unwrap()
	}

    stream.pause();
}

	/*if "--config" == args.get(1).unwrap_or(&"".to_string())
	{
		let config_string: &str = args.get(2).unwrap();
		println!("Found config argument: {}", config_string);

		run(&config_string)
		// println!("{:?}", prog.mode);
	} else {
		let mut args_iter = args.iter().skip(1);

		let mut prog = Program::new().from_conf_file(conf);
		match args_iter.next().unwrap_or(&String::new()).as_str() {
			"--win" => win_main(prog.as_win()).unwrap(),
			"--con" => con_main(prog.as_con()).unwrap(),

			#[cfg(feature = "winit")]
			"--winit" => {

				// std::env::set_var("WGPU_BACKEND", "gl");
				std::env::set_var("GDK_BACKEND", "x11");

				crate::modes::windowed_mode::winit_mode::win_main_winit(prog.as_winit()).unwrap()
			},

			"--con-ascii"   => con_main(prog.as_con_force(Mode::ConAlpha)).unwrap(),
			"--con-block"   => con_main(prog.as_con_force(Mode::ConBlock)).unwrap(),
			"--con-braille" => con_main(prog.as_con_force(Mode::ConBrail)).unwrap(),

			_       	    => run(conf),
		}
	}*/
