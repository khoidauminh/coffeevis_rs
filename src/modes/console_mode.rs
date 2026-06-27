use crossterm::{
    cursor::{self, Hide, Show},
    event::{Event, KeyCode, poll, read},
    queue,
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode, size,
    },
};

use arrayvec::ArrayString;

use std::io::{Error, Stdout, Write, stdout};

use crate::{
    data::*,
    graphics::{Argb, Painter, Pixel, blend::grayb},
    modes::Mode,
};

pub type Flusher = fn(&Program, &Painter, &mut Stdout);

const ERROR: u8 = 6;

const CHARSET_OPAC_EXP: &[u8] = b" `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ\
    5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@";

pub struct ConsoleProps {
    pub width: u16,
    pub height: u16,

    pub physical_width: u16,
    pub physical_height: u16,

    pub max_width: u16,
    pub max_height: u16,
    pub flusher: Flusher,
}

impl ConsoleProps {
    pub fn set_size(&mut self, s: (u16, u16), m: Mode) {
        self.physical_width = s.0;
        self.physical_height = s.1;

        let s = self.rescale(s, m);

        self.width = s.0;
        self.height = s.1;
    }

    pub fn set_max(&mut self, s: (u16, u16)) {
        self.max_width = s.0;
        self.max_height = s.1;
    }

