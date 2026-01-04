#![allow(unused_macros, unused_imports)]

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

static ENABLED: AtomicBool = AtomicBool::new(true);

pub fn is_log_enabled() -> bool {
    ENABLED.load(Relaxed)
}

pub fn set_log_enabled(b: bool) {
    ENABLED.store(b, Relaxed)
}

macro_rules! info {
    ($($arg:tt)*) => {
        if crate::data::log::is_log_enabled() {
            println!("{}", format_args!($($arg)*))
        }
    };
}

macro_rules! alert {
    ($($arg:tt)*) => {
        if crate::data::log::is_log_enabled() {
            println!("\x1B[95;1m{}\x1B[0m", format_args!($($arg)*))
        }
    };
}

macro_rules! format_red {
    ($($arg:tt)*) => {
        format!("\x1B[31;1m{}\x1B[0m", format!($($arg)*))
    };
}

macro_rules! error {
    ($lit:tt $(, $arg:tt)*) => {
        eprintln!("\x1B[31;1m{}\x1B[0m", format_args!($lit $(, $arg)*))
    };
}

pub(crate) use alert;
pub(crate) use error;
pub(crate) use info;
