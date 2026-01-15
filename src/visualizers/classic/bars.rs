use crate::data::FFT_SIZE;
use crate::graphics::{P2, Pixel};
use crate::math::Fft;
use crate::math::{self, Cplx, interpolate::linearf};
use crate::visualizers::Visualizer;

const COLOR: [u32; 3] = [0x66ff66, 0xaaffff, 0xaaaaff];

const FFT_SIZE_HALF: usize = FFT_SIZE / 2;

const FFT_SIZE_RECIP: f32 = 1.4 / FFT_SIZE as f32;
const NORMALIZE_FACTOR: f32 = FFT_SIZE_RECIP;
const BARS: usize = 36;
const MAX_BARS: usize = 144;
const MAX_BARS1: usize = MAX_BARS + 1;

pub struct Bars {
    pub data: [f32; MAX_BARS + 1],
    pub max: f32,
    pub fft: crate::math::Fft,
}

pub struct BarsCircle {
    pub data: [f32; MAX_BARS + 1],
    pub max: f32,
    pub fft: crate::math::Fft,
}

fn prepare(stream: &mut crate::AudioBuffer, bar_num: usize, data: &mut [f32], fft: &Fft) {
    let bar_num = bar_num + 1;

    let bnf = bar_num as f32;

    let mut data_c = [Cplx::zero(); FFT_SIZE];
    for (n, d) in data_c.iter_mut().enumerate() {
        *d = stream.get((FFT_SIZE - n) * 3);
    }

    fft.exec(&mut data_c);

    let norm: f32 = 1.0 / FFT_SIZE as f32;

    let mut data_f = [0f32; MAX_BARS1];
    data_f
        .iter_mut()
        .take(bar_num)
        .zip(data_c.iter())
        .enumerate()
        .for_each(|(i, (smp, cplx))| {
            let scl = 2.0 * ((i + 2) as f32).log2();
            let smp_f32: f32 = cplx.mag();

            *smp = smp_f32 * scl * norm;
        });

    crate::audio::limiter(&mut data_f[..bar_num], 0.0, 0.95, |x| x);

    let bnf = 1.0 / bnf;

    data.iter_mut()
        .zip(data_f.iter())
        .take(bar_num)
        .enumerate()
        .for_each(|(i, (w, r))| {
            let i_ = (i + 1) as f32 * bnf;
            let accel = 0.95 + 0.025 * i_;
            *w = math::interpolate::decay(*w, *r, accel);
        });

    stream.autoslide();
}

impl Visualizer for Bars {
    fn name(&self) -> &'static str {
        "Bars"
    }

    fn perform(
        &mut self,
        pix: &mut crate::graphics::PixelBuffer,
        key: &crate::data::KeyInput,
        stream: &mut crate::audio::AudioBuffer,
    ) {
        use crate::math::{fast::cubed_sqrt, interpolate::smooth_step};

        let bar_num = (pix.width() / 2).min(MAX_BARS);
        let bnf = bar_num as f32;
        let bnf_recip = 1.0 / bnf;

        prepare(stream, bar_num, &mut self.data, &self.fft);

        pix.clear();
        let size = pix.sizeu();
        let sizef = Cplx(pix.width() as f32, pix.height() as f32);

        let mut iter: f32 = 0.4;

        let mut prev_index = 0;
        let mut smoothed_smp = 0f32;

        loop {
            let i_ = iter * bnf_recip;
            let idxf = iter;

            iter += i_;

            let idx = idxf as usize;

            let t = idxf.fract();

            smoothed_smp = smoothed_smp.max(cubed_sqrt(smooth_step(
                self.data[idx],
                self.data[(idx + 1).min(bar_num)],
                t,
            )));

            if prev_index == idx {
                continue;
            }

            prev_index = idx;

            let idx = idx - 1;

            let bar = smoothed_smp * sizef.1;

            let bar = (bar as usize).clamp(1, pix.height());

            let fade = (128.0 + stream.get(idx * 3 / 2).0 * 256.0) as u8;
            let peak = (bar * 255 / pix.height()) as u8;
            let _red = (fade.wrapping_mul(2) / 3).saturating_add(128).max(peak);

            pix.color(u32::from_be_bytes([0xFF, 0xFF, (fade).max(peak), 0]));
            pix.rect(
                P2(
                    (size.0 * idx / bar_num) as i32,
                    (size.1 - bar.min(size.1 - 1)) as i32,
                ),
                2,
                bar,
            );

            smoothed_smp = 0.0;

            if idx + 1 == bar_num {
                break;
            }
        }
    }
}

impl Default for Bars {
    fn default() -> Self {
        Self {
            data: [0.0; _],
            max: 0.0,
            fft: Fft::new(FFT_SIZE),
        }
    }
}

impl Default for BarsCircle {
    fn default() -> Self {
        Self {
            data: [0.0; _],
            max: 0.0,
            fft: Fft::new(FFT_SIZE),
        }
    }
}

impl Visualizer for BarsCircle {
    fn name(&self) -> &'static str {
        "Bars Cicle"
    }

    fn perform(
        &mut self,
        pix: &mut crate::graphics::PixelBuffer,
        key: &crate::data::KeyInput,
        stream: &mut crate::audio::AudioBuffer,
    ) {
        let size = pix.height().min(pix.width()) as i32;
        let sizef = size as f32;

        let bar_num = pix.sizel().isqrt().min(MAX_BARS);
        let bnf = bar_num as f32;
        let bnf_recip = 1.0 / bnf;

        let wh = pix.width() as i32 / 2;
        let hh = pix.height() as i32 / 2;

        prepare(stream, bar_num, &mut self.data, &self.fft);

        pix.clear();

        const FFT_WINDOW: f32 = (FFT_SIZE >> 6) as f32 * 1.5;

        for i in 0..bar_num {
            let i_ = i as f32 * bnf_recip;

            let idxf = i_ * FFT_WINDOW;
            let t = idxf.fract();
            let i_next = i + 1;

            let angle = math::cos_sin(i_);

            let bar = linearf(self.data[i], self.data[i_next], t) * sizef;
            let bar = bar * 0.7;

            let p1 = P2(
                wh + (sizef * angle.0) as i32 / 2,
                hh + (sizef * angle.1) as i32 / 2,
            );
            let p2 = P2(
                wh + ((sizef - bar) * angle.0) as i32 / 2,
                hh + ((sizef - bar) * angle.1) as i32 / 2,
            );

            let pulse = (stream.get(i * 3 / 2).0 * 32768.0) as u8;
            let peak = (bar as i32 * 255 / size).min(255) as u8;

            let r: u8 = 0;
            let g: u8 = peak.saturating_add(pulse >> 1);
            let b: u8 = 0xFF;
            let c = u32::compose([0xFF, r, g, b]);

            pix.color(c);
            pix.line(p1, p2);
        }
    }
}
