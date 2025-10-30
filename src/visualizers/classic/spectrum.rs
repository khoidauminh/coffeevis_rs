use std::f32::consts::LN_2;

use std::sync::Mutex;

use crate::graphics::{P2, Pixel};
use crate::math::{self, Cplx, interpolate::*};

const FFT_SIZE: usize = 1 << 10;
const RANGE: usize = 64;
const RANGEF: f32 = RANGE as f32;
const FFT_SIZEF: f32 = FFT_SIZE as f32;
const FFT_SIZEF_RECIP: f32 = 1.0 / FFT_SIZEF;
const SMOOTHING: f32 = 0.91;

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

fn prepare(prog: &mut crate::Program, stream: &mut crate::AudioBuffer, local: &mut LocalType) {
    let mut fft = [Cplx::zero(); FFT_SIZE];

    stream.read(&mut fft[..FFT_SIZE/2]);
    math::fft_stereo(&mut fft, RANGE, math::Normalize::No);

    fft.iter_mut().take(RANGE).enumerate().for_each(|(i, smp)| {
        let scalef = 2.0 / FFT_SIZE as f32 * math::fast::ilog2(i + 1) as f32;
        *smp *= scalef;
    });

    crate::audio::limiter(&mut fft[0..RANGE], 0.0, 0.90, |x| x.max());

    local.iter_mut().zip(fft.iter()).for_each(|(smp, si)| {
        smp.x = decay(smp.x, si.x.abs(), SMOOTHING);
        smp.y = decay(smp.y, si.y.abs(), SMOOTHING);
    });

    stream.autoslide();
}

pub fn draw_spectrum(prog: &mut crate::Program, stream: &mut crate::AudioBuffer) {
    static DATA: Mutex<LocalType> = Mutex::new([Cplx::zero(); RANGE + 1]);

    let Ok(mut local) = DATA.try_lock() else {
        return;
    };

    prepare(prog, stream, &mut local);

    let _l = stream.len();
    let (w, h) = prog.pix.size().as_tuple();
    let winwh = w >> 1;

    let wf = w as f32;
    let hf = h as f32;

    let whf = wf * 0.5;

    let wf_recip = 1.0 / wf;
    let hf_recip = 1.0 / hf;

    prog.pix.clear();

    for y in 0..h {
        //let i_rev = h - i;
        let ifrac = (y as f32 / hf).exp2() - 1.0f32;
        let ifloat = ifrac * RANGEF;
        let ifloor = ifloat as usize;
        let iceil = ifloat.ceil() as usize;
        let ti = ifloat.fract(); 
        
        let sfloor = local[ifloor];
        let sceil = local[iceil];

        let sl = smooth_step(sfloor.x, sceil.x, ti);
        let sr = smooth_step(sfloor.y, sceil.y, ti);
    
        let sl = sl.powf(1.3) * whf;
        let sr = sr.powf(1.3) * whf;

        let channel = (y  * 255 / h) as u8;
        let green = 255.min(16 + (3.0f32 * (sl + sr)) as u32) as u8;

        let color = u32::from_be_bytes([
            0xFF,
            255 - channel,
            green,
            128 + channel / 2,
        ]);

        let yf = y as f32;
        let ry = h - y;
      
        let rect_l = P2::new((whf - sl) as i32, ry);
        let rect_r = P2::new((whf + sr) as i32, ry);

        let middle = P2::new(winwh + 1, h - y);

        prog.pix.rect(rect_l, middle, color, u32::over);
        prog.pix.rect(middle, rect_r, color, u32::over);

        let s = stream.get((h - y) as usize);
        let c1 = if s.x > 0.0 { 255 } else { 0 };
        let c2 = if s.y > 0.0 { 255 } else { 0 };

        prog.pix
            .rect_wh(P2::new(winwh - 1, ry), 2, 1, u32::from_be_bytes([255, c1, 0, c2]), u32::over);
    }
}
