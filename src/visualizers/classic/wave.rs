use crate::graphics::P2;

const PERBYTE: usize = 16; // like percent but ranges from 0..256

pub fn draw_wave(para: &mut crate::Program, stream: &mut crate::AudioBuffer) {
    let l = (stream.len() * PERBYTE) >> 8;
    let _random = 0usize;

    let (width, height) = para.pix.sizeu();

    for x in 0..width {
        let i = l * x / width;
        let smp = stream.get(i);

        let r: u8 = (smp.0 * 144.0 + 128.0) as u8;
        let b: u8 = (smp.1 * 144.0 + 128.0) as u8;
        let g: u8 = ((r as u16 + b as u16) / 2) as u8;

        para.pix.color(u32::from_be_bytes([b, r, g, b]));
        para.pix.mixerm();
        para.pix.rect(
            P2(x as i32, 0),
            1,
            height,
        );
    }

    stream.autoslide();
}
