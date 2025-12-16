use crate::data::SAMPLE_SIZE;
use crate::graphics::P2;
use crate::math::fast::sin_norm;
use crate::visualizers::Visualizer;

const C: u32 = 16720064;

#[derive(Default)]
pub struct VolSweeper {
    sweepi: usize,
    pong: bool,
}

impl Visualizer for VolSweeper {
    fn name(&self) -> &'static str {
        "Volum sweeper"
    }

    fn perform(
        &mut self,
        pix: &mut crate::graphics::PixelBuffer,
        key: &crate::data::KeyInput,
        stream: &mut crate::audio::AudioBuffer,
    ) {
        pix.fade(3);

        let w = {
            let mut sum = 0.0;
            for i in 0..SAMPLE_SIZE / 4 {
                sum += stream.get(i).l1_norm();
            }
            (sum / (SAMPLE_SIZE / 3) as f32 * pix.width() as f32) as usize
        };

        let color = (w * 255 / pix.width()).min(255) as u8;
        let color = u32::from_be_bytes([
            255,
            255,
            (sin_norm(color as f32 / 512.0) * 255.0) as u8,
            color,
        ]);

        let width = pix.width();

        pix.color(0);
        pix.mixerm();
        pix.rect(P2(0, self.sweepi as i32), width, 1);
        pix.color(color);
        pix.rect(P2(0, self.sweepi as i32), w, 1);

        match (self.sweepi >= pix.height(), self.pong) {
            (false, true) => self.sweepi = self.sweepi.wrapping_add(1),
            (false, false) => self.sweepi = self.sweepi.wrapping_sub(1),

            (true, true) => {
                self.sweepi -= 1;
                self.pong ^= true;
            }

            (true, false) => {
                self.sweepi = 0;
                self.pong ^= true;
            }
        }

        stream.autoslide();
    }
}
