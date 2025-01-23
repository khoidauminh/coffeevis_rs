use crate::graphics::{Pixel, P2};

const PERBYTE: usize = 16; // like percent but ranges from 0..256

pub fn draw_wave(para: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let l = (stream.len() * PERBYTE) >> 8;
    let _random = 0usize;

    for x in 0..para.pix.width() {
        let i = l * x / para.pix.width();
        let smp = stream[i];

        let r: u8 = (smp.x * 144.0 + 128.0) as u8;
        let b: u8 = (smp.y * 144.0 + 128.0) as u8;
        let g: u8 = ((r as u16 + b as u16) / 2) as u8;

        para.pix.command.rect_wh(
            P2::new(x as i32, 0),
            1,
            para.pix.height(),
            u32::from_be_bytes([b, r, g, b]),
            u32::mix,
        );
    }

    stream.rotate_left(l >> 1);
}
