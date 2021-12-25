// THis visualzier is not intended to display on its own.

use crate::constants::{PHASE_OFFSET, INCREMENT, VOL_SCL, WAV_WIN, console_clear, WIN_W, WIN_H};
use crate::graphics::graphical_fn::{rgb_to_u32, coord_to_1d, win_clear, win_clear_alpha, draw_line, P2, p2_add};
use crate::graphics::graphical_fn;

static mut _i : usize = 0;
const c : u32 = 16720064;

pub unsafe fn draw_dash_line(buf : &mut Vec<u32>, stream : &Vec<(f32, f32)>, horizontal : bool, offset : usize, flip_side : bool) {

    if (horizontal) {

        let o = if flip_side {
            WIN_H - offset -1
        } else {
            offset
        };

        for i in 0..WIN_W {
            let index = ((_i+i)/10)%stream.len();
            graphical_fn::draw_rect(buf, i, o, 2, 2, graphical_fn::apply_alpha(c, (stream[index].0 * 32784.0 % 256.0) as i32 as u8));
        }
    } else {

        let o = if flip_side {
            WIN_W - offset -1
        } else {
            offset
        };

        for i in 0..WIN_H {
            let index = ((_i+i)/10)%stream.len();
            graphical_fn::draw_rect(buf, o, i, 2, 2, graphical_fn::apply_alpha(c, (stream[index].0 * 32784.0 % 256.0) as i32 as u8));
        }
    }

    _i = (_i + INCREMENT)%WIN_H;
}
