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
/*
use sixel_rs;
use sixel_sys;
*/
use crate::{data::*, graphics::blend::grayb, modes::Mode};
use std::io::{stdout, Stdout, Write};

pub type Flusher = fn(&Program, &mut Stdout);

type Rgb = [u8; 3];
const BLACK: Rgb = [0; 3];
const ERROR: u8 = 6;

struct ColoredString {
    pub string: String,
    pub fg: Rgb,
    pub bg: Rgb,
    error: u8,
}

impl ColoredString {
    pub fn new(ch: char, fg: Rgb, error: u8) -> Self {
        Self {
            string: String::from(ch),
            fg: fg,
            bg: BLACK,
            error,
        }
    }

    pub fn new_bg(ch: char, fg: Rgb, bg: Rgb, error: u8) -> Self {
        Self {
            string: String::from(ch),
            fg: fg,
            bg: bg,
            error,
        }
    }

    pub fn append(&mut self, ch: char, fg: Rgb) -> bool {
        /*if ch == '\0' {
            self.string.push_str("\x1B[1C");
            return true;
        }*/

        let [r, g, b] = self.fg;
        let [nr, ng, nb] = fg;

        let er = r.abs_diff(nr);
        let eg = g.abs_diff(ng);
        let eb = b.abs_diff(nb);

        if er <= self.error && eg <= self.error && eb <= self.error {
            self.string.push(ch);
            return true;
        }

        false
        //return Some(Self::new(ch, r, g, b, self.error));
    }

    pub fn append_bg(&mut self, ch: char, fg: Rgb, bg: Rgb) -> bool {
        /*if ch == '\0' {
            self.string.push_str("\x1B[1C");
            return true;
        }*/

        let mergable_fg = {
            let [r, g, b] = self.fg;
            let [nr, ng, nb] = fg;

            let er = r.abs_diff(nr);
            let eg = g.abs_diff(ng);
            let eb = b.abs_diff(nb);

            er <= self.error && eg <= self.error && eb <= self.error
        };

        let mergable_bg = {
            let [r, g, b] = self.bg;
            let [nr, ng, nb] = bg;

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
    fn push_pixel(&mut self, ch: char, fg: Rgb);
    fn push_pixel_bg(&mut self, ch: char, fg: Rgb, bg: Rgb);
    fn queue_print(&self);
}

impl StyledLine for Vec<ColoredString> {
    fn init() -> Self {
        vec![ColoredString::new('\0', BLACK, ERROR)]
    }

    fn clear_line(&mut self) {
        self.clear();
        self.push(ColoredString::new('\0', BLACK, ERROR));
    }

    fn push_pixel(&mut self, ch: char, fg: Rgb) {
        if let Some(last) = self.last_mut() {
            if last.append(ch, fg) {
                return;
            }
        }

        self.push(ColoredString::new(ch, fg, ERROR));
    }

    fn push_pixel_bg(&mut self, ch: char, fg: Rgb, bg: Rgb) {
        if let Some(last) = self.last_mut() {
            if last.append_bg(ch, fg, bg) {
                return;
            }
        }

        self.push(ColoredString::new_bg(ch, fg, bg, ERROR));
    }

    fn queue_print(&self) {
        for ColoredString {
            string,
            fg: [r, g, b],
            ..
        } in self
        {
            let _ = queue!(
                stdout(),
                Print(string.clone().with(Color::Rgb {
                    r: *r,
                    g: *g,
                    b: *b
                }))
            );
        }
    }
}

// ASCII ONLY
// pub const CHARSET_BLOCKOPAC: &str = &[' ', '░', '▒', '▓'];
const CHARSET_OPAC_EXP: &[u8] = b" `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ\
    5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@";

#[allow(dead_code)]
pub const CHARSET_SIZEOPAC: &[u8] = b" -~+oiwGW@$";

impl Program {
    pub fn print_con(&mut self) {
        (self.flusher)(self, &mut std::io::stdout());
    }

    pub fn as_con(mut self) -> Self {
        match self.mode {
            Mode::Win => self.set_con_mode(Mode::ConAlpha),
            _ => {}
        }
        self
    }

    pub fn as_con_force(mut self, mode: Mode) -> Self {
        self.set_con_mode(mode);
        self
    }

    pub fn change_con_max(&mut self, amount: i16, replace: bool) {
        self.CON_MAX_W = ((self.CON_MAX_W * (!replace) as u16) as i16 + amount)
            .clamp(0, self.pix.width() as i16) as u16;
        self.CON_MAX_H = self.CON_MAX_W >> 1;
        self.clear_con();
    }

    pub fn refresh_con(&mut self) {
        self.update_size((self.CON_W, self.CON_H));
    }

    pub fn switch_con_mode(&mut self) {
        // self.mode.next_con();

        self.mode = match self.mode {
            Mode::ConAlpha => {
                self.flusher = Program::print_block;
                Mode::ConBlock
            }
            Mode::ConBlock => {
                self.flusher = Program::print_brail;
                Mode::ConBrail
            }
            Mode::ConBrail => {
                self.flusher = Program::print_alpha;
                Mode::ConAlpha
            }

            Mode::Win => Mode::Win,
            Mode::WinLegacy => Mode::WinLegacy,
        };

        self.refresh_con();
    }

    pub fn set_con_mode(&mut self, mode: Mode) {
        match mode {
            Mode::ConAlpha => self.flusher = Program::print_alpha,
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
            (self.CON_W / 2).saturating_sub(self.pix.width() as u16 / divider_x),
            (self.CON_H / 2).saturating_sub(self.pix.height() as u16 / divider_y),
        )
    }

    pub fn print_alpha(&self, stdout: &mut Stdout) {
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

                let alpha_char = to_ascii_art(CHARSET_OPAC_EXP, lum);

                line.push_pixel(alpha_char, [r, g, b]);
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

                let mut bg = [0u8; 3];

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
                            bg = [pr, pg, pb];
                            // blocks that aren't drawn can still be displayed
                            // by addding background color
                        }

                        _ => {}
                    }

                    acc
                });

                let block_char = ([' ', '▄', '▀', '█'])[bx as usize];

                // let [_, bgr, bgg, bgb] = self.bg.to_be_bytes();

                line.push_pixel_bg(block_char, [r, g, b], bg);

                //Self::print_ascii_with_bg(stdout, block_char, r, g, b, bg);
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

                line.push_pixel(char::from_u32(bx).unwrap(), [r, g, b]);
            }

            line.queue_print();
            line.clear_line();
        }
    }
    /*
    pub fn print_sixel(&self, stdout: &mut Stdout) {
        let w = self.pix.width();
        let h = self.pix.height();
        let canvas =
            sixel_rs::encoder::QuickFrameBuilder::new()
            .width(w)
            .height(h)
            .format(sixel_sys::PixelFormat::ARGB8888)
            .pixels(self.pix.iter().map(|x| x.to_be_bytes()).flatten().collect::<Vec<u8>>())
        ;

    }*/
}

