// THis visualzier is not intended to display on its own.

use crate::data::{INCREMENT, PHASE_OFFSET};
use crate::math::Cplx;
use crate::graphics::{blend::Blend, P2};

const c: u32 = 16720064;

pub fn draw_dash_line(
    para: &mut crate::data::Program,
    stream: &mut crate::audio::SampleArr,
    horizontal: bool,
    offset: usize,
    flip_side: bool,
) {
    if (horizontal) {
        let o = if flip_side {
            para.pix.height - offset - 1
        } else {
            offset
        };

        for i in 0..para.pix.width {
            let index = (i / 10 + i) % stream.len();
            para.pix.draw_rect_wh(
                P2::new(i as i32, o as i32),
                2,
                2,
                c.fade((stream[index].y * 32784.0 % 256.0) as i32 as u8),
            );
        }
    } else {
        let o = if flip_side {
            para.pix.width - offset - 1
        } else {
            offset
        };

        for i in 0..para.pix.height {
            let index = (i / 10 + i) % stream.len();
            para.pix.draw_rect_wh(
                P2::new(o as i32, i as i32),
                2,
                2,
                c.fade((stream[index].y * 32784.0 % 256.0) as i32 as u8),
            );
        }
    }
}
