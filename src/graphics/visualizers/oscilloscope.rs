use crate::constants::{PHASE_OFFSET, INCREMENT, VOL_SCL, WAV_WIN, console_clear, WIN_W, WIN_H, SMOOTHING};

use crate::graphics::graphical_fn::{rgb_to_u32, coord_to_1d, win_clear, win_clear_alpha, draw_line, P2, p2_add, draw_rect};

static mut i : usize = 0;
static mut _smooth_osc : [i32; WIN_W] = [0; WIN_W];

#[allow(dead_code)]
pub unsafe fn draw_oscilloscope(buf : &mut Vec<u32>, stream : Vec<(f32, f32)>) {
    let range = stream.len()*WAV_WIN/100;

    if range < WIN_H+WIN_W { return (); }

    let width = WIN_W as i32;
    let height = WIN_H as i32;

    let width_top_h = (width >> 1) + 1;
    let height_top_h = (height >> 1) + 1;


    let mut di = 0;

    win_clear(buf);

    //i = i%range;

    while di < range {
        let x = (di * width as usize / range) as i32;
        let xu = x as usize;
        
        let y = height_top_h + (stream[i%stream.len()].0*WIN_H as f32 *VOL_SCL *0.5) as i32;
        
        
        
        //let o = (y.abs()*4*PXL_OPC/width) as usize;

        _smooth_osc[xu] = crate::math::interpolate::lineari(_smooth_osc[xu], y, 64);

        //buf[coord_to_1d(x, y)] = rgb_to_u32(255, 255, 255);
        
        buf[coord_to_1d(x, _smooth_osc[xu])] = rgb_to_u32(255, 255, 255);

        i = (i+INCREMENT+1);
        di = di+INCREMENT+1;
    }
        
    i %= stream.len();
    
    draw_rect(buf, i%WIN_W, 8, 1, WIN_H-16, 0x00_55_55_55);
    
    draw_rect(buf, 0, WIN_H/2, WIN_W, 1, 0x00_55_55_55);
}

static mut grid: bool = false;
pub unsafe fn draw_vectorscope(buf : &mut Vec<u32>, stream : Vec<(f32, f32)>) {

    let range = stream.len()*WAV_WIN/100;

    if range < WIN_H+WIN_W { return (); }

    let size = if WIN_H > WIN_W {WIN_W as i32} else {WIN_H as i32};

    let width = WIN_W as i32;
    let height = WIN_H as i32;

    let width_top_h = width >> 1;
    let height_top_h = height >> 1;


    let mut di = 0;

    win_clear(buf);

    while di < range {
        let x =  (stream[i%stream.len()].0 *size as f32 *VOL_SCL * 0.5) as i32;
        let y =  (stream[(i+PHASE_OFFSET)%stream.len()].0 *size as f32 *VOL_SCL * 0.5) as i32;
        //let o = ((x.abs() + y.abs()*2)*PXL_OPC/width) as usize;
        // Using this instead of the circle formula. Creates a diamond/rhombus shape.

        buf[coord_to_1d(x+width_top_h, y+height_top_h)] = rgb_to_u32((x.abs()*510/size as i32) as u8, 255, (y.abs()*510/size as i32) as u8);

        i = (i+INCREMENT+1);
        di = di+INCREMENT+1;
    }
    
    if {grid ^= true; grid} {
        draw_rect(buf, WIN_W/2, 8, 1, WIN_H-16, 0x00_55_55_55);
    } else {
        draw_rect(buf, 8, WIN_H/2, WIN_W-16, 1, 0x00_55_55_55);
    }
    
    i %= stream.len();
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
