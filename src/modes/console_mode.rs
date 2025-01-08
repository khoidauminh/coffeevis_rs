use crossterm::{
    cursor::{self, Hide, Show},
    event::{poll, read, Event, KeyCode},
    queue,
    style::{Attribute, Color, Print, SetAttribute, Stylize},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use crate::{
    audio::get_no_sample,
    data::*,
    graphics::{
        blend::{grayb, Argb, Blend},
        Pixel,
    },
    modes::Mode,
};
use std::io::{stdout, Stdout, Write};

pub type Flusher = fn(&Program, &mut Stdout);

const ERROR: u8 = 6;

struct ColoredString {
    pub string: String,
    pub fg: Argb,
    pub bg: Argb,
    error: u8,
}

impl ColoredString {
    pub fn new(ch: char, fg: Argb, error: u8) -> Self {
        Self {
            string: String::from(ch),
            fg,
            bg: Argb::black(),
            error,
        }
    }

    pub fn new_bg(ch: char, fg: Argb, bg: Argb, error: u8) -> Self {
        Self {
            string: String::from(ch),
            fg,
            bg,
            error,
        }
    }

    pub fn append(&mut self, ch: char, fg: Argb) -> bool {
        let [_, r, g, b] = self.fg.decompose();
        let [_, nr, ng, nb] = fg.decompose();

        let er = r.abs_diff(nr);
        let eg = g.abs_diff(ng);
        let eb = b.abs_diff(nb);

        if er <= self.error && eg <= self.error && eb <= self.error {
            self.string.push(ch);
            return true;
        }

        false
    }

    pub fn append_bg(&mut self, ch: char, fg: Argb, bg: Argb) -> bool {
        let mergable_fg = {
            let [_, r, g, b] = self.fg.decompose();
            let [_, nr, ng, nb] = fg.decompose();

            let er = r.abs_diff(nr);
            let eg = g.abs_diff(ng);
            let eb = b.abs_diff(nb);

            er <= self.error && eg <= self.error && eb <= self.error
        };

        let mergable_bg = {
            let [_, r, g, b] = self.bg.decompose();
            let [_, nr, ng, nb] = bg.decompose();

            let er = r.abs_diff(nr);
            let eg = g.abs_diff(ng);
            let eb = b.abs_diff(nb);

            er <= self.error && eg <= self.error && eb <= self.error
        };

        if mergable_fg && mergable_bg {
            self.string.push(ch);
            return true;
        }

        false
    }
}

trait StyledLine {
    fn init() -> Self;
    fn clear_line(&mut self);
    fn push_pixel(&mut self, ch: char, fg: Argb);
    fn push_pixel_bg(&mut self, ch: char, fg: Argb, bg: Argb);
    fn queue_print(&self);
}

impl StyledLine for Vec<ColoredString> {
    fn init() -> Self {
        vec![ColoredString::new('\0', Argb::black(), ERROR)]
    }

    fn clear_line(&mut self) {
        self.clear();
        self.push(ColoredString::new('\0', Argb::black(), ERROR));
    }

    fn push_pixel(&mut self, ch: char, fg: Argb) {
        if let Some(last) = self.last_mut() {
            if last.append(ch, fg) {
                return;
            }
        }

        self.push(ColoredString::new(ch, fg, ERROR));
    }

    fn push_pixel_bg(&mut self, ch: char, fg: Argb, bg: Argb) {
        if let Some(last) = self.last_mut() {
            if last.append_bg(ch, fg, bg) {
                return;
            }
        }

        self.push(ColoredString::new_bg(ch, fg, bg, ERROR));
    }

    fn queue_print(&self) {
        for ColoredString { string, fg, .. } in self {
            let [_, r, g, b] = fg.decompose();
            let _ = queue!(stdout(), Print(string.clone().with(Color::Rgb { r, g, b })));
        }
    }
}

const CHARSET_OPAC_EXP: &[u8] = b" `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ\
    5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@";

impl Mode {
    pub fn get_flusher(&self) -> Flusher {
        use crate::Program;

        match *self {
            Mode::ConAscii => Program::print_ascii,
            Mode::ConBrail => Program::print_brail,
            _ => Program::print_block,
        }
    }
}

impl Program {
    pub fn print_con(&mut self) {
        (self.flusher)(self, &mut std::io::stdout());
    }

    pub fn as_con(mut self) -> Self {
        if self.mode == Mode::Win {
            self.set_con_mode(Mode::ConAscii)
        }
        self
    }

    pub fn as_con_force(mut self, mode: Mode) -> Self {
        self.set_con_mode(mode);
        self
    }

    pub fn change_con_max(&mut self, amount: i16, replace: bool) {
        self.console_max_width = ((self.console_width * (!replace) as u16) as i16 + amount)
            .clamp(0, self.pix.width() as i16) as u16;
        self.console_max_height = self.console_max_height >> 1;
        self.clear_con();
    }

    pub fn refresh_con(&mut self) {
        self.update_size((self.console_width, self.console_height));
    }

    pub fn switch_con_mode(&mut self) {
        self.mode = self.mode.next();
        self.flusher = self.mode.get_flusher();
        self.refresh_con();
    }

    pub fn set_con_mode(&mut self, mode: Mode) {
        match mode {
            Mode::ConAscii => self.flusher = Program::print_ascii,
            Mode::ConBlock => self.flusher = Program::print_block,
            Mode::ConBrail => self.flusher = Program::print_brail,
            _ => {}
        }
        self.mode = mode;
        self.refresh_con();
    }

    pub fn clear_con(&mut self) {
        let _ = queue!(std::io::stdout(), Clear(ClearType::All));
    }

    fn get_center(&self, divider_x: u16, divider_y: u16) -> (u16, u16) {
        (
            (self.console_width / 2).saturating_sub(self.pix.width() as u16 / divider_x),
            (self.console_height / 2).saturating_sub(self.pix.height() as u16 / divider_y),
        )
    }

    pub fn print_ascii(&self, stdout: &mut Stdout) {
        let center = self.get_center(2, 4);

        let mut line = Vec::<ColoredString>::init();

        for y in (0..self.pix.height()).step_by(2) {
            let cy = center.1 + y as u16 / 2;
            let _ = queue!(stdout, cursor::MoveTo(center.0, cy));

            for x in 0..self.pix.width() {
                let base = self.pix.width() * y + x;

                let [_, mut r, mut g, mut b] = self.pix.pixel(base).to_be_bytes();

                let [_, nr, ng, nb] = self.pix.pixel(base + self.pix.width()).to_be_bytes();

                r = r.max(nr);
                g = g.max(ng);
                b = b.max(nb);

                let lum = grayb(r, g, b);

                let alpha_char = to_ascii_art(CHARSET_OPAC_EXP, lum as usize);

                line.push_pixel(alpha_char, Argb::compose([0, r, g, b]));
            }

            line.queue_print();
            line.clear_line();
        }
    }

    pub fn print_block(&self, stdout: &mut Stdout) {
        let center = self.get_center(2, 4);

        let mut line = Vec::<ColoredString>::init();

        for y_base in (0..self.pix.height()).step_by(2) {
            let cy = center.1 + y_base as u16 / 2;
            let _ = queue!(stdout, cursor::MoveTo(center.0, cy));

            for x_base in (0..self.pix.width()).step_by(1) {
                let idx_base = y_base * self.pix.width() + x_base;
                let [_, mut r, mut g, mut b] = self.pix.pixel(idx_base).to_be_bytes();

                let mut bg = Argb::black();

                let bx = (0..2).fold(0, |acc, i| {
                    let idx = idx_base + i * self.pix.width(); // iterate horizontally, then jump to the nex row;
                    let [_, pr, pg, pb] = self.pix.pixel(idx).to_be_bytes();

                    match grayb(pr, pg, pb) {
                        48.. => {
                            r = r.max(pr);
                            g = g.max(pg);
                            b = b.max(pb);

                            return acc | (1 << (1 - i));
                        }

                        32..=47 => {
                            bg = Argb::compose([0, pr, pg, pb]);
                            // blocks that aren't drawn can still be displayed
                            // by addding background color
                        }

                        _ => {}
                    }

                    acc
                });

                let block_char = ([' ', '▄', '▀', '█'])[bx as usize];

                line.push_pixel_bg(block_char, Argb::compose([0, r, g, b]), bg);
            }

            line.queue_print();
            line.clear_line();
        }
    }

    pub fn print_brail(&self, stdout: &mut Stdout) {
        let center = self.get_center(4, 8);

        let mut line = Vec::<ColoredString>::init();

        for y_base in (0..self.pix.height()).step_by(4) {
            let cy = center.1 + y_base as u16 / 4;

            let _ = queue!(stdout, cursor::MoveTo(center.0, cy));

            for x_base in (0..self.pix.width()).step_by(2) {
                let idx_base = y_base * self.pix.width() + x_base;

                let [_, mut r, mut g, mut b] = self.pix.pixel(idx_base).to_be_bytes();

                let bx = '⠀' as u32 + // first char of braille
					(0..8).fold(0u8, |acc, i| {
						let idx = idx_base +
							if i < 6 {
								(i / 3) + (i % 3)*self.pix.width()
							} else {
								(i & 1) + 3*self.pix.width()
							}
						;

						let [_pa, pr, pg, pb] = self.pix.pixel(idx).to_be_bytes();

						r = r.max(pr);
						g = g.max(pg);
						b = b.max(pb);

						acc | (((grayb(pr, pg, pb) > 36) as u8) << i)
						// All braille patterns fit into a u8, so a bitwise OR can
						// be used to increase performance.
					}) as u32;

                line.push_pixel(char::from_u32(bx).unwrap(), Argb::compose([0, r, g, b]));
            }

            line.queue_print();
            line.clear_line();
        }
    }
}

fn to_ascii_art(table: &[u8], x: usize) -> char {
    table[(x * table.len()) >> 8] as char
}

pub fn rescale(mut s: (u16, u16), prog: &Program) -> (u16, u16) {
    s.0 = s.0.min(prog.console_max_width);
    s.1 = s.1.min(prog.console_max_height);

    match prog.mode() {
        Mode::ConBrail => {
            s.0 *= 2;
            s.1 *= 4;
        }
        _ => {
            s.1 *= 2;
        }
    }

    s
}

pub fn control_key_events_con(prog: &mut Program, exit: &mut bool) {
    prog.update_vis();

    let no_sample = crate::audio::get_no_sample();

    if poll(prog.get_rr_interval(no_sample)).unwrap() {
        match read().unwrap() {
            Event::Key(event) => match event.code {
                KeyCode::Char('b') => prog.change_visualizer(false),

                KeyCode::Char(' ') => prog.change_visualizer(true),

                KeyCode::Char('q') => *exit = true,

                KeyCode::Char('1') => prog.change_fps(10, true),
                KeyCode::Char('2') => prog.change_fps(20, true),
                KeyCode::Char('3') => prog.change_fps(30, true),
                KeyCode::Char('4') => prog.change_fps(40, true),
                KeyCode::Char('5') => prog.change_fps(50, true),
                KeyCode::Char('6') => prog.change_fps(60, true),

                KeyCode::Char('7') => prog.change_fps(-5, false),
                KeyCode::Char('8') => prog.change_fps(5, false),

                KeyCode::Char('9') => prog.change_con_max(-1, false),
                KeyCode::Char('0') => prog.change_con_max(1, false),

                KeyCode::Char('-') => prog.decrease_vol_scl(),
                KeyCode::Char('=') => prog.increase_vol_scl(),

                KeyCode::Char('[') => prog.decrease_smoothing(),
                KeyCode::Char(']') => prog.increase_smoothing(),

                KeyCode::Char(';') => prog.decrease_wav_win(),
                KeyCode::Char('\'') => prog.increase_wav_win(),

                KeyCode::Char('\\') => prog.toggle_auto_switch(),

                KeyCode::Char('.') => prog.switch_con_mode(),

                KeyCode::Char('n') => prog.change_vislist(),

                KeyCode::Char('/') => prog.reset_parameters(),

                _ => {}
            },

            Event::Resize(w, h) => {
                prog.update_size((w, h));
                prog.clear_con();
            }
            _ => {}
        }
    }
}

pub fn con_main(mut prog: Program) {
    prog.print_startup_info();

    let mut stdout = stdout();

    enable_raw_mode().unwrap();
    stdout.flush().unwrap();

    let mut exit: bool = false;

    let size = size().unwrap();
    prog.update_size(size);

    if !prog.is_display_enabled() {
        while !exit {
            control_key_events_con(&mut prog, &mut exit);
            prog.render();
        }
    } else {
        let _ = queue!(
            stdout,
            EnterAlternateScreen,
            Hide,
            SetAttribute(Attribute::Bold)
        );
        while !exit {
            control_key_events_con(&mut prog, &mut exit);

            if get_no_sample() > STOP_RENDERING {
                continue;
            }

            prog.render();
            prog.print_con();

            let _ = stdout.flush();
        }
        queue!(stdout, LeaveAlternateScreen, Show).unwrap();
    }

    disable_raw_mode().unwrap();
}
