use minifb::{Key, KeyRepeat, Window, WindowOptions};
use crossterm::{
    queue, QueueableCommand,
    terminal::{Clear, ClearType},
    event::{poll, read, Event, KeyCode},
};

use std::sync::{
    Arc, atomic::{AtomicBool, Ordering},
};

use crate::{
    data::*,
    modes::console_mode::{
        Flusher,
        rescale,
    },
    graphics,
    math,
    visualizers::*
};

pub fn init_window(prog: &Program) -> Result<Window, minifb::Error> {
	let mut win = Window::new(
        "kvis",
        prog.pix.width*prog.SCALE as usize,
        prog.pix.height*prog.SCALE as usize,
        WindowOptions {
            resize: prog.RESIZE,
            topmost: true,
            borderless: false,
            transparency: false,
            scale_mode: minifb::ScaleMode::UpperLeft,
            ..WindowOptions::default()
        },
    )?;

    win.limit_update_rate(Some(prog.REFRESH_RATE));

    Ok(win)
}

pub fn usleep(micros: u64) {
    std::thread::sleep(std::time::Duration::from_micros(micros));
}
/*
pub fn update_size_con(
    prog: &mut Program,
    stdout: &mut std::io::Stdout
) {
    let size = crossterm::terminal::size().unwrap();
    prog.update_con_size(size);
    queue!(stdout, Clear(ClearType::All));
}*/
/*
pub fn change_stdout_func(
    prog: &mut Program,
    stdout: &mut std::io::Stdout
) {
    prog.con_func =
        if prog.con_bool {
            prepare_stdout_ascii
        } else {
            prepare_stdout_braille
        };
    prog.con_bool ^= true;

    update_size_con(prog, stdout);
}
*/
pub fn change_fps(prog: &mut Program, amount: i16, replace: bool) {
    prog.FPS =
        ((prog.FPS * (!replace) as u64) as i16 + amount)
        .clamp(1, 144 as i16)
        as u64
        ;
    prog.REFRESH_RATE = std::time::Duration::from_micros(1_000_000 / prog.FPS);
}
/*
pub fn change_charset(prog: &mut Program) {
    prog.ascii_set =
        if prog.char_bool {
            CHARSET_SIZEOPAC
        } else {
            CHARSET_BLOCKOPAC
        };
    prog.char_bool ^= true;
}
*/

pub fn control_key_events_win(
    win: &mut minifb::Window,
    prog: &mut Program,
) {

    let mut fps_change = false;
    
    prog.update_vis();
    
    
    
    win.get_keys_pressed(KeyRepeat::No).iter().for_each(|key|
        match key {
            Key::Space => prog.change_visualizer(true),

            //~ Key::Key1 =>  { change_fps(prog, 10, true); fps_change = true; },
            //~ Key::Key2 =>  { change_fps(prog, 20, true); fps_change = true; },
            //~ Key::Key3 =>  { change_fps(prog, 30, true); fps_change = true; },
            //~ Key::Key4 =>  { change_fps(prog, 40, true); fps_change = true; },
            //~ Key::Key5 =>  { change_fps(prog, 50, true); fps_change = true; },
            //~ Key::Key6 =>  { change_fps(prog, 60, true); fps_change = true; },

            //~ Key::Key7 =>  { change_fps(prog, -5, false); fps_change = true; },
            //~ Key::Key8 =>  { change_fps(prog,  5, false); fps_change = true; },

            Key::Minus =>   prog.VOL_SCL = (prog.VOL_SCL / 1.2).clamp(0.0, 10.0),
            Key::Equal =>   prog.VOL_SCL = (prog.VOL_SCL * 1.2).clamp(0.0, 10.0),

            Key::LeftBracket =>   prog.SMOOTHING = (prog.SMOOTHING - 0.05).clamp(0.0, 0.95),
            Key::RightBracket =>   prog.SMOOTHING = (prog.SMOOTHING + 0.05).clamp(0.0, 0.95),

            Key::Semicolon =>   prog.WAV_WIN = (prog.WAV_WIN - 3).clamp(3, 50),
            Key::Apostrophe =>  prog.WAV_WIN = (prog.WAV_WIN + 3).clamp(3, 50),

            Key::Backslash =>  prog.AUTO_SWITCH ^= true,

            Key::Slash => {
                prog.VOL_SCL = DEFAULT_VOL_SCL;
                prog.SMOOTHING = DEFAULT_SMOOTHING;
                prog.WAV_WIN = DEFAULT_WAV_WIN;
                // change_fps(prog, 144, true);
            }

            _ => {},
        }
    );

    if fps_change {
        win.limit_update_rate(Some(prog.REFRESH_RATE));
    }
}

pub fn control_key_events_con(
    prog: &mut Program,
    exit: &mut bool
) -> crossterm::Result<()> {	
	prog.update_vis();
	
    if poll(prog.REFRESH_RATE)? {
        match read()? {
            Event::Key(event) => match event.code {
                KeyCode::Char(' ') if event.modifiers == crossterm::event::KeyModifiers::CONTROL
                                   =>   prog.change_visualizer(false),
                KeyCode::Char(' ') =>   prog.change_visualizer(true),
                
                KeyCode::Char('q') =>   *exit = true,

                KeyCode::Char('1') =>   change_fps(prog, 10, true),
                KeyCode::Char('2') =>   change_fps(prog, 20, true),
                KeyCode::Char('3') =>   change_fps(prog, 30, true),
                KeyCode::Char('4') =>   change_fps(prog, 40, true),
                KeyCode::Char('5') =>   change_fps(prog, 50, true),
                KeyCode::Char('6') =>   change_fps(prog, 60, true),

                KeyCode::Char('7') =>   change_fps(prog, -5, false),
                KeyCode::Char('8') =>   change_fps(prog,  5, false),
                KeyCode::Char('9') =>   prog.change_con_max(-1, false),
                KeyCode::Char('0') =>   prog.change_con_max(1, false),

                KeyCode::Char('-') =>   prog.VOL_SCL = (prog.VOL_SCL / 1.2).clamp(0.0, 10.0),
                KeyCode::Char('=') =>   prog.VOL_SCL = (prog.VOL_SCL * 1.2).clamp(0.0, 10.0),

                KeyCode::Char('[') =>   prog.SMOOTHING = (prog.SMOOTHING - 0.05).clamp(0.0, 0.95),
                KeyCode::Char(']') =>   prog.SMOOTHING = (prog.SMOOTHING + 0.05).clamp(0.0, 0.95),

                KeyCode::Char(';') =>   prog.WAV_WIN = (prog.WAV_WIN - 3).clamp(3, 50),
                KeyCode::Char('\'') =>  prog.WAV_WIN = (prog.WAV_WIN + 3).clamp(3, 50),

                KeyCode::Char('\\') =>  prog.AUTO_SWITCH ^= true,

                KeyCode::Char('.') => prog.switch_con_mode(),
                // KeyCode::Char(',') => change_charset(prog),

                KeyCode::Char('/') => {
					prog.VOL_SCL = DEFAULT_VOL_SCL;
					prog.SMOOTHING = DEFAULT_SMOOTHING;
					prog.WAV_WIN = DEFAULT_WAV_WIN;
                    prog.change_con_max(50, true);
                    change_fps(prog, DEFAULT_FPS as i16, true);
                }

                _ => {},
            },
            Event::Resize(w, h) => {
  				prog.update_size((w, h));
                prog.clear_con();
            },
            _ => {},
        }
    }
    Ok(())
}
