use std::{
	io::{Read, BufReader, Write, Cursor, prelude::*},
	fs::File,
	path::PathBuf,
	env,
};

use crate::{
	data::Program,
	graphics::{self, Image},
	modes::Mode
};
/*
pub fn check_path() -> Option<PathBuf> {

	let PATHS: [PathBuf; 2] = [
		[dirs::home_dir().unwrap_or(PathBuf::new()), PathBuf::from(".coffeevis.conf")].iter().collect(),
		[dirs::config_dir().unwrap_or(PathBuf::new()), PathBuf::from(".coffeevis.conf")].iter().collect(),
	];

	for path in PATHS.iter() {
		if path.exists() {
			return Some(path.clone())
		}
	}
	return None
}
*/
/*

use image::{
	ImageFormat,
	io::Reader,
	GenericImageView
};
pub fn prepare_image(file: &[u8]) -> Image {

	let img =
		Reader::new(Cursor::new(crate::data::IMAGE))
		.with_guessed_format()
		.unwrap()
		.decode().unwrap();

	let (w, h) = (img.width() as usize, img.height() as usize);

	Image::from_buffer(
		img.pixels().map(|pixel| {
			let mut pixel_u8 = pixel.2.0;
			pixel_u8.rotate_right(1);
			u32::from_be_bytes(pixel_u8)
		}).collect::<Vec<_>>(),
		w,
		h
	)
}*/

impl Program {
	pub fn write_err<E: std::error::Error>(&mut self, l: usize, err: E) {
		self.msg = Err(format!("{} {}: {}", crate::data::ERR_MSG, l, err));
	}

	pub fn write_err_msg(&mut self, l: usize, err: &str) {
		self.msg = Err(format!("{} {}: {}", crate::data::ERR_MSG, l, err));
	}

	pub fn eval_args(mut self, args: &mut dyn std::iter::Iterator<Item = &String>) -> Self
	{
		use crate::{
			modes::Mode::*,
			data::*
		};

		let mut size = (DEFAULT_SIZE_WIN, DEFAULT_SIZE_WIN);

		args.next();
		
		loop {
			let mut arg =  "";

			if let Some(st) = args.next() {
				arg = st.as_str();
				println!("{}", arg);
			} else {
				break
			}

			match arg
			{
				"--win-legacy" 	=> self.mode = WinLegacy,

				"--win" 	=> self.mode = Win,

				"--braille" => (self.mode, self.flusher) = (ConBrail, Program::print_brail),
				"--ascii" 	=> (self.mode, self.flusher) = (ConAlpha, Program::print_alpha),
				"--block" 	=> (self.mode, self.flusher) = (ConBlock, Program::print_block),

				"--no-auto-switch" =>
					self.AUTO_SWITCH = false,

				"--size" => {
					let s = args.next()
					.expect("Argument error: Expected value for size.")
					.split("x")
					.map(|x| x.parse::<u16>().expect("Argument error: Invalid value"))
					.collect::<Vec<_>>();

					size = (s[0], s[1]);
				},

				"--scale" =>
				{
				    self.SCALE =
						args.next()
						.expect("Argument error: Expected u8 value for scale")
						.parse::<u8>()
						.expect("Argument error: Invalid value");

					if self.SCALE == 0 {
					    panic!("Argument error: scale is 0");
					}
				},

				"--fps" =>
				{
				    let new_fps =
				        args.next()
				        .expect("Argument error: Expected value for fps")
				        .parse::<u64>()
				        .expect("Argument error: Invalid value.");

				    if new_fps > 200 {
				        panic!("Fps value too high (must be lower than 200)");
				    }

				    self.update_fps(new_fps);
				},

				"--resizeable" =>
				{
				    todo!();
				    self.RESIZE = true;
				},
				
				"--transparent" => 
				{
					match args.next() {
						Some(string) => self.transparency = string.parse::<u8>().expect("Invalid value for transparency"),
						None => self.transparency = 0
				    }
				}

				"--max-con-size" => {
					let s = args.next()
					.expect("Argument error: Expected value for size")
					.split("x")
					.map(|x| x.parse::<u16>().expect("Argument error: Invalid value"))
					.collect::<Vec<_>>();

					(self.CON_MAX_W, self.CON_MAX_H) = (s[0], s[1]);
				}

				"--force-wayland" => {
                    self.WAYLAND = true;
                    std::env::set_var("LANG", "C");
                    self.SCALE = 1;
				},

				&_ => eprintln!("Argument error: Unknown option {}", arg),
			}
		}

		self.update_size(size);

		/*
		match self.mode {
			Win => (self.WIN_W, self.WIN_H) = (size[0], size[1]),

			#[cfg(feature = "winit")]
			Winit => (self.WIN_W, self.WIN_H) = (size[0], size[1]),

			_ => (self.CON_W, self.CON_H) = crate::modes::console_mode::rescale((size[0] as u16, size[1] as u16), self),
		}*/
		
		if !self.WAYLAND {
			std::env::set_var("WAYLAND_DISPLAY", "");
		}

		self
	}

	pub fn print_err_con(&mut self) {
		use crossterm::{
			queue,
			Command,
			QueueableCommand,
			cursor::{self, Hide, Show},
			style::{Stylize, Colors, SetColors, Print, Color, Attribute, SetAttribute},
		};
		use crate::modes::Mode;

		let mut stdout = std::io::stdout();

		match &self.msg {
			Ok(()) => {},

			Err(string) => {
				queue!(
					stdout,
					cursor::MoveTo(0, 0),
						SetColors(Colors::new(
						Color::White,
						Color::Reset
					)),
					Print(string)
				);
				if (self.msg_timeout >> 2) > self.FPS {
					self.msg = Ok(());
					self.clear_con();
					self.msg_timeout = 0;
				} else {
					self.msg_timeout += 1;
				}
			}
		}
	}

	/*pub fn print_err_win(&mut self) {
		match &self.msg {
			Ok(()) => {},
			Err(string)  => {
				println!("Configuration error: {}", string);
				self.msg = Ok(());
			}
		}
	}*/
}
