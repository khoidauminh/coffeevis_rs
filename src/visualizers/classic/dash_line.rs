// THis visualzier is not intended to display on its own.


use crate::graphics::P2;

const C: u32 = 0x00_FF_20_C0;

pub fn draw_dash_line(
    para: &mut crate::data::Program,
    stream: &mut crate::audio::SampleArr,
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
            para.pix.draw_rect_wh(
                P2::new(i, o),
                2,
                2,
                C | (((stream[index].y * 32784.0 % 256.0) as u32) << 24),
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
            para.pix.draw_rect_wh(
                P2::new(o, i),
                2,
                2,
                C | (((stream[index].y * 32784.0 % 256.0) as u32) << 24),
            );
        }
    }
}
