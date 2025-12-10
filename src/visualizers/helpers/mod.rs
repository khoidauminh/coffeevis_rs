use crate::graphics::{P2, Pixel, PixelBuffer};

pub const CROSS_COL: u32 = 0xFF_44_44_44;

static CROSS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn draw_cross(pix: &mut PixelBuffer) {
    let P2(width, height) = pix.size();

    pix.color(CROSS_COL);
    pix.mixer(u32::over);

    if CROSS.fetch_xor(true, std::sync::atomic::Ordering::Relaxed) {
        pix.rect(
            P2(width / 2, height / 10),
            1,
            (height - height / 5 + 1) as usize,
        );
    } else {
        pix.rect(
            P2(width / 10, height / 2),
            (width - width / 5 + 1) as usize,
            1,
        );
    }
}

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
            para.pix
                .color(C | (((stream.get(index).1 * 32784.0 % 256.0) as u32) << 24));
            para.pix.rect(P2(i as i32, o as i32), 2, 2);
        }
    } else {
        let o = if flip_side {
            para.pix.width() - offset - 1
        } else {
            offset
        };

        for i in 0..para.pix.height() {
            let index = (i / 10 + i) % stream.len();
            para.pix
                .color(C | (((stream.get(index).1 * 32784.0 % 256.0) as u32) << 24));
            para.pix.rect(P2(o as i32, i as i32), 2, 2);
        }
    }
}
