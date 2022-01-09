use crate::constants::{INCREMENT, VOL_SCL, WAV_WIN, WIN_W, WIN_H, pi2};
use crate::graphics::graphical_fn::{rgb_to_u32, coord_to_1d, win_clear, win_clear_alpha, draw_line, P2, p2_add};

static mut _i : usize = 0;
static mut wrap_rate_incremeter : f32 = 0.0;

pub unsafe fn draw_flower(buf : &mut Vec<u32>, stream : Vec<(f32, f32)>) {
    
    let range = stream.len()*WAV_WIN/100;

    if range < WIN_H+WIN_W { return (); }

    let size = if WIN_H > WIN_W {WIN_W as i32} else {WIN_H as i32};

    let width = WIN_W as i32;
    let height = WIN_H as i32;

    let width_top_h = width >> 1;
    let height_top_h = height >> 1;


    let mut di = 0;
    
    win_clear(buf);
    
    let rate = pi2 * (1.5 + crate::math::fast_sin(wrap_rate_incremeter))/ range as f32;
    while di < range {
        let p = crate::math::euler(stream[_i % stream.len()], (di as f32 * rate)); 
        let x = (p.0*size as f32 * VOL_SCL) as i32;
        let y = (p.1*size as f32 * VOL_SCL) as i32;
        
        buf[coord_to_1d(x/2+width_top_h, y/2+height_top_h)] = rgb_to_u32(128, (x.abs()*510/size as i32) as u8, (y.abs()*510/size as i32) as u8);

        _i = (_i+INCREMENT+1) % stream.len();
        di = di+INCREMENT+1;
    }
    
    wrap_rate_incremeter += 0.001;
    if (wrap_rate_incremeter > pi2) {
        wrap_rate_incremeter = 0.0;
    }
}
