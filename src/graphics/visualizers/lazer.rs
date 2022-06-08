use crate::graphics::graphical_fn::{rgb_to_u32, coord_to_1d, win_clear, win_clear_alpha, draw_rect, P2, draw_line_direct};
use crate::constants::{PHASE_OFFSET, INCREMENT};
use crate::constants::Parameters;

static mut pos : P2 = P2(72, 72);
static mut p1 : P2 = P2(72, 72);
static mut swap : bool = false;

pub fn draw_lazer(buf : &mut [u32], stream : &[(f32, f32)], para: &mut Parameters) {

    let w = para.WIN_W as i32;
    let h = para.WIN_H as i32;

    let mut ax = {
        let mut sum : f32 = 0.0;
        for i in 1..stream.len()/2 {
            sum += stream[i].0.abs()-stream[i-1].0.abs()*0.6;
        }
        (sum/(stream.len() as f32 / 128.0 * stream[0].0.signum()) * para.VOL_SCL as f32) as i32
    };

    let mut ay = {
        let mut sum : f32 = 0.0;
        for i in stream.len()/2+1..stream.len() {
            sum += stream[i].0.abs()-stream[i-1].0.abs()*0.6;
        }
        (sum/(stream.len() as f32 / 128.0 * stream[stream.len()/2].0.signum()) * para.VOL_SCL as f32) as i32
    };
    

    //~ pos.0 = linear_interp(pos.0 as f32, (pos.0+ax).clamp(0, w -1) as f32, 0.7) as i32;
    //~ pos.1 = linear_interp(pos.1 as f32, (pos.1+ay).clamp(0, h -1) as f32, 0.7) as i32;
    
    // no smoothing version 
    //~ pos.0 = (pos.0+ax).clamp(0, w -1);
    //~ pos.1 = (pos.1+ay).clamp(0, h -1);
    
    para.lazer.0 = (para.lazer.0+ax+w) %w;
    para.lazer.1 = (para.lazer.1+ay+h) %h;

    let color = rgb_to_u32(
        (48 + para.lazer.0*255/w).min(255) as u8, 
        (48 + para.lazer.1*255/h).min(255) as u8,
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

    if (para.lazer.2 == para.lazer.0 && para.lazer.2 == para.lazer.1) {
        buf[coord_to_1d(para.lazer.0, para.lazer.1, para)] = color;
    } else {
        draw_line_direct(buf, P2(para.lazer.2, para.lazer.3), P2(para.lazer.0, para.lazer.1), color, para);
    }

    (para.lazer.2, para.lazer.3) = (para.lazer.0, para.lazer.1);

    //buf[coord_to_1d(pos.0, pos.1)] = color;

    //para._i = (para._i + INCREMENT) % stream.len();

}
