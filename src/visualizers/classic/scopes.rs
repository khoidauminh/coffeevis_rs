use std::sync::{
    atomic::{AtomicUsize, Ordering::Relaxed},
    Mutex,
};

use crate::graphics::Pixel;

use smallvec::SmallVec;

use crate::audio::MovingAverage;
use crate::data::{INCREMENT, PHASE_OFFSET};
use crate::graphics::P2;
use crate::math::interpolate::linearfc;
use crate::visualizers::classic::cross::{draw_cross, CROSS_COL};

use crate::math::{self, Cplx};

static LOCALI: AtomicUsize = AtomicUsize::new(0);
static WAVE_SCALE_FACTOR: Mutex<f32> = Mutex::new(1.0);
const SMOOTH_SIZE: usize = 7;
const SMOOTH_BUFFER_SIZE: usize = 512;

fn to_color(s: i32, size: i32) -> u8 {
    (s.abs() * 256 / size).min(255) as u8
}

pub fn draw_vectorscope(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let range = prog.wav_win;
    let _l = stream.len();

    let size = prog.pix.height().min(prog.pix.width()) as i32;
    let sizei = size;
    let scale = size as f32 * prog.vol_scl * 0.5;

    let (width, height) = prog.pix.size().as_tuple();

    let width_top_h = width >> 1;
    let height_top_h = height >> 1;

    let mut di: usize = 0;

    prog.pix.clear();

    let mut smoothed_sample = MovingAverage::<_, SMOOTH_SIZE>::init(Cplx::zero(), SMOOTH_SIZE);

    for _ in 0..SMOOTH_SIZE {
        let sample = Cplx::new(stream[di].x, stream[di + PHASE_OFFSET].y);
        _ = smoothed_sample.update(sample);
        di += INCREMENT;
    }

    while di < range {
        let sample = Cplx::new(stream[di].x, stream[di + PHASE_OFFSET].y);

        let sample = smoothed_sample.update(sample);

        let x = (sample.x * scale) as i32;
        let y = (sample.y * scale) as i32;
        let amp = (x.abs() + y.abs()) * 3 / 2;

        prog.pix.plot(
            P2::new(x + width_top_h, y + height_top_h),
            u32::from_be_bytes([255, to_color(amp, sizei), 255, 64]),
            u32::mix,
        );

        di += INCREMENT;
    }

    draw_cross(prog);

    stream.rotate_left(prog.wav_win);
}

pub fn draw_oscilloscope(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let Ok(mut wave_scale_factor) = WAVE_SCALE_FACTOR.try_lock() else {
        return;
    };

    let l = stream.input_size() * 2 / 3;
    let (width, height) = prog.pix.size().as_tuple();
    let width_top_h = width / 2;
    let height_top_h = height / 2;

    let scale = prog.pix.height() as f32 * prog.vol_scl * 0.45;

    let mut bass_sum = 0.0;

    let mut i = (-20_isize) as usize;

    let mut ssmp = stream[i];
    let mut old = ssmp;
    let mut old2 = old;

    while i != 0 {
        ssmp = linearfc(ssmp, stream[i], 0.01);
        old2 = old;
        old = ssmp;
        i = i.wrapping_add(1);
    }

    let mut zeros = SmallVec::<[usize; 6]>::new();
    zeros.push(0);

    for i in 0..l {
        let t = stream[i];
        ssmp = linearfc(ssmp, t, 0.01);

        if zeros.len() < 5 {
            if old2.x > 0.0 && ssmp.x < 0.0 {
                zeros.push(i);
            }
            if old2.y > 0.0 && ssmp.y < 0.0 {
                zeros.push(i);
            }
        }

        old2 = old;
        old = ssmp;

        bass_sum += ssmp.x.powi(2) + ssmp.y.powi(2);
    }

    let zeroi = zeros[zeros.len() - 1] - 50;

    let bass = (bass_sum / l as f32).sqrt();

    let wsf = bass * 10.0 + 2.0;

    *wave_scale_factor = math::interpolate::subtractive_fall(*wave_scale_factor, wsf, 1.0, 0.5);

    prog.pix.clear();

    let wave_scale_factor = *wave_scale_factor as isize;

    for x in 0..prog.pix.width() as i32 {
        let di = (x - width_top_h) as isize * wave_scale_factor + zeroi as isize;

        let mut smp_max = Cplx::new(-1000.0, -1000.0);
        let mut smp_min = Cplx::new(1000.0, 1000.0);

        for i in di..di + wave_scale_factor {
            let smp = &stream[i as usize];

            smp_max.x = smp_max.x.max(smp.x);
            smp_min.x = smp_min.x.min(smp.x);

            smp_max.y = smp_max.y.max(smp.y);
            smp_min.y = smp_min.y.min(smp.y);
        }

        let y1_max = height_top_h + (smp_max.x * scale) as i32;
        let y1_min = height_top_h + (smp_min.x * scale) as i32;

        let y2_max = height_top_h + (smp_max.y * scale) as i32;
        let y2_min = height_top_h + (smp_min.y * scale) as i32;

        let p1_max = P2::new(x, y1_max);
        let p1_min = P2::new(x, y1_min);

        let p2_max = P2::new(x, y2_max);
        let p2_min = P2::new(x, y2_min);

        prog.pix.line(p1_max, p1_min, 0xFF_55_FF_55, |a, b| a | b);
        prog.pix.line(p2_max, p2_min, 0xFF_55_55_FF, |a, b| a | b);
    }

    let mut li = LOCALI.load(Relaxed);
    let random = math::rng::random_int(3) as usize;

    li = (li + random + 3) % prog.pix.width();

    let rx = width - li as i32 - 1;

    let width = width as usize;
    let height = height as usize;

    prog.pix.rect_wh(
        P2::new(rx, height / 10),
        1,
        width - height / 4,
        CROSS_COL,
        u32::max,
    );

    prog.pix
        .rect_wh(P2::new(rx, height / 2), width >> 3, 1, CROSS_COL, u32::mix);

    LOCALI.store(li, Relaxed);

    stream.rotate_left(zeroi / 2);
}
