use crate::modes::Mode;

pub fn write_to_stdout(string: &str, mode: Mode) {
    match mode {
        Mode::Win => println!("{}", string),

        _ => {
            use crossterm::{self, queue, terminal::{EnterAlternateScreen as ES, LeaveAlternateScreen as LS}, style::Print};
            queue!(std::io::stdout(), LS, Print(string.to_string()), ES);
        }
    }
}

