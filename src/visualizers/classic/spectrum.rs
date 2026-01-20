use std::f32::consts::LN_2;

use crate::audio::AudioBuffer;
use crate::graphics::P2;
use crate::graphics::PixelBuffer;
use crate::math::Dct;
use crate::math::{self, Cplx, interpolate::*};
use crate::visualizers::Visualizer;

const DCT_SIZE: usize = 1 << 9;
const RANGE: usize = 64;
const RANGEF: f32 = RANGE as f32;
const DCT_SIZEF: f32 = DCT_SIZE as f32;
const DCT_SIZEF_RECIP: f32 = 1.0 / DCT_SIZEF;
const SMOOTHING: f32 = 0.905;

type LocalType = [Cplx; RANGE + 1];

fn l1_norm_slide(a: Cplx, t: f32) -> f32 {
    a.0.abs() * t + a.1.abs() * (1.0 - t)
}

fn index_scale(x: f32) -> f32 {
    math::fast::unit_exp2_0(x)
}

fn index_scale_derivative(x: f32) -> f32 {
    (math::fast::unit_exp2_0(x) + 1.0) * LN_2
}

pub struct Spectrum {
    buffer: [Cplx; RANGE + 1],
    dct: Dct<Cplx>,
}

impl Default for Spectrum {
    fn default() -> Self {
        Self {
            buffer: [Cplx::zero(); _],
            dct: Dct::new(DCT_SIZE),
        }
    }
}

impl Spectrum {
    fn prepare(&mut self, stream: &mut crate::AudioBuffer) {
        let mut dct = [Cplx::zero(); DCT_SIZE];

        stream.read(&mut dct);

        self.dct.exec(&mut dct);

        dct.iter_mut().take(RANGE).enumerate().for_each(|(i, smp)| {
            let scalef = 12.0 / DCT_SIZE as f32 * 1.0f32.min(i as f32 * 0.25);
            *smp *= scalef;
        });

        crate::audio::limiter(&mut dct[0..RANGE], 0.0, 0.90, |x| x.max());

        self.buffer
            .iter_mut()
            .zip(dct.iter())
            .for_each(|(smp, si)| {
                smp.0 = decay(smp.0, si.0.abs(), SMOOTHING);
                smp.1 = decay(smp.1, si.1.abs(), SMOOTHING);
            });

        stream.autoslide();
    }
}

impl Visualizer for Spectrum {
    fn name(&self) -> &'static str {
        "Spectrum"
    }

    fn perform(
        &mut self,
        pix: &mut PixelBuffer,
        key: &crate::data::KeyInput,
        stream: &mut AudioBuffer,
    ) {
        self.prepare(stream);

        let P2(w, h) = pix.size();
        let winwh = w >> 1;

        let wf = w as f32;
        let hf = h as f32;

        let whf = wf * 0.5;

        pix.clear();
        pix.mixerd();

        for y in 0..h {
            let ifrac = (y as f32 / hf).exp2() - 1.0f32;
            let ifloat = ifrac * RANGEF;
            let ifloor = ifloat as usize;
            let iceil = ifloat.ceil() as usize;
            let ti = ifloat.fract();

            let sfloor = self.buffer[ifloor];
            let sceil = self.buffer[iceil];

            let sl = smooth_step(sfloor.0, sceil.0, ti);
            let sr = smooth_step(sfloor.1, sceil.1, ti);

            let sl = sl.powf(1.3) * whf;
            let sr = sr.powf(1.3) * whf;

            let channel = (y * 255 / h) as u8;
            let green = 255.min(16 + (3.0f32 * (sl + sr)) as u32) as u8;

            let color = u32::from_be_bytes([0xFF, 255 - channel, green, 128 + channel / 2]);

            let ry = h - y;

            let rect_l = P2((whf - sl) as i32, ry);
            let rect_r = P2((whf + sr) as i32, ry);
            let middle = P2(winwh + 1, ry);

            pix.color(color);
            pix.rect_xy(rect_l, middle);
            pix.rect_xy(middle, rect_r);

            let s = stream.get((h - y) as usize);
            let c1 = if s.0 > 0.0 { 255 } else { 0 };
            let c2 = if s.1 > 0.0 { 255 } else { 0 };

            pix.color(u32::from_be_bytes([255, c1, 0, c2]));
            pix.rect(P2(winwh - 1, ry), 2, 1);
        }
    }
}
