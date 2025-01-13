use std::f32::consts::LN_2;

use std::sync::{Mutex, MutexGuard};

use crate::graphics::{blend::Blend, P2};
use crate::math::{self, interpolate::*, Cplx};

const FFT_SIZE: usize = crate::data::FFT_SIZE / 2;
const RANGE: usize = 64;
const RANGEF: f32 = RANGE as f32;
const FFT_SIZEF: f32 = FFT_SIZE as f32;
const FFT_SIZEF_RECIP: f32 = 1.0 / FFT_SIZEF;

type LocalType = [Cplx; RANGE + 1];

fn l1_norm_slide(a: Cplx, t: f32) -> f32 {
    a.x.abs() * t + a.y.abs() * (1.0 - t)
}

fn index_scale(x: f32) -> f32 {
    math::fast::unit_exp2_0(x)
}

fn index_scale_derivative(x: f32) -> f32 {
    (math::fast::unit_exp2_0(x) + 1.0) * LN_2
}

fn prepare(
    prog: &mut crate::data::Program,
    stream: &mut crate::audio::SampleArr,
    local: &mut MutexGuard<LocalType>,
) {
    let accel = 0.28 * prog.smoothing.powi(2);
    let mut fft = [Cplx::zero(); FFT_SIZE];
    const UP: usize = 2 * FFT_SIZE / (RANGE * 3 / 2);

    fft.iter_mut()
        .take(FFT_SIZE)
        .enumerate()
        .for_each(|(i, smp)| {
            let idx = i * UP;
            *smp = stream[idx];
        });

    math::fft_stereo(&mut fft, RANGE, true);

    fft.iter_mut().take(RANGE).enumerate().for_each(|(i, smp)| {
        let scalef = math::fast::ilog2(i + 1) as f32 * (1.5 - i as f32 / RANGEF) * 1.621;
        *smp *= scalef;
    });

    crate::audio::limiter::<_, RANGE>(&mut fft[0..RANGE], 1.5, 7, prog.vol_scl * 2.0, |x| x.max());

    local.iter_mut().zip(fft.iter()).for_each(|(smp, si)| {
        smp.x = multiplicative_fall(smp.x, si.x, 0.0, accel);
        smp.y = multiplicative_fall(smp.y, si.y, 0.0, accel);
    });

    stream.auto_rotate();
}

pub fn draw_spectrum(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    static DATA: Mutex<LocalType> = Mutex::new([Cplx::zero(); RANGE + 1]);

    let Ok(mut local) = DATA.try_lock() else {
        return;
    };

    prepare(prog, stream, &mut local);

    let _l = stream.len();
    let (w, h) = prog.pix.sizet();
    let winwh = w >> 1;

    let wf = w as f32;
    let hf = h as f32;

    let whf = wf * 0.5;

    let wf_recip = 1.0 / wf;
    let hf_recip = 1.0 / hf;

    prog.pix.clear();

    for i in 0..h {
        let i_rev = h - i;

        let i_ratio = i_rev as f32 * hf_recip;

        let slide_output = index_scale(i_ratio);
        let idxf = slide_output * RANGEF;
        let idx = idxf as usize;
        let idx_next = idx + 1;

        let bar_width_l;
        let bar_width_r;

        if idx_next < RANGE {
            let t = idxf.fract();

            bar_width_l = smooth_step(local[idx].x, local[idx_next].x, t);
            bar_width_r = smooth_step(local[idx].y, local[idx_next].y, t);
        } else {
            const LAST: usize = RANGE - 1;
            bar_width_l = local[LAST].x;
            bar_width_r = local[LAST].y;
        }

        let bar_width_l = bar_width_l * whf;
        let bar_width_r = bar_width_r * whf;

        let channel_l = (255.0 * bar_width_l.min(wf) * wf_recip) as u32;
        let channel_r = (255.0 * bar_width_r.min(wf) * wf_recip) as u32;

        let color = u32::from_be_bytes([
            0xFF,
            (255 - 255 * i_rev / h) as u8,
            0,
            (128 + 96 * i_rev / h) as u8,
        ]);

        let color1 = color | channel_l << 8;
        let color2 = color | channel_r << 8;

        let rect_l = P2::new((wf - bar_width_l).max(0.0) as i32 / 2, i);
        let rect_r = P2::new(((bar_width_r as i32 + 1) / 2 + winwh).min(w), i);

        let middle = P2::new(winwh + 1, i);

        prog.pix.draw_rect_xy(rect_l, middle, color1);
        prog.pix.draw_rect_xy(middle, rect_r, color2);

        let alpha = (128.0 + stream[i as usize / 2].x * 32768.0) as u8;
        prog.pix
            .draw_rect_wh_by(P2::new(winwh - 1, i), 2, 1, color.fade(alpha), u32::over);
    }
}
