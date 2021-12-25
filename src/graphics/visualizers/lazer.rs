use crate::graphics::graphical_fn::{rgb_to_u32, coord_to_1d, win_clear, win_clear_alpha, draw_rect, P2, linear_interp, draw_line_direct};
use crate::constants::{PHASE_OFFSET, INCREMENT, VOL_SCL, WAV_WIN, console_clear, WIN_W, WIN_H};

static mut pos : P2 = P2(72, 72);
static mut p1 : P2 = P2(72, 72);
static mut p2 : P2 = P2(72, 72);
static mut index : usize = 0;
static mut swap : bool = false;

pub unsafe fn draw_lazer(buf : &mut Vec<u32>, stream : Vec<(f32, f32)>) {

    let w = WIN_W as i32;
    let h = WIN_H as i32;

    let mut ax = {
        let mut sum : f32 = 0.0;
        for i in 1..stream.len()/2 {
            sum += stream[i].0.abs()-stream[i-1].0.abs()*0.6;
        }
        (sum/(stream.len() as f32 / 128.0 * stream[0].0.signum()) * VOL_SCL as f32) as i32
    };

    let mut ay = {
        let mut sum : f32 = 0.0;
        for i in stream.len()/2+1..stream.len() {
            sum += stream[i].0.abs()-stream[i-1].0.abs()*0.6;
        }
        (sum/(stream.len() as f32 / 128.0 * stream[stream.len()/2].0.signum()) * VOL_SCL as f32) as i32
    };
    

    //~ pos.0 = linear_interp(pos.0 as f32, (pos.0+ax).clamp(0, w -1) as f32, 0.7) as i32;
    //~ pos.1 = linear_interp(pos.1 as f32, (pos.1+ay).clamp(0, h -1) as f32, 0.7) as i32;
    
    // no smoothing version 
    //~ pos.0 = (pos.0+ax).clamp(0, w -1);
    //~ pos.1 = (pos.1+ay).clamp(0, h -1);
    
    pos.0 = (pos.0+ax+w) %w;
    pos.1 = (pos.1+ay+h) %h;

    let color = rgb_to_u32(
        (48 + pos.0*255/w).min(255) as u8, 
        (48 + pos.1*255/h).min(255) as u8,
        ((255-ax*ay*2).abs().min(255)) as u8
    );

    win_clear_alpha(buf, 0.999);
/*
    pos.0 = (pos.0+ax);
    pos.1 = (pos.1+ay);

    quad_bezier(buf, p2, p1, pos, color);

    pos.0 = (pos.0+w) %w;
    pos.1 = (pos.1+h) %h;

    p2 = p1;
    p1 = pos;
*/

    //pos.0 = (pos.0+ax+w) %w;
    //pos.1 = (pos.1+ay+h) %h;

    if (p1.0 == pos.0 && p1.1 == pos.1) {
        buf[coord_to_1d(pos.0, pos.1)] = color;
    } else {
        draw_line_direct(buf, p1, pos, color);
    }

    p1 = pos;

    //buf[coord_to_1d(pos.0, pos.1)] = color;

    index += INCREMENT % stream.len();

}
