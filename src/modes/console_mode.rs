use crossterm::{
    queue,
    Command,
    QueueableCommand,
    event::{poll, read, Event, KeyCode},
    terminal::{
        Clear,
        ClearType,
        disable_raw_mode,
        enable_raw_mode,
        size,
        EnterAlternateScreen,
        LeaveAlternateScreen,
        DisableLineWrap,
        EnableLineWrap
    },
    style::{Stylize, Colors, SetColors, Print, Color, Attribute, SetAttribute},
    cursor::{self, Hide, Show},
    Result
};
/*
use sixel_rs;
use sixel_sys;
*/
use std::{sync::{Arc, RwLock}, io::{Stdout, Stdin, stdout, stdin, Write}};
use crate::{
    audio::get_buf,
    graphics::grayb,
    data::*,
    controls::{usleep, control_key_events_con},
    math::Cplx,
    visualizers::VisFunc,
    modes::Mode,
};

pub type Flusher = fn(&Program, &mut Stdout);

use std::str::Chars;

// ASCII ONLY
// pub const CHARSET_BLOCKOPAC: &str = &[' ', '░', '▒', '▓'];
const CHARSET_OPAC_EXP: &[u8] = b"`.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@";
//pub const CHARSET_SIZEOPAC: &[u8] = &[' ', '-', '~', '+', 'o', 'i', 'w', 'G', 'W', '@', '$'].as_bytes();
pub const CHARSET_SIZEOPAC: &[u8] = b" -~+oiwGW@$";

impl Program
{
	pub fn print_con(&mut self) {
		(self.flusher)(self, &mut std::io::stdout());
	}

	pub fn change_con_max(
		&mut self,
		amount: i16,
		replace: bool,
	) {
		self.CON_MAX_W =
			((self.CON_MAX_W * (!replace) as u16) as i16 + amount)
			.clamp(0, self.pix.width() as i16)
			as u16
			;
		self.CON_MAX_H = self.CON_MAX_W >> 1;
		self.clear_con();
	}

	pub fn switch_con_mode(&mut self) {
		// self.mode.next_con();

		self.mode = match self.mode {
			Mode::ConAlpha => {
				self.flusher = Program::print_block;
				Mode::ConBlock
			},
			Mode::ConBlock => {
				self.flusher = Program::print_brail;
				Mode::ConBrail
			},
			Mode::ConBrail => {
				self.flusher = Program::print_alpha;
				Mode::ConAlpha
			},

			Mode::Win       => Mode::Win,
			Mode::WinLegacy => Mode::WinLegacy,
		};

		self.refresh_con();
	}

	pub fn set_con_mode(&mut self, mode: Mode) {
		match mode {
			Mode::ConAlpha => self.flusher = Program::print_alpha,
			Mode::ConBlock => self.flusher = Program::print_block,
			Mode::ConBrail => self.flusher = Program::print_brail,
			_ => {},
		}
		self.mode = mode;
		self.refresh_con();
	}

	pub fn refresh_con(&mut self) {
		self.update_size((self.CON_W, self.CON_H));
	}

	pub fn clear_con(&mut self) {
		queue!(std::io::stdout(), Clear(ClearType::All));
	}

	fn get_center(&self, divider_x: u16, divider_y: u16) -> (u16, u16)
	{
		(
			(self.CON_W /2).saturating_sub(self.pix.width() as u16 /divider_x),
			(self.CON_H /2).saturating_sub(self.pix.height() as u16 /divider_y)
		)
	}

	fn print_ascii(stdout: &mut Stdout, ch: char, r: u8, g: u8, b: u8, bg: Option<(u8, u8, u8)>)
	{
		/*if ch != ' ' && ch != '⠀' /*empty braille*/ {
			queue!(
				stdout,
				Print(
					ch
					.with(Color::Rgb{r: r, g: g, b: b})
					.on(match bg {
						Some((r, g, b)) => Color::Rgb{r: r, g: g, b: b},
						None => Color::Reset,
					})
				)
			);
		} else {
			queue!(
				stdout,
				Print(' ')
			);
		}*/

		queue!(
			stdout,
			Print(
				ch
				.with(Color::Rgb{r: r, g: g, b: b})
				.on(match bg {
					Some((r, g, b)) => Color::Rgb{r: r, g: g, b: b},
					None => Color::Reset,
				})
			)
		);
	}

	pub fn print_alpha(&self, stdout: &mut Stdout)
	{
		let center = self.get_center(2, 4);

		for y in (0..self.pix.height())
		{

			let cy = center.1 + y as u16 / 2;
			queue!(stdout, cursor::MoveTo(center.0, cy));

			for x in (0..self.pix.width())
			{

				let i = y*self.pix.width() + x;

				let [a, r, g, b] = self.pix.pixel(i).to_be_bytes();

				let lum = grayb(r, g, b);

				let alpha_char = to_art(CHARSET_SIZEOPAC, lum);
				//let alpha_char = CHARSET_OPAC_EXP[lum as usize*CHARSET_OPAC_EXP.len() / 256] as char;

				Self::print_ascii(stdout, alpha_char, r, g, b, None);

			}
		}
	}

