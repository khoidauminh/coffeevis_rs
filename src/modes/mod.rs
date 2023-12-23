pub mod console_mode;
pub mod windowed_mode;

#[derive(Debug, PartialEq)]
pub enum Mode {
	Win,
    WinLegacy,

	ConAlpha,
	ConBlock,
	ConBrail,
}

impl Mode {
	/*
	pub fn next_con(&mut self) {
		*self = match self {
			Mode::ConAlpha  => Mode::ConBlock,
			Mode::ConBlock  => Mode::ConBrail,
			Mode::ConBrail  => Mode::ConAlpha,
			Mode::Win 	    => Mode::Win,
			Mode::WinLegacy => Mode::WinLegacy,
		}
	}*/

	pub fn is_con(&self) -> bool {
		match self {
			Mode::ConAlpha |
			Mode::ConBlock |
			Mode::ConBrail => true,

			_ => false,
		}
	}
}
