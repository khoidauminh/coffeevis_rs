use std::cell::RefCell;
use std::f64::consts::LN_2;

use crate::graphics::{blend::Blend, P2};
use crate::math::{self, interpolate::*, Cplx};

const FFT_SIZE: usize = crate::data::FFT_SIZE / 2;
const RANGE: usize = 64;
const RANGEF: f64 = RANGE as f64;
const FFT_SIZEF: f64 = FFT_SIZE as f64;
const FFT_SIZEF_RECIP: f64 = 1.0 / FFT_SIZEF;

// static DATA: RwLock<[Cplx; RANGE + 1]> = RwLock::new([Cplx::zero(); RANGE + 1]);
// I'm not sure if this has any improvement, but I'm trying it anyway.
thread_local! {
    static DATA: RefCell<[Cplx; RANGE + 1]> = const { RefCell::new([Cplx::zero(); RANGE + 1]) };
}

fn l1_norm_slide(a: Cplx, t: f64) -> f64 {
    a.x.abs() * t + a.y.abs() * (1.0 - t)
}

fn index_scale(x: f64) -> f64 {
    math::fast::unit_exp2_0(x)
}

fn index_scale_derivative(x: f64) -> f64 {
    (math::fast::unit_exp2_0(x) + 1.0) * LN_2
}

fn prepare(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    // const WINDOW: usize = 2 * FFT_SIZE / 3;
    let fall_factor = 0.4 * prog.SMOOTHING.powi(2) * (prog.MILLI_HZ / 1000) as f64 * 0.006944444;
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
        let scalef = math::fast::ilog2(i + 1) as f64 * (1.5 - i as f64 / RANGEF) * 1.621;
        *smp = *smp * scalef;
    });

    crate::audio::limiter_pong(&mut fft[0..RANGE], 1.5, 15, prog.VOL_SCL, |x| x.max());

    DATA.with_borrow_mut(move |LOCAL| {
        LOCAL.iter_mut().zip(fft.iter()).for_each(|(smp, si)| {
            smp.x = multiplicative_fall(smp.x, si.x, 0.0, fall_factor);
            smp.y = multiplicative_fall(smp.y, si.y, 0.0, fall_factor);
        });
    });

    stream.auto_rotate();
}

pub fn draw_spectrum(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let _l = stream.len();

    let (w, h) = prog.pix.sizet();

    let winwh = w >> 1;

    prepare(prog, stream);

    let wf = w as f64;
    let hf = h as f64;

    let wf_recip = 1.0 / wf;
    let hf_recip = 1.0 / hf;

    DATA.with_borrow(move |normalized| {
        prog.pix.clear();

        for i in 0..h {
            let i_rev = h - i;

            let i_ratio = i_rev as f64 * hf_recip;

            let slide_output = index_scale(i_ratio);
            let idxf = slide_output * RANGEF;
            let idx = idxf as usize;
            let idx_next = idx + 1;

            let bar_width_l;
            let bar_width_r;

            if idx_next < RANGE {
                let t = idxf.fract();

                bar_width_l = smooth_step(normalized[idx].x, normalized[idx_next].x, t);
                bar_width_r = smooth_step(normalized[idx].y, normalized[idx_next].y, t);
            } else {
                const LAST: usize = RANGE - 1;
                bar_width_l = normalized[LAST].x;
                bar_width_r = normalized[LAST].y;
            }

            let bar_width_l = bar_width_l * wf;
            let bar_width_r = bar_width_r * wf;

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
            // let rect_l_size = (bar_width_l as usize / 2, 1);

            let rect_r = P2::new(((bar_width_r as i32 + 1) / 2 + winwh).min(w), i);

            let middle = P2::new(winwh + 1, i);

            prog.pix.draw_rect_xy(rect_l, middle, color1);
            prog.pix.draw_rect_xy(middle, rect_r, color2);

            let alpha = (128.0 + stream[i as usize / 2].x * 32768.0) as u8;
            prog.pix
                .draw_rect_wh_by(P2::new(winwh - 1, i), 2, 1, color.fade(alpha), u32::over);
        }
    });
}
