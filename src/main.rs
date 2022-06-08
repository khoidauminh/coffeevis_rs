//use core::time::Duration;
#![allow(warnings)]

mod constants;
use crate::constants::*;

// Audio lib
mod audio_input;
use audio_input::input_stream::{get_source, read_samples};
use cpal;
use cpal::traits::StreamTrait;

mod math;

mod graphics;
use graphics::{
    graphical_fn,
    visualizers::{
        bars, experiment1, lazer, oscilloscope, ring, shaky_coffee, spectrum, vol_sweeper,
    },
};

//mod assets;
use minifb::{Key, KeyRepeat, Window, WindowOptions};

static mut buf: [(f32, f32); SAMPLE_SIZE] = [(0.0f32, 0.0f32); SAMPLE_SIZE];

//static mut i : usize = 0;
//static mut visualizer : (dyn Fn(Vec<u32>, Vec<(f32, f32)>)) = &oscilloscope::draw_vectorscope;

fn main() {
    let coffee_pixart_file = include_bytes!("coffee_pixart_2x.ppm");
    //~ let coffee_pixart_file = include_bytes!("/home/khoidauminh/Pictures/scared_Đức_.ppm");

    let mut para = Parameters {
        SWITCH: 0,
        AUTO_SWITCH: 1,
        SWITCH_INCR: 0,
        VIS_IDX: 0,

        WAV_WIN: 30,
        VOL_SCL: 0.8,
        SMOOTHING: 0.5,

        WIN_W: 96,
        WIN_H: 96,
        WIN_R: 96 * 96,
        WIN_RL: 96 * 96 - 1,

        IMG: graphics::visualizers::shaky_coffee::prepare_img(coffee_pixart_file),

        _i: 0,

        cross: false,

        bars_smoothing_ft: vec![0.0f32; 37],

        bars_smoothing_ft_circle: vec![0.0f32; 144],

        shaky_coffee: (0.0, 0.0, 0.0, 0.0, 0, 0),

        vol_sweeper: (0, 0),

        lazer: (0, 0, 0, 0, false),

        spectrum_smoothing_ft: vec![(0.0, 0.0); FFT_SIZE + 1],
    };

    let mut stream;

    unsafe {
        stream = get_source(read_samples::<f32>, &mut para);
    }
    let status = stream.play();

    let mut win = match Window::new(
        "Coffee Visualizer",
        para.WIN_W,
        para.WIN_H,
        WindowOptions {
            scale: minifb::Scale::X1,
            resize: true,
            topmost: true,
            borderless: false,
            ..WindowOptions::default()
        },
    ) {
        Ok(win) => win,
        Err(err) => {
            println!("Unable to create window {}", err);
            return;
        }
    };

    win.limit_update_rate(Some(std::time::Duration::from_micros(FPS_ITVL)));
    //spectrum::prepare_index();
    //bars::prepare_index_bar();
    //crate::constants::prepare_table();
    //bars::data_f1.resize(bars::bar_num+1, 0.0);

    let mut visualizer: fn(&mut [u32], &[(f32, f32)], para: &mut Parameters) -> () =
        oscilloscope::draw_vectorscope;
    //~ let mut visualizer: fn(&mut [u32], &[(f32, f32)], para: &mut Parameters) -> () = bars::draw_bars_circle;
    //change_visualizer(&mut pix_buf, &mut visualizer);

    let mut pix_buf: Vec<u32> = vec![0u32; para.WIN_R];

    while win.is_open() && !win.is_key_down(Key::Escape) {
        let s = win.get_size();
        if s.0 != para.WIN_W || s.1 != para.WIN_H {
            graphical_fn::update_size(s, &mut para);
            pix_buf = vec![0u32; s.0 * s.1];
        }

        if para.SWITCH_INCR == AUTO_SWITCH_ITVL {
            change_visualizer(&mut pix_buf, &mut visualizer, &mut para);
            para.SWITCH_INCR = 0;
        }

        let stream_data = unsafe { buf };
        visualizer(&mut pix_buf, &stream_data, &mut para);
        win.update_with_buffer(&pix_buf, para.WIN_W, para.WIN_H);

        control_key_events(&win, &mut pix_buf, &mut visualizer, &mut para);

        usleep(FPS);

        para.SWITCH_INCR += para.AUTO_SWITCH;
    }
    stream.pause();
}

fn change_visualizer(
    pix: &mut Vec<u32>,
    f: &mut fn(&mut [u32], &[(f32, f32)], para: &mut Parameters) -> (),
    para: &mut Parameters,
) {
    para.VIS_IDX = math::advance_with_limit(para.VIS_IDX, VISES);
    *pix = vec![0u32; para.WIN_R];

    *f = match para.VIS_IDX {
        1 => shaky_coffee::draw_shaky,
        2 => vol_sweeper::draw_vol_sweeper,
        3 => spectrum::draw_spectrum,
        4 => oscilloscope::draw_oscilloscope,
        5 => lazer::draw_lazer,
        6 => ring::draw_ring,
        7 => bars::draw_bars,
        8 => experiment1::draw_exp1,
        9 => bars::draw_bars_circle,
        10 => experiment1::draw_f32,
        _ => oscilloscope::draw_vectorscope,
    };
}

fn control_key_events(
    win: &minifb::Window,
    pix: &mut Vec<u32>,
    f: &mut fn(&mut [u32], &[(f32, f32)], para: &mut Parameters) -> (),
    para: &mut Parameters,
) {
    if win.is_key_pressed(Key::Space, KeyRepeat::Yes) {
        change_visualizer(pix, f, para);
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
    }

    //~ if win.is_key_pressed(Key::Backspace, KeyRepeat::Yes) {
    //~ unsafe {
    //~ graphical_fn::update_size((144, 144), para);
    //~ }
    //~ }
}