	pub fn print_block(&self, stdout: &mut Stdout)
	{
		let center = self.get_center(2, 4);

		for y_base in (0..self.pix.height()).step_by(2)
		{
			let cy = center.1 + y_base as u16 / 2;
			queue!(stdout, cursor::MoveTo(center.0, cy));

			for x_base in (0..self.pix.width()).step_by(1)
			{

				let idx_base = y_base*self.pix.width() + x_base;
				let [_, mut r, mut g, mut b] = self.pix.pixel(idx_base).to_be_bytes();

				let mut bg: Option<(u8, u8, u8)> = None;

				let bx =
					(0..2).fold(0, |acc, i| {
						let idx = idx_base + i*self.pix.width(); // iterate horizontally, then jump to the nex row;
						let [_, pr, pg, pb] = self.pix.pixel(idx).to_be_bytes();

						/*
						r = ((r as u16 + pr as u16) / 2) as u8;
						g = ((g as u16 + pg as u16) / 2) as u8;
						b = ((b as u16 + pb as u16) / 2) as u8;
						* */

						// let check = grayb(pr, pg, pb) > 36;

						match grayb(pr, pg, pb)
						{

							48.. =>
							{
								r = r.max(pr);
								g = g.max(pg);
								b = b.max(pb);

								return acc | (1 << (1-i));
							},

							32..=47 =>
							{
								bg = Some((pr, pg, pb));
								// blocks that aren't drawn can still be displayed
								// by addding background color
							}

							_ => {}
						}

						return acc;
					});

				let block_char =
					([' ', '▄', '▀', '█'])
					[bx as usize]
				;

				// let [_, bgr, bgg, bgb] = self.bg.to_be_bytes();

				Self::print_ascii(stdout, block_char, r, g, b, bg);
			}
		}
	}

	pub fn print_brail(&self, stdout: &mut Stdout)
	{
		let center = self.get_center(4, 8);

		for y_base in (0..self.pix.height()).step_by(4)
		{
			let cy = center.1 + y_base as u16 / 4;

			queue!(stdout, cursor::MoveTo(center.0, cy));

			for x_base in (0..self.pix.width()).step_by(2)
			{

				let idx_base = y_base*self.pix.width() + x_base;

				let [_, mut r, mut g, mut b] = self.pix.pixel(idx_base).to_be_bytes();

				let bx = '⠀' as u32 + // first char of braille
					(0..8).fold(0u8, |acc, i|
					{
						let idx = idx_base +
							if i < 6 {
								(i / 3) + (i % 3)*self.pix.width()
							} else {
								(i & 1) + 3*self.pix.width()
							}
						;

						let [pa, pr, pg, pb] = self.pix.pixel(idx).to_be_bytes();

						r = r.max(pr);
						g = g.max(pg);
						b = b.max(pb);

						acc | (((grayb(pr, pg, pb) > 36) as u8) << i)
						// All braille patterns fit into a u8, so a bitwise or can
						// be used to increase performance
					}) as u32;

				let braille_char = char::from_u32(bx).unwrap_or(' ');

				Self::print_ascii(stdout, braille_char, r, g, b, None);
			}
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

fn to_art<T>(table: &[u8], x: T) -> char
where usize: From<T>
{
    *table.get(usize::from(x) * table.len() / 256).unwrap_or(&b' ') as char
}

fn rgb_to_ansi(r: u8, g: u8, b: u8) -> u8
{
    (16 + (r as u16 *6/256)*36 + (g as u16 *6/256)*6 + (b as u16*6/256)) as u8
}

pub fn rescale(mut s: (u16, u16), prog: &Program) -> (u16, u16)
{
    use super::Mode;

	s.0 = s.0.min(prog.CON_MAX_W);
    s.1 = s.1.min(prog.CON_MAX_H);

	match prog.mode {
		Mode::ConBrail => {
			s.0 <<= 1;
			s.1 <<= 2;
		},
		_ => {
			s.1 <<= 1;
		}
	}

    s
}

pub fn con_main(mut prog: Program) -> Result<()>
{
	let mut stdout = stdout();

    enable_raw_mode()?;
    stdout.flush().unwrap();

	let mut EXIT: bool = false;

    let size = size()?;
    prog.update_size(size);


	if !prog.DISPLAY
	{
		while !EXIT
		{
			control_key_events_con(&mut prog, &mut EXIT)?;
			prog.render();
		}
	}
	else
	{
		queue!(
			stdout, EnterAlternateScreen,
			Hide,
			SetAttribute(Attribute::Bold)
		);
		while !EXIT
		{
			control_key_events_con(&mut prog, &mut EXIT)?;
			prog.render();
			prog.print_con();
			prog.print_err_con();

			stdout.flush().unwrap();
		}
		queue!(stdout, LeaveAlternateScreen, Show)?;
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
