use std::sync::Mutex;

use crate::data::FFT_SIZE;
use crate::graphics::{blend::Blend, P2};
use crate::math::{self, Cplx};

const COLOR: [u32; 3] = [0x66ff66, 0xaaffff, 0xaaaaff];

const FFT_SIZE_HALF: usize = FFT_SIZE / 2;

static DATA: Mutex<Vec<f32>> = Mutex::new(Vec::new());
static MAX: Mutex<f32> = Mutex::new(0.0);

const FFT_SIZE_RECIP: f32 = 1.4 / FFT_SIZE as f32;
const NORMALIZE_FACTOR: f32 = FFT_SIZE_RECIP;
const BARS: usize = 48;
const MAX_BARS: usize = 128;

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

    let mut local = DATA.lock().unwrap();

    if bar_num != local.len() {
        local.resize(bar_num, 0.0);
    }

    let mut data_c = [Cplx::zero(); FFT_SIZE];

    const UP: usize = FFT_SIZE / (BARS * 3 / 2);
    data_c
        .iter_mut()
        .take(FFT_SIZE_HALF)
        .enumerate()
        .for_each(|(i, smp)| *smp = stream[i * UP]);

    let bound = data_c.len().min(math::ideal_fft_bound(BARS));

    math::fft(&mut data_c[0..bound]);

    let _max = MAX.lock().unwrap();

    const NORM: f32 = FFT_SIZE_RECIP;

    let mut data_f = [0f32; MAX_BARS];
    data_f
        .iter_mut()
        .take(bar_num)
        .zip(data_c.iter())
        .enumerate()
        .for_each(|(i, (smp, cplx))| {
            let scl = ((i + 2) as f32).log2().powi(2);
            let smp_f32: f32 = cplx.mag();

            *smp = smp_f32 * volume_scale * scl * NORM;
        });

    crate::audio::limiter::<_, MAX_BARS>(&mut data_f[..bar_num], 1.0, 10, 0.98, |x| x);

    let bnf = 1.0 / bnf;

    local
        .iter_mut()
        .zip(data_f.iter())
        .take(bar_num)
        .enumerate()
        .for_each(|(i, (w, r))| {
            let i_ = i as f32 * bnf;
            let accel = (0.090 - 0.055 * i_) * prog_smoothing;
            *w = math::interpolate::multiplicative_fall(*w, *r, 0.0, accel);
        });

    stream.rotate_left(FFT_SIZE_HALF / 2);
}

pub fn draw_bars(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    use crate::math::{fast::cubed_sqrt, interpolate::smooth_step};

    let bar_num = prog.pix.width() / 2;
    let bnf = bar_num as f32;
    let bnf_recip = 1.0 / bnf;
    let _l = stream.len();

    prepare(stream, bar_num, prog.vol_scl, prog.smoothing);

    let local = DATA.lock().unwrap();

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
            local[idx],
            local[(idx + 1).min(bar_num)],
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

    let bar_num = math::fast::isqrt(prog.pix.sizel()).min(MAX_BARS - 1);
    let bnf = bar_num as f32;
    let bnf_recip = 1.0 / bnf;
    let _l = stream.len();

    let wh = prog.pix.width() as i32 / 2;
    let hh = prog.pix.height() as i32 / 2;

    prepare(stream, bar_num, prog.vol_scl, prog.smoothing);

    let local = DATA.lock().unwrap();

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

        let bar = math::interpolate::linearf(local[i], local[i_next], t) * sizef;
        let bar = bar * 0.7;

        let p1 = P2::new(
            wh + (sizef * angle.x) as i32 / 2,
            hh + (sizef * angle.y) as i32 / 2,
        );
        let p2 = P2::new(
            wh + ((sizef - bar) * angle.x) as i32 / 2,
            hh + ((sizef - bar) * angle.y) as i32 / 2,
        );

        let _i2 = i << 2;

        let pulse = (stream[i * 3 / 2].x * 32768.0) as u8;
        let peak = (bar as i32 * 255 / size).min(255) as u8;

        let r: u8 = 0;
        let g: u8 = peak.saturating_add(pulse >> 1);
        let b: u8 = 0xFF;
        let c = u32::compose([0xFF, r, g, b]);

        prog.pix.draw_line(p1, p2, c);
    }
}
