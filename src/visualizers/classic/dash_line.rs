// This visualizer is not intended to display on its own.

use crate::graphics::P2;

const C: u32 = 0x00_FF_20_C0;

pub fn draw_dash_line(
    para: &mut crate::Program,
    stream: &mut crate::AudioBuffer,
    horizontal: bool,
    offset: usize,
    flip_side: bool,
) {
    para.pix.mixerd();

    if horizontal {
        let o = if flip_side {
            para.pix.height() - offset - 1
        } else {
            offset
        };

        for i in 0..para.pix.width() {
            let index = (i / 10 + i) % stream.len();
            para.pix.color(C | (((stream.get(index).1 * 32784.0 % 256.0) as u32) << 24));
            para.pix.rect(
                P2(i as i32, o as i32),
                2,
                2,
            );
        }
    } else {
        let o = if flip_side {
            para.pix.width() - offset - 1
        } else {
            offset
        };

        for i in 0..para.pix.height() {
            let index = (i / 10 + i) % stream.len();
            para.pix.color(C | (((stream.get(index).1 * 32784.0 % 256.0) as u32) << 24));
            para.pix.rect(
                P2(o as i32, i as i32),
                2,
                2,
            );
        }
    }
}
