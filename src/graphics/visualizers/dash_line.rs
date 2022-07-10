// THis visualzier is not intended to display on its own.

use crate::config::{PHASE_OFFSET, INCREMENT};
use crate::graphics::graphical_fn::{rgb_to_u32, coord_to_1d, win_clear, win_clear_alpha, draw_line, P2, p2_add};
use crate::graphics::graphical_fn;

static mut _i : usize = 0;
const c : u32 = 16720064;

pub fn draw_dash_line(buf : &mut [u32], stream : &[(f32, f32)], horizontal : bool, offset : usize, flip_side : bool, para: &mut crate::config::Parameters) {

    if (horizontal) {

        let o = if flip_side {
            para.WIN_H - offset -1
        } else {
            offset
        };

        for i in 0..para.WIN_W {
            let index = ((para._i+i)/10)%stream.len();
            graphical_fn::draw_rect(buf, i, o, 2, 2, graphical_fn::apply_alpha(c, (stream[index].0 * 32784.0 % 256.0) as i32 as u8), para);
        }
    } else {

        let o = if flip_side {
            para.WIN_W - offset -1
        } else {
            offset
        };

        for i in 0..para.WIN_H {
            let index = ((para._i+i)/10)%stream.len();
            graphical_fn::draw_rect(buf, o, i, 2, 2, graphical_fn::apply_alpha(c, (stream[index].0 * 32784.0 % 256.0) as i32 as u8), para);
        }
    }

    para._i = (para._i + INCREMENT)%para.WIN_H;
}
