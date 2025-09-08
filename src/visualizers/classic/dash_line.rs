// This visualizer is not intended to display on its own.

use crate::graphics::{P2, Pixel};

const C: u32 = 0x00_FF_20_C0;

pub fn draw_dash_line(
    para: &mut crate::Program,
    stream: &mut crate::AudioBuffer,
    horizontal: bool,
    offset: usize,
    flip_side: bool,
) {
    if horizontal {
        let o = if flip_side {
            para.pix.height() - offset - 1
        } else {
            offset
        };

        for i in 0..para.pix.width() {
            let index = (i / 10 + i) % stream.len();
            para.pix.rect_wh(
                P2::new(i, o),
                2,
                2,
                C | (((stream.get(index).y * 32784.0 % 256.0) as u32) << 24),
                Pixel::over,
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
            para.pix.rect_wh(
                P2::new(o, i),
                2,
                2,
                C | (((stream.get(index).y * 32784.0 % 256.0) as u32) << 24),
                Pixel::over,
            );
        }
    }
}
