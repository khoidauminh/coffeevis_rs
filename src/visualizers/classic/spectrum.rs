use std::f32::consts::LN_2;

use crate::audio::AudioBuffer;
use crate::data::Program;
use crate::graphics::P2;
use crate::math::{self, Cplx, interpolate::*};
use crate::visualizers::{Visualizer, VisualizerConfig};

const FFT_SIZE: usize = 1 << 10;
const RANGE: usize = 64;
const RANGEF: f32 = RANGE as f32;
const FFT_SIZEF: f32 = FFT_SIZE as f32;
const FFT_SIZEF_RECIP: f32 = 1.0 / FFT_SIZEF;
const SMOOTHING: f32 = 0.90;

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

fn prepare(stream: &mut crate::AudioBuffer, local: &mut LocalType) {
    let mut fft = [Cplx::zero(); FFT_SIZE];

    stream.read(&mut fft[..FFT_SIZE / 2]);
    math::fft_stereo(&mut fft, RANGE, math::Normalize::No);

    fft.iter_mut().take(RANGE).enumerate().for_each(|(i, smp)| {
        let scalef = 2.5 / FFT_SIZE as f32 * math::fast::ilog2(i + 1) as f32;
        *smp *= scalef;
    });

    crate::audio::limiter(&mut fft[0..RANGE], 0.0, 0.90, |x| x.max());

    local.iter_mut().zip(fft.iter()).for_each(|(smp, si)| {
        smp.0 = decay(smp.0, si.0.abs(), SMOOTHING);
        smp.1 = decay(smp.1, si.1.abs(), SMOOTHING);
    });

    stream.autoslide();
}

pub struct Spectrum {
    buffer: [Cplx; RANGE + 1],
}

impl Default for Spectrum {
    fn default() -> Self {
        Self {
            buffer: [Cplx::zero(); _],
        }
    }
}

impl Visualizer for Spectrum {
    fn name(&self) -> &'static str {
        "Spectrum"
    }

    fn config(&self) -> crate::visualizers::VisualizerConfig {
        return VisualizerConfig { normalize: true };
    }

    fn perform(&mut self, prog: &mut Program, stream: &mut AudioBuffer) {
        prepare(stream, &mut self.buffer);

        let P2(w, h) = prog.pix.size();
        let winwh = w >> 1;

        let wf = w as f32;
        let hf = h as f32;

        let whf = wf * 0.5;

        prog.pix.clear();
        prog.pix.mixerd();

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
            let middle = P2(winwh + 1, h - y);

            prog.pix.color(color);
            prog.pix.rect_xy(rect_l, middle);
            prog.pix.rect_xy(middle, rect_r);

            let s = stream.get((h - y) as usize);
            let c1 = if s.0 > 0.0 { 255 } else { 0 };
            let c2 = if s.1 > 0.0 { 255 } else { 0 };

            prog.pix.color(u32::from_be_bytes([255, c1, 0, c2]));
            prog.pix.rect(P2(winwh - 1, ry), 2, 1);
        }
    }
}
