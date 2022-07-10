use crate::config::Parameters;
use crate::config::{INCREMENT, PHASE_OFFSET, SAMPLE_SIZE};

use crate::graphics::{
    visualizers::cross::CROSS_COL,
    graphical_fn::{
        coord_to_1d, draw_line, draw_rect,
        flatten, p2_add, rgb_to_u32, win_clear,
        win_clear_alpha, P2,
    }
};


// static mut _smooth_osc : [f32; WIN_W] = [0.0f32; WIN_W];

#[allow(dead_code)]
pub fn draw_oscilloscope(buf: &mut [u32], stream: &[(f32, f32)], para: &mut Parameters) {
    let range = stream.len() * para.WAV_WIN / 100;

    //if range < WIN_H+WIN_W { return (); }

	// let incr = INCREMENT >> 2;

    let width = para.WIN_W as i32;
    let height = para.WIN_H as i32;

    let width_top_h = (width >> 1) + 1;
    let height_top_h = (height >> 1) + 1;

    let mut di = 0;

    win_clear(buf);

    //i = i%range;

    para._i %= stream.len();

    while di < range {
        let x = (di * width as usize / range) as i32;
        let xu = x as usize;

        let y1 = height_top_h + (stream[para._i].0 * para.WIN_H as f32 * para.VOL_SCL) as i32 / 2;
        let y2 = height_top_h + (stream[para._i].1 * para.WIN_H as f32 * para.VOL_SCL) as i32 / 2;

        //let o = (y.abs()*4*PXL_OPC/width) as usize;

        //_smooth_osc[xu] = crate::math::interpolate::linearf(_smooth_osc[xu], y, 0.7);

        //buf[coord_to_1d(x, y)] = rgb_to_u32(255, 255, 255);
        /*
        let brightness1 = y1 / height_top_h * 256;
        let brightness2 = y1 / height_top_h * 256;
       */

        buf[flatten(x, y1, para.WIN_W, para.WIN_H)] = 0x00_FF_55_55;
        buf[flatten(x, y2, para.WIN_W, para.WIN_H)] = 0x00_55_55_FF;



        para._i = (para._i + INCREMENT + 1) % stream.len();
        di = di + INCREMENT + 1;
    }

    draw_rect(
        buf,
        para._i % para.WIN_W,
        para.WIN_H / 10,
        2,
        para.WIN_H - para.WIN_H / 5,
        CROSS_COL,
        para,
    );

    para._i %= stream.len();

    draw_rect(buf, 0, para.WIN_H / 2, para.WIN_W, 2, CROSS_COL, para);
}

pub fn draw_vectorscope(buf: &mut [u32], stream: &[(f32, f32)], para: &mut Parameters) {
    let range = stream.len() * para.WAV_WIN / 100;

    //if range < WIN_H+WIN_W { return (); }

    // let incr = INCREMENT >> 2;

    let size = para.WIN_H.min(para.WIN_W) as i32;

    let width = para.WIN_W as i32;
    let height = para.WIN_H as i32;

    let width_top_h = width >> 1;
    let height_top_h = height >> 1;

    let mut di = 0;

    win_clear(buf);

    while di < range {
        let x = (stream[para._i % stream.len()].0 * size as f32 * para.VOL_SCL * 0.5) as i32;
        let y =
            (stream[(para._i + PHASE_OFFSET) % stream.len()].1 * size as f32 * para.VOL_SCL * 0.5)
                as i32;

        buf[flatten(x + width_top_h, y + height_top_h, para.WIN_W, para.WIN_H)] = rgb_to_u32(
            (x.abs() * 510 / size as i32) as u8,
            255,
            (y.abs() * 510 / size as i32) as u8,
        );

        para._i = (para._i + INCREMENT);
        di = di + INCREMENT;
    }

    crate::graphics::visualizers::cross::draw_cross(buf, para);

    para._i %= stream.len();
}

// Attempting to write the oscilloscope visualizer with the linear function.
// Not finished
/*
static mut p1 : P2 = P2(0, 0);
pub unsafe fn draw_vectorscope_(buf : &mut Vec<u32>, stream : Vec<i32>) {

    let range = stream.len()*WAV_WIN/100;

    if range < WIN_H+WIN_W { return (); }

    let size = if WIN_H > WIN_W {WIN_W as i32} else {WIN_H as i32};

    let width = WIN_W as i32;
    let height = WIN_H as i32;

    let width_top_h = width/2;
    let height_top_h = height/2;

    let dx = 32767/size;
    let dy = 32767/size;

    let mut di = 0;

    let center = P2(width_top_h, height_top_h);

    win_clear(buf);

    while di < range {
        let p = P2((stream[i%stream.len()]/dx)*VOL_SCL/100, (stream[(i+PHASE_OFFSET)%stream.len()]/dy)*VOL_SCL/100);

        let color = rgb_to_u32((p.0.abs()*510/size as i32) as u8, 255, (p.1.abs()*510/size as i32) as u8);

        draw_line(buf, p2_add(center, p1), p2_add(center, p), color, 1.0);

        p1 = p;
        i = (i+3*(INCREMENT+1)) % stream.len();
        di = di+3*(INCREMENT+1);
    }
}*/
