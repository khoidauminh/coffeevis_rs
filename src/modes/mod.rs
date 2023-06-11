pub mod console_mode;
pub mod windowed_mode;

#[derive(Debug, PartialEq)]
pub enum Mode {
	Win,
	
//	#[cfg(feature = "winit")]
//	Winit,
	
	ConAlpha,
	ConBlock,
	ConBrail,
}

impl Mode {
	pub fn next_con(&mut self) {
		*self = match self {
			Mode::ConAlpha => Mode::ConBlock,
			Mode::ConBlock => Mode::ConBrail,
			Mode::ConBrail => Mode::ConAlpha,
			Mode::Win 	   => Mode::Win,
			
//			#[cfg(feature = "winit")]
//			Mode::Winit    => Mode::Winit,
		}
	}
}
/*
pub fn run(mut conf: &str) {
	let mut prog = crate::data::Program::new().from_conf_str(conf);
	
	#[cfg(feature = "winit")]
	if prog.mode == Mode::Winit
	{
		windowed_mode::winit_mode::win_main_winit(prog);
		return
	}
	
	match &prog.mode {
		&Mode::Win => windowed_mode::win_main(prog).unwrap(),
		
		&_ 		   => console_mode::con_main(prog).unwrap(),
	}
}
*/
