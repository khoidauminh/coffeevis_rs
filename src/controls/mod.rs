use minifb::{Key, KeyRepeat, Window, WindowOptions};
use crossterm::{
    queue, QueueableCommand,
    terminal::{Clear, ClearType},
    event::{poll, read, Event, KeyCode},
};

use crate::{
    config::*,
    math,
    console_mode::{
        OutFunc,
        prepare_stdout_ascii,
        prepare_stdout_braille,
        CHARSET_BLOCKOPAC,
        CHARSET_SIZEOPAC,
        rescale,
    },
    graphics::{
        graphical_fn::{self, update_size},
        visualizers::{*, VisFunc}
    }
};

pub fn init_window(para: &Parameters) -> Result<Window, minifb::Error> {
	let mut win = Window::new(
        "Coffee Visualizer",
        para.WIN_W,
        para.WIN_H,
        WindowOptions {
            scale: minifb::Scale::X1,
            resize: true,
            topmost: true,
            borderless: false,
            scale_mode: minifb::ScaleMode::UpperLeft,
            ..WindowOptions::default()
        },
    )?;

    win.limit_update_rate(Some(std::time::Duration::from_millis(para.FPS_ITVL)));

    Ok(win)
}

pub fn usleep(micros: u64) {
    std::thread::sleep(std::time::Duration::from_micros(micros));
}

pub fn change_visualizer(
    pix: &mut Vec<u32>,
    para: &mut Parameters,
) {
    para.VIS_IDX = math::advance_with_limit(para.VIS_IDX, VISES);
    *pix = vec![0u32; para.WIN_R];

    para.vis_func = match para.VIS_IDX {
        1 => oscilloscope::draw_oscilloscope,
        2 => ring::draw_ring,
        3 => spectrum::draw_spectrum,
        4 => bars::draw_bars,
        5 => bars::draw_bars_circle,
        6 => shaky_coffee::draw_shaky,
        7 => vol_sweeper::draw_vol_sweeper,
        8 => lazer::draw_lazer,
        9 => experiment1::draw_exp1,
        10 => experiment1::draw_f32,
        11 => wave::draw_wave,

        _ => oscilloscope::draw_vectorscope,
    };
}

pub fn update_size_con(
    para: &mut Parameters,
    pix: &mut Vec<u32>,
    stdout: &mut std::io::Stdout
) {
    let size = rescale(crossterm::terminal::size().unwrap(), para);
    update_size(size, para);
    pix.resize(para.WIN_R, 0);
    queue!(stdout, Clear(ClearType::All));
}

pub fn change_stdout_func(
    pix: &mut Vec<u32>,
    para: &mut Parameters,
    stdout: &mut std::io::Stdout
) {
    para.con_func =
        if para.con_bool {
            prepare_stdout_ascii
        } else {
            prepare_stdout_braille
        };
    para.con_bool ^= true;

    update_size_con(para, pix, stdout);
}

pub fn change_fps(para: &mut Parameters, amount: i16, replace: bool) {
    para.FPS =
        ((para.FPS * (!replace) as u64) as i16 + amount)
        .clamp(1, 60 as i16)
        as u64
        ;
    para.FPS_ITVL = 1000 / para.FPS;
    para.AUTO_SWITCH_ITVL = para.FPS as u16 *8;
}

pub fn change_charset(para: &mut Parameters) {
    para.ascii_set =
        if para.char_bool {
            &CHARSET_SIZEOPAC
        } else {
            &CHARSET_BLOCKOPAC
        };
    para.char_bool ^= true;
}

pub fn change_con_max(
    para: &mut Parameters,
    amount: i16,
    replace: bool,
    pix: &mut Vec<u32>,
    stdout: &mut std::io::Stdout
) {
    para.CON_MAX_W =
        ((para.CON_MAX_W * (!replace) as u16) as i16 + amount)
        .clamp(0, para.WIN_W as i16)
        as u16
        ;
    para.CON_MAX_H = para.CON_MAX_W >> 1;
    update_size_con(para, pix, stdout);
}

