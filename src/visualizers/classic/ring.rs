use crate::graphics::P2;
use crate::math::Cplx;
use crate::visualizers::Visualizer;

pub struct Ring;

impl Visualizer for Ring {
    fn name(&self) -> &'static str {
        "Ring"
    }

    fn perform(
        &mut self,
        pix: &mut crate::graphics::PixelBuffer,
        key: &crate::data::KeyInput,
        stream: &mut crate::audio::AudioBuffer,
    ) {
        const RANGE: usize = 128;

        let size = pix.height().min(pix.width()) as i32;

        let width = pix.width() as i32;
        let height = pix.height() as i32;

        let width_top_h = width >> 1;
        let height_top_h = height >> 1;

        pix.clear();

        let rate = -1.0 / RANGE as f32;

        let loop_range = size as usize * 2;

        for i in 1..loop_range {
            let di = i * RANGE / loop_range;

            let smp = stream.get(di) - stream.get(di.saturating_sub(1)).scale(0.7);

            let p = (smp * Cplx::new(0.65, 0.0) + Cplx::new(0.4, 0.4))
                * crate::math::cos_sin(di as f32 * rate);
            let x = (p.0 * size as f32) as i32;
            let y = (p.1 * size as f32) as i32;

            let int = (smp.l1_norm() * 128.0) as u8;

            pix.color(u32::from_be_bytes([
                255,
                ((128 + x.abs() * 64 / size) as u8).saturating_sub(int),
                255,
                ((128 + y.abs() * 64 / size) as u8).saturating_add(int),
            ]));
            pix.mixerd();
            pix.plot(P2(x / 2 + width_top_h, y / 2 + height_top_h));
        }

        stream.autoslide();
    }
}
