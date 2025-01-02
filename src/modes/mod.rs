#[cfg(not(feature = "window_only"))]
pub mod console_mode;

#[cfg(not(feature = "console_only"))]
pub mod windowed_mode;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mode {
    Win,
    ConAlpha,
    ConBlock,
    ConBrail,
}

impl Mode {
    pub fn default() -> Mode {
        #[cfg(not(feature = "console_only"))]
        return Mode::Win;

        #[allow(unreachable_code)]
        return Mode::ConBlock;
    }

    #[cfg(not(feature = "window_only"))]
    pub fn get_flusher(&self) -> console_mode::Flusher {
        use crate::Program;

        match *self {
            Mode::ConAlpha => Program::print_alpha,
            Mode::ConBrail => Program::print_brail,
            _ => Program::print_block,
        }
    }

    pub fn is_con(&self) -> bool {
        matches!(self, Mode::ConAlpha | Mode::ConBlock | Mode::ConBrail)
    }
}