pub fn control_key_events_win(
    win: &mut minifb::Window,
    pix: &mut Vec<u32>,
    para: &mut Parameters,
) {
    /*

    if win.is_key_pressed(Key::Space, KeyRepeat::Yes) {
        change_visualizer(pix, para);
    }

    if win.is_key_pressed(Key::Minus, KeyRepeat::Yes) {
        para.VOL_SCL = (para.VOL_SCL / 1.2).clamp(0.0, 10.0);
    } else if win.is_key_pressed(Key::Equal, KeyRepeat::Yes) {
        para.VOL_SCL = (para.VOL_SCL * 1.2).clamp(0.0, 10.0);
    }

    if win.is_key_pressed(Key::LeftBracket, KeyRepeat::Yes) {
        para.SMOOTHING = (para.SMOOTHING - 0.05).clamp(0.0, 0.95);
    } else if win.is_key_pressed(Key::RightBracket, KeyRepeat::Yes) {
        para.SMOOTHING = (para.SMOOTHING + 0.05).clamp(0.0, 0.95);
    }

    if win.is_key_pressed(Key::Semicolon, KeyRepeat::Yes) {
        para.WAV_WIN = (para.WAV_WIN - 3).clamp(3, 50);
    } else if win.is_key_pressed(Key::Apostrophe, KeyRepeat::Yes) {
        para.WAV_WIN = (para.WAV_WIN + 3).clamp(3, 50);
    }

    if win.is_key_pressed(Key::Backslash, KeyRepeat::No) {
        para.AUTO_SWITCH ^= 1;
    }

    if win.is_key_pressed(Key::Slash, KeyRepeat::Yes) {
        para.VOL_SCL = 0.85;
        para.SMOOTHING = 0.65;
        para.WAV_WIN = 30;
    } */

    let mut fps_change = false;

    win.get_keys_pressed(KeyRepeat::No).iter().for_each(|key|
        match key {
            Key::Space =>   change_visualizer(pix, para),

            Key::Key1 =>  { change_fps(para, 10, true); fps_change = true; },
            Key::Key2 =>  { change_fps(para, 20, true); fps_change = true; },
            Key::Key3 =>  { change_fps(para, 30, true); fps_change = true; },
            Key::Key4 =>  { change_fps(para, 40, true); fps_change = true; },
            Key::Key5 =>  { change_fps(para, 50, true); fps_change = true; },
            Key::Key6 =>  { change_fps(para, 60, true); fps_change = true; },

            Key::Key7 =>  { change_fps(para, -5, false); fps_change = true; },
            Key::Key8 =>  { change_fps(para,  5, false); fps_change = true; },

            Key::Minus =>   para.VOL_SCL = (para.VOL_SCL / 1.2).clamp(0.0, 10.0),
            Key::Equal =>   para.VOL_SCL = (para.VOL_SCL * 1.2).clamp(0.0, 10.0),

            Key::LeftBracket =>   para.SMOOTHING = (para.SMOOTHING - 0.05).clamp(0.0, 0.95),
            Key::RightBracket =>   para.SMOOTHING = (para.SMOOTHING + 0.05).clamp(0.0, 0.95),

            Key::Semicolon =>   para.WAV_WIN = (para.WAV_WIN - 3).clamp(3, 50),
            Key::Apostrophe =>  para.WAV_WIN = (para.WAV_WIN + 3).clamp(3, 50),

            Key::Backslash =>  para.AUTO_SWITCH ^= 1,


            Key::Slash => {
                para.VOL_SCL = 0.85;
                para.SMOOTHING = 0.65;
                para.WAV_WIN = 25;
            }

            _ => {},
        }
    );

    if fps_change {
        win.limit_update_rate(Some(std::time::Duration::from_millis(para.FPS_ITVL)));
    }

    //~ if win.is_key_pressed(Key::Backspace, KeyRepeat::Yes) {
    //~ unsafe {
    //~ graphical_fn::update_size((144, 144), para);
    //~ }
    //~ }
}

pub fn control_key_events_con(
    para: &mut Parameters,
    pix: &mut Vec<u32>,
    stdout: &mut std::io::Stdout,
    exit: &mut bool
) -> crossterm::Result<()> {
    if poll(std::time::Duration::from_millis(para.FPS_ITVL))? {
        match read()? {
            Event::Key(event) => match event.code {
                KeyCode::Char(' ') =>   change_visualizer(pix, para),
                KeyCode::Char('q') =>   *exit = true,

                KeyCode::Char('1') =>   change_fps(para, 10, true),
                KeyCode::Char('2') =>   change_fps(para, 20, true),
                KeyCode::Char('3') =>   change_fps(para, 30, true),
                KeyCode::Char('4') =>   change_fps(para, 40, true),
                KeyCode::Char('5') =>   change_fps(para, 50, true),
                KeyCode::Char('6') =>   change_fps(para, 60, true),

                KeyCode::Char('7') =>   change_fps(para, -5, false),
                KeyCode::Char('8') =>   change_fps(para,  5, false),
                KeyCode::Char('9') =>   change_con_max(para, -1, false, pix, stdout),
                KeyCode::Char('0') =>   change_con_max(para,  1, false, pix, stdout),

                KeyCode::Char('-') =>   para.VOL_SCL = (para.VOL_SCL / 1.2).clamp(0.0, 10.0),
                KeyCode::Char('=') =>   para.VOL_SCL = (para.VOL_SCL * 1.2).clamp(0.0, 10.0),

                KeyCode::Char('[') =>   para.SMOOTHING = (para.SMOOTHING - 0.05).clamp(0.0, 0.95),
                KeyCode::Char(']') =>   para.SMOOTHING = (para.SMOOTHING + 0.05).clamp(0.0, 0.95),

                KeyCode::Char(';') =>   para.WAV_WIN = (para.WAV_WIN - 3).clamp(3, 50),
                KeyCode::Char('\'') =>  para.WAV_WIN = (para.WAV_WIN + 3).clamp(3, 50),

                KeyCode::Char('\\') =>  para.AUTO_SWITCH ^= 1,

                KeyCode::Char('.') => change_stdout_func(pix, para, stdout),
                KeyCode::Char(',') => change_charset(para),

                KeyCode::Char('/') => {
                    para.VOL_SCL = 0.85;
                    para.SMOOTHING = 0.65;
                    para.WAV_WIN = 25;
                    change_con_max(para, 50, true, pix, stdout);
                }

                _ => {},
            },
            Event::Resize(width, height) => {
                let size = rescale((width, height), para);
                update_size(size, para);
                pix.resize(para.WIN_R, 0);
                queue!(stdout, Clear(ClearType::All));
            },
            _ => {},
        }
    }
    Ok(())
}
