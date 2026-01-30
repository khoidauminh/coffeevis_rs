use crate::{
    graphics::P2,
    visualizers::{Visualizer, VisualizerArgs},
};

const PERBYTE: usize = 16; // like percent but ranges from 0..256

pub struct Wave;

impl Visualizer for Wave {
    fn name(&self) -> &'static str {
        "Wave"
    }

    fn perform(&mut self, args: VisualizerArgs) {
        let VisualizerArgs {
            pix, stream, keys, ..
        } = args;

        let l = (stream.len() * PERBYTE) >> 8;

        let (width, height) = pix.sizeu();

        for x in 0..width {
            let i = l * x / width;
            let smp = stream.get(l - i);

            let r: u8 = (smp.0 * 144.0 + 128.0) as u8;
            let b: u8 = (smp.1 * 144.0 + 128.0) as u8;
            let g: u8 = ((r as u16 + b as u16) / 2) as u8;

            pix.color(u32::from_be_bytes([b, r, g, b]));
            pix.mixerm();
            pix.rect(P2(x as i32, 0), 1, height);
        }

        stream.autoslide();
    }
}