    pub fn rescale(&self, mut s: (u16, u16), m: Mode) -> (u16, u16) {
        s.0 = s.0.min(self.max_width);
        s.1 = s.1.min(self.max_height);

        match m {
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
}

struct ColoredString {
    pub string: ArrayString<{ MAX_CON_WIDTH as usize }>,
    pub fg: Argb,
    error: u8,
}

/// Compress similar pixels into one string with the same
/// color. Hopefully this reduces IO performance cost.
impl ColoredString {
    pub fn new(ch: char, fg: Argb, error: u8) -> Self {
        let mut string = ArrayString::new();
        string.push(ch);

        Self { string, fg, error }
    }

    pub fn default() -> Self {
        ColoredString::new('\0', Argb::black(), ERROR)
    }

    pub fn append(&mut self, ch: char, fg: Argb) -> bool {
        let [_, r, g, b] = self.fg.decompose();
        let [_, nr, ng, nb] = fg.decompose();

        let er = r.abs_diff(nr);
        let eg = g.abs_diff(ng);
        let eb = b.abs_diff(nb);

        if er <= self.error && eg <= self.error && eb <= self.error {
            _ = self.string.try_push(ch);
            return true;
        }

        false
    }

    pub fn replace(&mut self, ch: char, fg: Argb) {
        self.string.clear();
        self.string.push(ch);
        self.fg = fg;
    }

    fn push_pixel(&mut self, ch: char, fg: Argb) {
        if !self.append(ch, fg) {
            self.queue_print();
            self.replace(ch, fg);
        }
    }

    fn queue_print(&mut self) {
        let [_, r, g, b] = self.fg.decompose();
        let _ = queue!(
            stdout(),
            SetForegroundColor(Color::Rgb { r, g, b }),
            Print(&self.string)
        );
    }

    fn reset(&mut self) {
        self.replace('\0', Argb::black());
    }
}

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
    pub fn clear_con(&mut self) {
        let _ = queue!(std::io::stdout(), Clear(ClearType::All));
    }

    pub fn switch_con_mode(&mut self) {
        self.set_mode(self.mode().next());
        self.console_props.flusher = self.mode().get_flusher();
        if let Ok(size) = size() {
            self.update_size(size);
        }
    }

    pub fn change_con_max(&mut self, amount: i16, replace: bool) {
        self.console_props.max_width = if replace {
            amount as u16
        } else {
            self.console_props
                .max_width
                .saturating_add_signed(amount)
                .clamp(0, MAX_CON_WIDTH)
        };
        self.console_props.max_height = self.console_props.max_width / 2;
        self.clear_con();
    }

    pub fn get_center(
        &self,
        width: u16,
        height: u16,
        divider_x: u16,
        divider_y: u16,
    ) -> (u16, u16) {
        (
            (self.console_props.physical_width / 2).saturating_sub(width / divider_x),
            (self.console_props.physical_height / 2).saturating_sub(height / divider_y),
        )
    }

    pub fn console_size(&self) -> (u16, u16) {
        (self.console_props.width, self.console_props.height)
    }

    pub fn print_ascii(&self, pix: &Painter, stdout: &mut Stdout) {
        let center = self.get_center(
            pix.logical_width() as u16,
            pix.logical_height() as u16,
            2,
            4,
        );

        let mut line = ColoredString::default();

        for y in (0..pix.logical_height()).step_by(2) {
            let cy = center.1 + y as u16 / 2;
            let _ = queue!(stdout, cursor::MoveTo(center.0, cy));

            for x in 0..pix.logical_width() {
                let base = pix.logical_width() * y + x;

                let [_, mut r, mut g, mut b] = pix.pixel(base).to_be_bytes();

                let [_, nr, ng, nb] = pix.pixel(base + pix.logical_width()).to_be_bytes();

                r = r.max(nr);
                g = g.max(ng);
                b = b.max(nb);

                let lum = grayb(r, g, b);

                let alpha_char = to_ascii_art(CHARSET_OPAC_EXP, lum as usize);

                line.push_pixel(alpha_char, Argb::compose([0, r, g, b]));
            }

            line.queue_print();
            line.reset();
        }
    }

    pub fn print_block(&self, pix: &Painter, stdout: &mut Stdout) {
        let center = self.get_center(
            pix.logical_width() as u16,
            pix.logical_height() as u16,
            2,
            4,
        );

        let mut line = ColoredString::default();

        for y_base in (0..pix.logical_height()).step_by(2) {
            let cy = center.1 + y_base as u16 / 2;
            let _ = queue!(stdout, cursor::MoveTo(center.0, cy));

            for x_base in 0..pix.logical_width() {
                let idx_base = y_base * pix.logical_width() + x_base;
                let [_, mut r, mut g, mut b] = pix.pixel(idx_base).to_be_bytes();

                let bx = (0..2).fold(0, |acc, i| {
                    let idx = idx_base + i * pix.logical_width(); // iterate horizontally, then jump to the nex row;
                    let [_, pr, pg, pb] = pix.pixel(idx).to_be_bytes();

                    if grayb(pr, pg, pb) >= 48 {
                        // Values smaller than this becomes transparent.
                        r = r.max(pr);
                        g = g.max(pg);
                        b = b.max(pb);

                        return acc | (1 << (1 - i));
                    }

                    acc
                });

                let block_char = ([' ', '▄', '▀', '█'])[bx as usize];

                line.push_pixel(block_char, Argb::compose([0, r, g, b]));
            }

            line.queue_print();
            line.reset();
        }
    }

    pub fn print_brail(&self, pix: &Painter, stdout: &mut Stdout) {
        let center = self.get_center(
            pix.logical_width() as u16,
            pix.logical_height() as u16,
            4,
            8,
        );

        let mut line = ColoredString::default();

        for y_base in (0..pix.logical_height()).step_by(4) {
            let cy = center.1 + y_base as u16 / 4;

            let _ = queue!(stdout, cursor::MoveTo(center.0, cy));

            for x_base in (0..pix.logical_width()).step_by(2) {
                let idx_base = y_base * pix.logical_width() + x_base;

                let [_, mut r, mut g, mut b] = pix.pixel(idx_base).to_be_bytes();

                let bx = '⠀' as u32 + // first char of braille
					(0..8).fold(0u8, |acc, i| {
						let idx = idx_base +
							if i < 6 {
								(i / 3) + (i % 3)*pix.logical_width()
							} else {
								(i & 1) + 3*pix.logical_width()
							}
						;

						let [_pa, pr, pg, pb] = pix.pixel(idx).to_be_bytes();

						r = r.max(pr);
						g = g.max(pg);
						b = b.max(pb);

						acc | (((grayb(pr, pg, pb) > 36) as u8) << i)
						// All braille patterns fit into a u8, so a bitwise OR can
						// be used to increase performance.
					}) as u32;

                line.push_pixel(
                    char::from_u32(bx).unwrap_or('⠀'),
                    Argb::compose([0, r, g, b]),
                );
            }

            line.queue_print();
            line.reset();
        }
    }
}

fn to_ascii_art(table: &[u8], x: usize) -> char {
    table[(x * table.len()) >> 8] as char
}

pub fn control_key_events_con(prog: &mut Program, exit: &mut bool) -> Result<bool, Error> {
    prog.autoupdate_visualizer();

    prog.key = KeyInput::default();

    if poll(prog.get_rr_interval())? {
        match read()? {
            Event::Key(event) => {
                match event.code {
                    KeyCode::Char('b') => prog.change_visualizer(false),
                    KeyCode::Char('n') => prog.change_visualizer(true),

                    KeyCode::Char('q') => *exit = true,

                    KeyCode::Char('1') => prog.change_fps(10, true),
                    KeyCode::Char('2') => prog.change_fps(20, true),
                    KeyCode::Char('3') => prog.change_fps(30, true),
                    KeyCode::Char('4') => prog.change_fps(40, true),
                    KeyCode::Char('5') => prog.change_fps(50, true),
                    KeyCode::Char('6') => prog.change_fps(60, true),

                    KeyCode::Char('7') => prog.change_fps(-5, false),
                    KeyCode::Char('8') => prog.change_fps(5, false),

                    KeyCode::Char('z') => prog.key.z = true,
                    KeyCode::Char('x') => prog.key.x = true,
                    KeyCode::Char('c') => prog.key.c = true,

                    KeyCode::Left => prog.key.left = true,
                    KeyCode::Right => prog.key.right = true,
                    KeyCode::Up => prog.key.up = true,
                    KeyCode::Down => prog.key.down = true,

                    KeyCode::Char('9') => {
                        prog.change_con_max(-1, false);
                        prog.update_size(prog.console_size())
                    }

                    KeyCode::Char('0') => {
                        prog.change_con_max(1, false);
                        prog.update_size(prog.console_size())
                    }

                    KeyCode::Char('\\') => prog.toggle_auto_switch(),

                    KeyCode::Char('.') => {
                        prog.switch_con_mode();
                        prog.clear_con();
                    }

                    KeyCode::Char('/') => prog.reset_parameters(),

                    _ => {}
                }
                return Ok(true);
            }

            Event::Resize(w, h) => {
                prog.update_size((w, h));
                prog.clear_con();
                return Ok(true);
            }
            _ => {}
        }
    }

    Ok(false)
}

pub fn con_main(mut prog: Program) -> std::io::Result<()> {
    prog.print_startup_info();

    let mut stdout = stdout();

    enable_raw_mode()?;
    stdout.flush()?;

    let mut exit: bool = false;

    let _ = queue!(
        stdout,
        EnterAlternateScreen,
        Hide,
        SetAttribute(Attribute::Bold)
    );

    let size = size()?;
    prog.update_size(size);
    let mut raw_buffer = vec![0x0; (prog.console_props.width * prog.console_props.height) as usize];

    while !exit {
        let forcedraw = control_key_events_con(&mut prog, &mut exit)?;

        let mut buf = crate::audio::get_buf();
        let silent = buf.silent();

        if silent > STOP_RENDERING && !prog.nosleep() && !forcedraw {
            continue;
        }

        raw_buffer.resize(
            (prog.console_props.width * prog.console_props.height) as usize,
            0,
        );

        let mut pix = Painter::from(
            &mut raw_buffer,
            prog.console_props.width as usize,
            prog.console_props.height as usize,
            1,
            0,
            false,
        );
        (prog.console_props.flusher)(&prog, &pix, &mut std::io::stdout());
        prog.render(&mut pix, &mut buf);

        let _ = stdout.flush();
    }

    queue!(stdout, LeaveAlternateScreen, Show)?;

    disable_raw_mode()?;

    Ok(())
}