fn to_ascii_art<T>(table: &[u8], x: T) -> char
where
    usize: From<T>,
{
    table[(usize::from(x) * table.len()) >> 8] as char
}

#[allow(dead_code)]
fn rgb_to_ansi(r: u8, g: u8, b: u8) -> u8 {
    (16 + (r as u16 * 6 / 256) * 36 + (g as u16 * 6 / 256) * 6 + (b as u16 * 6 / 256)) as u8
}

pub fn rescale(mut s: (u16, u16), prog: &Program) -> (u16, u16) {
    s.0 = s.0.min(prog.CON_MAX_W);
    s.1 = s.1.min(prog.CON_MAX_H);

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

pub fn control_key_events_con(prog: &mut Program, exit: &mut bool) -> std::io::Result<()> {
    prog.update_vis();

    let refresh_rates = [
        prog.REFRESH_RATE,
        prog.REFRESH_RATE * 10,
        prog.REFRESH_RATE * 50,
    ];
    let no_sample = crate::audio::get_no_sample();
    let inactive = (no_sample > 64) as u8 + (no_sample > 192) as u8;

    let mut command = Command::Blank;

    if poll(refresh_rates[inactive as usize])? {
        match read()? {
            Event::Key(event) => {
                command = match event.code {
                    KeyCode::Char('b') => Command::VisualizerPrev,

                    KeyCode::Char(' ') => Command::VisualizerNext,

                    KeyCode::Char('q') => {
                        *exit = true;
                        Command::Blank
                    }

                    KeyCode::Char('1') => Command::Fps(10, true),
                    KeyCode::Char('2') => Command::Fps(20, true),
                    KeyCode::Char('3') => Command::Fps(30, true),
                    KeyCode::Char('4') => Command::Fps(40, true),
                    KeyCode::Char('5') => Command::Fps(50, true),
                    KeyCode::Char('6') => Command::Fps(60, true),

                    KeyCode::Char('7') => Command::Fps(-5, false),
                    KeyCode::Char('8') => Command::Fps(5, false),
                    KeyCode::Char('9') => Command::ConMax(-1, false),
                    KeyCode::Char('0') => Command::ConMax(1, false),

                    KeyCode::Char('-') => Command::VolDown,
                    KeyCode::Char('=') => Command::VolUp,

                    KeyCode::Char('[') => Command::SmoothDown,
                    KeyCode::Char(']') => Command::SmoothUp,

                    KeyCode::Char(';') => Command::WavDown,
                    KeyCode::Char('\'') => Command::WavUp,

                    KeyCode::Char('\\') => Command::AutoSwitch,

                    KeyCode::Char('.') => Command::SwitchConMode,

                    KeyCode::Char('n') => Command::SwitchVisList,

                    KeyCode::Char('/') => Command::Reset,

                    _ => Command::Blank,
                }
            }

            Event::Resize(w, h) => {
                prog.update_size((w, h));
                prog.clear_con();
            }
            _ => {}
        }
    }

    prog.eval_command(&command);

    Ok(())
}

pub fn con_main(mut prog: Program) -> std::io::Result<()> {
    let mut stdout = stdout();

    enable_raw_mode()?;
    stdout.flush().unwrap();

    let mut EXIT: bool = false;

    let size = size()?;
    prog.update_size(size);

    if !prog.is_display_enabled() {
        while !EXIT {
            control_key_events_con(&mut prog, &mut EXIT)?;
            prog.force_render();
        }
    } else {
        let _ = queue!(
            stdout,
            EnterAlternateScreen,
            Hide,
            SetAttribute(Attribute::Bold)
        );
        while !EXIT {
            control_key_events_con(&mut prog, &mut EXIT)?;

            if crate::audio::get_no_sample() > crate::data::STOP_RENDERING {
                continue;
            }

            prog.force_render();
            prog.print_con();
            // prog.print_err_con();

            let _ = stdout.flush();
        }
        let _ = queue!(stdout, LeaveAlternateScreen, Show)?;
    }
    disable_raw_mode()?;

    Ok(())
}

/*
pub fn prepare_stdout_ascii(prog: &mut Program, stdout: &mut Stdout) {
    let center = (
        (prog.CON_W /2).saturating_sub(prog.pix.width() as u16 /2),
        (prog.CON_H /2).saturating_sub(prog.pix.height() as u16 /4)
    );

    for y in 0..prog.pix.height() {
        let cy = center.1 + y as u16 / 2;
        queue!(stdout, cursor::MoveTo(center.0, cy));

        for x in 0..prog.pix.width() {
            let [_, r, g, b] = prog.pix[y*prog.pix.width() + x].to_le_bytes();
           // let cx = center.0 + x as u16;

            //let gray = (30 * r as usize + 59 * g as usize + 11 * b as usize) / 100;
            let gray = grayb(r, g, b);
            //let ansi = rgb_to_ansi(r, g, b);
            let c = to_art(prog.ascii_set, gray);

            queue!(
                stdout,

                SetColors(Colors::new(
                    Color::Rgb{r: r, g: g, b: b},
                    //Color::AnsiValue(ansi),
                    Color::Reset
                )),
                Print(c)
            );

        }
    }
}

pub fn prepare_stdout_braille(prog: &mut Program, stdout: &mut Stdout) {
    let center = (
        (prog.CON_W /2).saturating_sub(prog.pix.width() as u16 /4),
        (prog.CON_H /2).saturating_sub(prog.pix.height() as u16 /8)
    );

    //let w = prog.pix.width().saturating_sub(2) as u32;
    //let h = prog.pix.height().saturating_sub(4) as u32;

    let w = prog.pix.width() as u32;
    let h = prog.pix.height() as u32;

    let mut braille = Canvas::new(
        w,
        h
    );

    let (mut ar, mut ag, mut ab) = (0u8, 0u8, 0u8);

    //let hh = prog.pix.height()-1;

    for (i, p) in prog.pix.iter().enumerate() {
        let (r, g, b) = u32_to_rgb(*p);
        let gray = (r as u16 + g as u16 + b as u16) / 144;
        let (x, y) = (
            (i % prog.pix.width()),
            (i / prog.pix.width())
        );

        if gray > 0 {
            /*if (x & 1 != 0) {
                ar = ((ar as u16 + r as u16) /2 ) as u8;
                ag = ((ar as u16 + g as u16) /2 ) as u8;
                ab = ((ar as u16 + b as u16) /2 ) as u8;
            } else {
                ar = r;
                ag = g;
                ab = b;
            }*/

               ar = r;
            ag = g;
               ab = b;

            braille.set_colored(
                x as u32, y as u32, PixelColor::TrueColor{r: ar, g: ag, b: ab}
            );

        //queue!(stdout, Clear(ClearType::All))
        }
    }

    for (y, line) in braille.rows().iter().enumerate() {
        let yl = center.1 + y as u16;
        if yl < prog.CON_H {
            queue!(stdout, cursor::MoveTo(center.0, yl), Print(line));
        }
    }

    //queue!(stdout, cursor::MoveTo(1, 1), Print(braille.frame()));
}


pub fn console_draw_braille(prog: &mut Program, stdout: &mut Stdout) {
    let mut
}*/
