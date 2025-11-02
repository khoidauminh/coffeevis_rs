pub mod console_mode;
pub mod windowed_mode;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mode {
    Win,
    ConAscii,
    ConBlock,
    ConBrail,
}

impl Mode {
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
