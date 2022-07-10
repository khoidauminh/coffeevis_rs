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
    style::{Stylize, Colors, SetColors, Print, Color},
    cursor::{self, Hide, Show},
    Result
};
use drawille::{self, Canvas, PixelColor};

use std::{sync::{Arc, Mutex}, io::{Stdout, Stdin, stdout, stdin, Write}};
use crate::{
    graphics::graphical_fn::{update_size, u32_to_rgb},
    config::*,
    controls::{change_visualizer, usleep, control_key_events_con},
    VisFunc
};

pub const CHARSET_BLOCKOPAC: [char; 4] = [' ', '░', '▒', '▓'];
pub const CHARSET_SIZEOPAC: [char; 8] = [' ', '\'', '*', 'o', 'O', 'G', '@', '#'];
pub type OutFunc = fn(&mut Parameters, &[u32], &mut Stdout) -> ();

fn to_art<T>(table: &[char], x: T) -> char where usize: From<T> {
    table[usize::from(x) * table.len() / 256]
}

fn rgb_to_ansi(r: u8, g: u8, b: u8) -> u8 {
    (16 + (r as u16 *6/256)*36 + (g as u16 *6/256)*6 + (b as u16*6/256)) as u8
}

pub fn rescale(mut s: (u16, u16), para: &mut Parameters) -> (usize, usize) {
    let p = para.con_bool as u16; // If the drawing function (OutFunc)
                        // is for braille, double the resolution.
    para.CENTER = (s.0 >> 1, s.1 >> 1);

    s.0 = s.0.min(para.CON_MAX_W);
    s.1 = s.1.min(para.CON_MAX_H);

    s.0 <<= p;
    s.1 <<= (p+1);

    (s.0 as usize, s.1 as usize)
}

pub fn console_main() -> Result<()> {
    //let mut con = init_console(para);
    let mut stdout = stdout();

    queue!(stdout, EnterAlternateScreen, Hide, DisableLineWrap);
    enable_raw_mode()?;
    stdout.flush().unwrap();

    let mut para: Parameters = Parameters::new(crate::IMAGE);
    para.WAV_WIN = 25;

    let mut pix: Vec<u32> = Vec::new();
    //let mut vis: VisFunc = crate::graphics::visualizers::oscilloscope::draw_vectorscope;
    //let mut stdout = stdout();
    let mut EXIT: bool = false;

    {
        let size = size()?;
        update_size(rescale(size, &mut para), &mut para);
        pix.resize(para.WIN_R, 0);
    }

    while !EXIT {
        control_key_events_con(&mut para, &mut pix, &mut stdout, &mut EXIT)?;
        visualize(&mut para, &mut pix, unsafe{&crate::buf}, &mut stdout);
    }

    queue!(stdout, LeaveAlternateScreen, Show, EnableLineWrap)?;
    disable_raw_mode()?;

    Ok(())
}

fn visualize(
    para:   &mut Parameters,
    pix:    &mut Vec<u32>,
    stream: &[(f32, f32)],
    stdout: &mut Stdout
) {
    (para.vis_func)(pix, stream, para);
    (para.con_func)(para, pix, stdout);
    stdout.flush().unwrap();

    if para.SWITCH_INCR == para.AUTO_SWITCH_ITVL {
        change_visualizer(pix, para);
        para.SWITCH_INCR = 0;
    }

    para.SWITCH_INCR += para.AUTO_SWITCH;

    //usleep(FPS_ITVL);
}

pub fn prepare_stdout_ascii(para: &mut Parameters, pix: &[u32], stdout: &mut Stdout) {
    let center = (
        para.CENTER.0.saturating_sub(para.WIN_W as u16 /2),
        para.CENTER.1.saturating_sub(para.WIN_H as u16 /4)
    );

    for y in 0..para.WIN_H {
        let cy = center.1 + y as u16 / 2;
        queue!(stdout, cursor::MoveTo(center.0, cy));

        for x in 0..para.WIN_W {
            let (r, g, b) = u32_to_rgb(pix[y*para.WIN_W + x]);
           // let cx = center.0 + x as u16;

            //let gray = (30 * r as usize + 59 * g as usize + 11 * b as usize) / 100;
            let gray = (r as u16 + g as u16 + b as u16) / 3;
            let ansi = rgb_to_ansi(r, g, b);
            let c = to_art(para.ascii_set, gray);


            //format!("\x1B[{};{}H{}", y, x, c)
            queue!(
                stdout,
                //cursor::MoveTo(cx, cy),
                SetColors(Colors::new(
                  //  Color::Rgb{r: r, g: g, b: b},
                    Color::AnsiValue(ansi),
                    Color::Black
                )),
                if c > para.ascii_set[0] {Print(c)} else {Print(' ')}
            );

        }
    }
}

pub fn prepare_stdout_braille(para: &mut Parameters, pix: &[u32], stdout: &mut Stdout) {
    let center = (
        para.CENTER.0.saturating_sub(para.WIN_W as u16 /4),
        para.CENTER.1.saturating_sub(para.WIN_H as u16 /8)
    );

    let w = para.WIN_W.saturating_sub(2) as u32;
    let h = para.WIN_H.saturating_sub(4) as u32;

    let mut braille = Canvas::new(
        w,
        h
    );

    let (mut ar, mut ag, mut ab) = (0u8, 0u8, 0u8);

    //let hh = para.WIN_H-1;

    for (i, p) in pix.iter().enumerate() {
        let (r, g, b) = u32_to_rgb(*p);
        let gray = (r as u16 + g as u16 + b as u16) / 144;
        let (x, y) = (
            (i % para.WIN_W) as u32,
            (i / para.WIN_W) as u32
        );

/*        if (x & 1 != 0) {
            ar = ((ar as u16 + r as u16) /2 ) as u8;
            ag = ((ar as u16 + g as u16) /2 ) as u8;
            ab = ((ar as u16 + b as u16) /2 ) as u8;
        } else { */
            ar = r;
            ag = g;
            ab = b;
       // }

        if gray > 0 { // avoid drawing off dimensions
            braille.set_colored(
                x, y, PixelColor::TrueColor{r: ar, g: ag, b: ab}
            );
        }
    }
    //queue!(stdout, Clear(ClearType::All));

    for (y, line) in braille.rows().iter().enumerate() {
        queue!(stdout, cursor::MoveTo(center.0, center.1 + y as u16), Print(line));
    }

    //queue!(stdout, cursor::MoveTo(1, 1), Print(braille.frame()));
}
