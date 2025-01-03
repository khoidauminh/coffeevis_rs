use std::sync::{
    atomic::{AtomicUsize, Ordering::Relaxed},
    Mutex,
};

use crate::audio::MovingAverage;
use crate::data::{INCREMENT, PHASE_OFFSET};
use crate::graphics::P2;
use crate::visualizers::classic::cross::{draw_cross, CROSS_COL};

use crate::math::{self, Cplx};

static LOCALI: AtomicUsize = AtomicUsize::new(0);
static WAVE_SCALE_FACTOR: Mutex<f32> = Mutex::new(1.0);

fn to_color(s: i32, size: i32) -> u8 {
    (s.abs() * 256 / size).min(255) as u8
}

pub fn draw_vectorscope(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let range = prog.wav_win;
    let _l = stream.len();

    let size = prog.pix.height().min(prog.pix.width()) as i32;
    let sizei = size;
    let scale = size as f32 * prog.vol_scl * 0.5;

    let (width, height) = prog.pix.sizet();

    let width_top_h = width >> 1;
    let height_top_h = height >> 1;

    let mut di: usize = 0;

    prog.pix.clear();

    const SMOOTH_SIZE: usize = 7;

    let mut smoothed_sample = MovingAverage::init(Cplx::zero(), SMOOTH_SIZE);

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

        prog.pix.set_pixel_xy(
            P2::new(x + width_top_h, y + height_top_h),
            u32::from_be_bytes([255, to_color(amp, sizei), 255, 64]),
        );

        di += INCREMENT;
    }

    draw_cross(prog);

    stream.rotate_left(prog.wav_win);
}

pub fn draw_oscilloscope(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let l = stream.len();
    let _li = l as i32;

    let (width, height) = prog.pix.sizet();

    let _width_top_h = width >> 1;
    let height_top_h = height >> 1;

    let scale = prog.pix.height() as f32 * prog.vol_scl * 0.45;

    prog.pix.clear();

    let mut stream_ = stream.to_vec();
    const UP_COUNT: usize = 75;
    math::integrate_inplace(&mut stream_, UP_COUNT, true);

    let mut zeroi = UP_COUNT;

    let mut bass = 0.0;

    while zeroi < stream_.len() {
        let smp1 = stream_[zeroi].x;
        let smp2 = stream_[zeroi - 2].x;

        if smp1 < 0.0 && smp2 >= 0.0 {
            break;
        }

        zeroi += 1;
    }

    if zeroi == stream_.len() {
        zeroi = UP_COUNT;

        while zeroi < stream_.len() {
            let smp1 = stream_[zeroi].y;
            let smp2 = stream_[zeroi - 2].y;

            if smp1 < 0.0 && smp2 >= 0.0 {
                break;
            }

            zeroi += 1;
        }
    }

    if zeroi == stream_.len() {
        zeroi = UP_COUNT;
    } else {
        for rest in stream_.iter().skip(zeroi + 1) {
            bass += rest.l1_norm();
        }
    }

    let wave_scale_factor = (bass / (stream_.len() as f32)) * 13.0 + 2.0;

    let wave_scale_factor_old = *WAVE_SCALE_FACTOR.lock().unwrap();

    let wave_scale_factor =
        math::interpolate::subtractive_fall(wave_scale_factor_old, wave_scale_factor, 1.0, 0.5);

    *WAVE_SCALE_FACTOR.lock().unwrap() = wave_scale_factor;

    let mut smoothed_smp = stream[zeroi];

    for x in 0..4 {
        let di = x as usize * wave_scale_factor as usize + zeroi;
        smoothed_smp = crate::math::interpolate::linearfc(smoothed_smp, stream[di], 0.33);
    }

    for x in 4..prog.pix.width() as i32 + 4 {
        let di = x as usize * wave_scale_factor as usize + zeroi;

        smoothed_smp = crate::math::interpolate::linearfc(smoothed_smp, stream[di], 0.33);

        let y1 = height_top_h + (smoothed_smp.x * scale) as i32;
        let y2 = height_top_h + (smoothed_smp.y * scale) as i32;

        let x = x - 4;
        prog.pix
            .set_pixel_xy_by(P2::new(x, y1), 0xFF_55_FF_55, |a, b| a | b);
        prog.pix
            .set_pixel_xy_by(P2::new(x, y2), 0xFF_55_55_FF, |a, b| a | b);
    }

    let li = (LOCALI.load(Relaxed) + prog.pix.width() * 3 / 2 + 1) % prog.pix.width();

    prog.pix.draw_rect_wh(
        P2::new(li as i32, height / 10),
        1,
        prog.pix.height() - prog.pix.height() / 4,
        CROSS_COL,
    );

    prog.pix.draw_rect_wh(
        P2::new(li as i32, height / 2),
        prog.pix.width() >> 3,
        1,
        CROSS_COL,
    );

    stream.rotate_left(200);
    LOCALI.store(li, Relaxed);
}

