#[cfg(all(not(feature = "window_only"), target_os = "linux"))]
pub mod console_mode;

#[cfg(not(feature = "console_only"))]
pub mod windowed_mode;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mode {
    Win,
    ConAscii,
    ConBlock,
    ConBrail,
}

impl Mode {
    pub fn default() -> Mode {
        #[cfg(not(feature = "console_only"))]
        return Mode::Win;

        #[allow(unreachable_code)]
        return Mode::ConAscii;
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            Mode::ConBrail => "braille",
            Mode::ConAscii => "ascii",
            Mode::ConBlock => "block",
            _ => unreachable!(),
        }
    }

    pub fn next(self) -> Self {
        match self {
            Mode::ConAscii => Mode::ConBlock,
            Mode::ConBlock => Mode::ConBrail,
            Mode::ConBrail => Mode::ConAscii,
            _ => self,
        }
    }

    pub fn is_con(&self) -> bool {
        matches!(self, Mode::ConAscii | Mode::ConBlock | Mode::ConBrail)
    }
}
