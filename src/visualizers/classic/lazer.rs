use crate::data::SAMPLE_SIZE;
use crate::math::{Cplx, interpolate::linearf};
use crate::visualizers::{Visualizer, VisualizerArgs};

pub struct Lazer {
    p0: Cplx,
    p1: Cplx,
}

impl Default for Lazer {
    fn default() -> Self {
        Self {
            p0: Cplx::new(1.0, 1.0),
            p1: Cplx::new(1.0, 1.0),
        }
    }
}

impl Visualizer for Lazer {
    fn name(&self) -> &'static str {
        "Lazer"
    }

    fn perform(&mut self, args: VisualizerArgs) {
        let VisualizerArgs {
            pix, stream, keys, ..
        } = args;

        let w = pix.width() as f32;
        let h = pix.height() as f32;

        let mut a = {
            let mut sum = Cplx::zero();

            let left = 0..SAMPLE_SIZE / 8;
            let right = SAMPLE_SIZE / 8..SAMPLE_SIZE / 4;

            let mut smooth = 0.0;

            for i in left {
                smooth = linearf(smooth, stream.get(i).0, 0.1);
                sum.0 += smooth;
            }

            let mut smooth = 0.0;

            for i in right {
                smooth = linearf(smooth, stream.get(i).1, 0.1);
                sum.1 += smooth;
            }

            Cplx(sum.0 * 0.0035, sum.1 * 0.0035)
        };

        let mut p0 = self.p0;
        let mut p1 = self.p1;

        a *= p0;

        p0.0 = (p0.0 + a.0 + w) % w;
        p0.1 = (p0.1 + a.1 + h) % h;

        let color = u32::from_be_bytes([
            0xff,
            (48.0 + p0.0 * 255.0 / w).min(255.0) as u8,
            (48.0 + p0.1 * 255.0 / h).min(255.0) as u8,
            ((255.0 - a.0 * a.1 * 2.0).abs().min(255.0)) as u8,
        ]);

        pix.fade(3);

        pix.color(color);
        pix.mixerd();
        pix.line(p1.to_p2(), p0.to_p2());

        p1 = p0;

        self.p0 = p0;
        self.p1 = p1;
    }
}
