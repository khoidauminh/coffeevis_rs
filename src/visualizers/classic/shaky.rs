use std::f32::consts::FRAC_PI_2;

use crate::graphics::Pixel;
use crate::math::{self, Cplx, fast};
use crate::visualizers::Visualizer;

// soft shaking
const INCR: f32 = 0.0001;

#[derive(Default)]
pub struct Shaky {
    i: f32,
    js: f32,
    jc: f32,
    xshake: f32,
    yshake: f32,
    x: i32,
    y: i32,
}

fn diamond_func(amp: f32, prd: f32, t: f32) -> (i32, i32) {
    (
        triangle_wav(amp, prd, t) as i32,
        triangle_wav(amp, prd, t + prd / 4.0) as i32,
    )
}

fn triangle_wav(amp: f32, prd: f32, t: f32) -> f32 {
    (4.0 * (t / prd - (t / prd + 0.5).trunc()).abs() - 1.0) * amp
}

impl Visualizer for Shaky {
    fn name(&self) -> &'static str {
        "Shaky"
    }

    fn perform(
        &mut self,
        pix: &mut crate::graphics::PixelBuffer,
        key: &crate::data::KeyInput,
        stream: &mut crate::audio::AudioBuffer,
    ) {
        let mut data_f = [Cplx::zero(); 512];
        stream.read(&mut data_f);

        let sizef = pix.width().min(pix.height()) as f32;

        math::integrate_inplace(&mut data_f, 128, math::Normalize::No);

        let amplitude = data_f.iter().fold(0f32, |acc, x| acc + x.l1_norm()) * sizef;

        let smooth_amplitude = amplitude * 0.00003;
        let amplitude_scaled = amplitude * 0.00000002;

        self.js = (self.js + amplitude_scaled) % 2.0;
        self.jc = (self.jc + amplitude_scaled * FRAC_PI_2) % 2.0;

        self.xshake = (smooth_amplitude) * fast::cos_norm(fast::wrap(self.jc));
        self.yshake = (smooth_amplitude) * fast::sin_norm(fast::wrap(self.js));

        self.x = math::interpolate::linearf(self.x as f32, self.xshake, 0.1) as i32;
        self.y = math::interpolate::linearf(self.y as f32, self.yshake, 0.1) as i32;

        self.js += 0.01;
        self.jc += 0.01;
        self.i = (self.i + INCR + amplitude_scaled) % 1.0;

        pix.color(pix.background());
        pix.fade(4);

        let (x_soft_shake, y_soft_shake) = diamond_func(8.0, 1.0, self.i);

        let final_x = x_soft_shake + self.x;
        let final_y = y_soft_shake + self.y;

        let width = pix.width() as i32;
        let height = pix.height() as i32;

        let red = 255i32.saturating_sub(final_x.abs() * 5) as u8;
        let blue = 255i32.saturating_sub(final_y.abs() * 5) as u8;
        let green = (amplitude * 0.0001) as u8;

        pix.color(u32::compose([0xFF, red, green, blue]));
        pix.mixerm();
        pix.rect(
            crate::graphics::P2(final_x + width / 2 - 1, final_y + height / 2 - 1),
            3,
            3,
        );
    }
}

const WRAPPER: f32 = 725.0;
