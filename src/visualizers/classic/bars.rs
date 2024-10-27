use std::sync::RwLock;

use crate::data::FFT_SIZE;
use crate::graphics::{blend::Blend, P2};
use crate::math::{self, Cplx};

const COLOR: [u32; 3] = [0x66ff66, 0xaaffff, 0xaaaaff];

const FFT_SIZE_HALF: usize = FFT_SIZE / 2;

static DATA: RwLock<Vec<f32>> = RwLock::new(Vec::new());
static MAX: RwLock<f32> = RwLock::new(0.0);

const FFT_SIZE_RECIP: f32 = 1.4 / FFT_SIZE as f32;
const NORMALIZE_FACTOR: f32 = FFT_SIZE_RECIP;
const BARS: usize = 48;

fn dynamic_smooth(t: f32, a: f32) -> f32 {
    ((t - a) / a).powi(2)
}

fn dynamic_smooth2(t: f32, b: f32) -> f32 {
    (b * t + 1.3).recip()
}

fn prepare(
    stream: &mut crate::audio::SampleArr,
    bar_num: usize,
    volume_scale: f32,
    prog_smoothing: f32,
) {
    let bar_num = bar_num + 1;

    let bnf = bar_num as f32;
    let _l = stream.len();

    let mut LOCAL = DATA.write().unwrap();

    if bar_num != LOCAL.len() {
        LOCAL.resize(bar_num, 0.0);
    }

    let mut data_f = [Cplx::zero(); FFT_SIZE];

    const UP: usize = FFT_SIZE / (BARS * 3 / 2);
    data_f
        .iter_mut()
        .take(FFT_SIZE_HALF)
        .enumerate()
        .for_each(|(i, smp)| *smp = stream[i * UP]);

    let bound = data_f.len().min(math::ideal_fft_bound(BARS));

    math::fft(&mut data_f[0..bound]);

    let _max = MAX.write().unwrap();

    const NORM: f32 = FFT_SIZE_RECIP;

    let mut data_f = data_f
        .iter_mut()
        .take(bar_num)
        .enumerate()
        .map(|(i, smp)| {
            let scl = ((i + 2) as f32).log2().powi(2);
            let smp_f32: f32 = (*smp).mag();

            smp_f32 * volume_scale * scl * NORM
        })
        .collect::<Vec<f32>>();

    crate::audio::limiter(&mut data_f[..bar_num], 0.9, 7, 0.98, |x| x);

    let bnf = 1.0 / bnf;

    LOCAL
        .iter_mut()
        .zip(data_f.iter())
        .take(bar_num)
        .enumerate()
        .for_each(|(i, (w, r))| {
            let i_ = i as f32 * bnf;
            let smoothing = (0.095 - 0.055 * i_) * prog_smoothing;
            *w = math::interpolate::multiplicative_fall(*w, *r, 0.0, smoothing);

            // *w = math::interpolate::linearf(*w, *r, dynamic_smoothing+0.25);
        });

    stream.rotate_left(FFT_SIZE_HALF / 2);
}

pub fn draw_bars(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    use crate::math::{fast::cubed_sqrt, interpolate::smooth_step};

    let bar_num = prog.pix.width() / 2;
    let bnf = bar_num as f32;
    let bnf_recip = 1.0 / bnf;
    let _l = stream.len();

    prepare(stream, bar_num, prog.VOL_SCL, prog.SMOOTHING);

    let LOCAL = DATA.read().unwrap();

    prog.pix.clear();
    let sizef = Cplx::new(prog.pix.width() as f32, prog.pix.height() as f32);

    let _bnfh = bnf * 0.5;

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
            LOCAL[idx],
            LOCAL[(idx + 1).min(bar_num)],
            t,
        )));

        if prev_index == idx {
            continue;
        }

        prev_index = idx;

        let idx = idx - 1;

        let bar = smoothed_smp * sizef.y;

        let bar = (bar as usize).clamp(1, prog.pix.height());

        let fade = (128.0 + stream[idx * 3 / 2].x * 256.0) as u8;
        let peak = (bar * 255 / prog.pix.height()) as u8;
        let _red = (fade.wrapping_mul(2) / 3).saturating_add(128).max(peak);

        prog.pix.draw_rect_wh(
            P2::new(
                (prog.pix.width() * idx / bar_num) as i32,
                (prog.pix.height() - bar.min(prog.pix.height() - 1)) as i32,
            ),
            2,
            bar,
            u32::from_be_bytes([0xFF, 0xFF, (fade).max(peak), 0]),
        );

        smoothed_smp = 0.0;

        if idx + 1 == bar_num {
            break;
        }
    }
}

pub fn draw_bars_circle(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let size = prog.pix.height().min(prog.pix.width()) as i32;
    let sizef = size as f32;

    let bar_num = math::fast::isqrt(prog.pix.sizel());
    let bnf = bar_num as f32;
    let bnf_recip = 1.0 / bnf;
    let _l = stream.len();

    let wh = prog.pix.width() as i32 / 2;
    let hh = prog.pix.height() as i32 / 2;

    prepare(stream, bar_num, prog.VOL_SCL, prog.SMOOTHING);

    let LOCAL = DATA.read().unwrap();

    prog.pix.clear();

    //let base_angle = math::cos_sin(1.0 / bnf);

    const FFT_WINDOW: f32 = (FFT_SIZE >> 6) as f32 * 1.5;

    for i in 0..bar_num {
        let i_ = i as f32 * bnf_recip;

        let idxf = i_ * FFT_WINDOW;
        let _idx = idxf as usize;
        let t = idxf.fract();
        let i_next = i + 1;

        let angle = math::cos_sin(i_);

        // let scalef = math::fft_scale_up(i, bar_num);

        let bar = (math::interpolate::linearf(LOCAL[i], LOCAL[i_next], t) * sizef) as i32;
        let bar = bar * 7 / 10;

        let p1 = P2::new(
            wh + (sizef * angle.x) as i32 / 2,
            hh + (sizef * angle.y) as i32 / 2,
        );
        let p2 = P2::new(
            wh + ((size - bar) as f32 * angle.x) as i32 / 2,
            hh + ((size - bar) as f32 * angle.y) as i32 / 2,
        );

        let _i2 = i << 2;

        let pulse = (stream[i * 3 / 2].x * 32768.0) as u8;
        let peak = (bar * 255 / size).min(255) as u8;

        let r: u8 = 0;
        let g: u8 = peak.saturating_add(pulse >> 1);
        //let b: u8 =
        let b: u8 = 0xFF;

        let c = u32::compose([0xFF, r, g, b]);

        prog.pix.draw_line(p1, p2, c);
    }
}