pub fn draw_oscilloscope2(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let l = stream.input_size();

    let (width, height) = prog.pix.sizet();

    let _width_top_h = width >> 1;
    let height_top_h = height >> 1;

    let scale = prog.pix.height() as f32 * prog.vol_scl * 0.45;

    prog.pix.clear();

    let mut sum = 0.0;
    let mut smp = stream[0];

    let mut zero_x = 0;
    let mut zero_y = 0;
    let mut max_x = -100.0;
    let mut max_y = -100.0;

    for i in 1..l {
        smp = crate::math::interpolate::linearfc(smp, stream[i], 0.05);

        if smp.x > max_x {
            zero_x = i;
            max_x = smp.x;
        }

        if smp.y > max_y {
            zero_y = i;
            max_y = smp.y;
        }

        sum += smp.x * smp.x + smp.y * smp.y;
    }

    let zeroi = if zero_x > 5 { zero_x } else { zero_y };

    let bass = (sum / l as f32).sqrt();

    let wave_scale_factor = bass * 13.0 + 2.0;

    let wave_scale_factor_old = *WAVE_SCALE_FACTOR.lock().unwrap();

    let wave_scale_factor =
        math::interpolate::subtractive_fall(wave_scale_factor_old, wave_scale_factor, 1.0, 0.5);

    *WAVE_SCALE_FACTOR.lock().unwrap() = wave_scale_factor;

    let mut smoothed_smp = stream[zeroi];

    for x in 0..4 {
        let di = x as usize * wave_scale_factor as usize + zeroi;
        smoothed_smp = crate::math::interpolate::linearfc(smoothed_smp, stream[di], 0.25);
    }

    for x in 4..prog.pix.width() as i32 + 4 {
        let di = x as usize * wave_scale_factor as usize + zeroi;

        smoothed_smp = crate::math::interpolate::linearfc(smoothed_smp, stream[di], 0.25);

        let y1 = height_top_h + (smoothed_smp.x * scale) as i32;
        let y2 = height_top_h + (smoothed_smp.y * scale) as i32;

        let x = x - 4;

        prog.pix
            .set_pixel_xy_by(P2::new(x, y1), 0xFF_55_FF_55, |a, b| a | b);
        prog.pix
            .set_pixel_xy_by(P2::new(x, y2), 0xFF_55_55_FF, |a, b| a | b);
    }

    let mut li = LOCALI.load(Relaxed);
    let random = math::rng::random_int(127) as usize;

    li = (li + math::rng::faster_random_int(random, li, 3) + 3) % prog.pix.width();

    let rx = width - li as i32 - 1;

    prog.pix.draw_rect_wh(
        P2::new(rx, height / 10),
        1,
        prog.pix.height() - prog.pix.height() / 4,
        CROSS_COL,
    );

    prog.pix
        .draw_rect_wh(P2::new(rx, height / 2), prog.pix.width() >> 3, 1, CROSS_COL);

    stream.rotate_left(zeroi);
    LOCALI.store(li, Relaxed);
}

use crate::math::interpolate::linearfc;

pub fn draw_oscilloscope3(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let Ok(mut wave_scale_factor) = WAVE_SCALE_FACTOR.try_lock() else {
        return;
    };

    let l = stream.input_size() * 2 / 3;
    let (width, height) = prog.pix.sizet();
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

    let mut zeros = [0].to_vec();

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

        let mut smp_max = Cplx::new(-1.0, -1.0);
        let mut smp_min = Cplx::new(1.0, 1.0);

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

        prog.pix
            .draw_line_by(p1_max, p1_min, 0xFF_55_FF_55, |a, b| a | b);
        prog.pix
            .draw_line_by(p2_max, p2_min, 0xFF_55_55_FF, |a, b| a | b);
    }

    let mut li = LOCALI.load(Relaxed);
    let random = math::rng::random_int(127) as usize;

    li = (li + math::rng::faster_random_int(random, li, 3) + 3) % prog.pix.width();

    let rx = width - li as i32 - 1;

    prog.pix.draw_rect_wh(
        P2::new(rx, height / 10),
        1,
        prog.pix.height() - prog.pix.height() / 4,
        CROSS_COL,
    );

    prog.pix
        .draw_rect_wh(P2::new(rx, height / 2), prog.pix.width() >> 3, 1, CROSS_COL);

    LOCALI.store(li, Relaxed);

    stream.rotate_left(zeroi / 2);
}
