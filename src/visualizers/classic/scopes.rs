use std::sync::{
    Mutex,
    atomic::AtomicUsize,
};

use arrayvec::ArrayVec;

use crate::graphics::Pixel;

use crate::audio::MovingAverage;
use crate::data::{INCREMENT, PHASE_OFFSET};
use crate::graphics::P2;
use crate::math::interpolate::linearfc;
use crate::visualizers::classic::cross::draw_cross;

use crate::math::Cplx;

static LOCALI: AtomicUsize = AtomicUsize::new(0);
static WAVE_SCALE_FACTOR: Mutex<f32> = Mutex::new(1.0);
const SMOOTH_SIZE: usize = 7;
const SMOOTH_BUFFER_SIZE: usize = 512;

fn to_color(s: i32, size: i32) -> u8 {
    (s.abs() * 256 / size).min(255) as u8
}

pub fn draw_vectorscope(prog: &mut crate::Program, stream: &mut crate::AudioBuffer) {
    let range = prog.wav_win;
    let _l = stream.len();

    let size = prog.pix.height().min(prog.pix.width()) as i32;
    let sizei = size;
    let scale = size as f32 * prog.vol_scl * 0.5;

    let P2(width, height) = prog.pix.size();

    let width_top_h = width >> 1;
    let height_top_h = height >> 1;

    let mut di: usize = 0;

    prog.pix.clear();

    let mut smoothed_sample = MovingAverage::<_, SMOOTH_SIZE>::init(Cplx::zero());

    for _ in 0..SMOOTH_SIZE {
        let sample = Cplx::new(stream.get(di).0, stream.get(di + PHASE_OFFSET).1);
        _ = smoothed_sample.update(sample);
        di += INCREMENT;
    }

    while di < range {
        let sample = Cplx::new(stream.get(di).0, stream.get(di + PHASE_OFFSET).1);

        let sample = smoothed_sample.update(sample);

        let x = (sample.0 * scale) as i32;
        let y = (sample.1 * scale) as i32;
        let amp = (x.abs() + y.abs()) * 3 / 2;

        prog.pix.color(u32::from_be_bytes([255, to_color(amp, sizei), 255, 64]));
        prog.pix.mixerd();
        prog.pix.plot(P2(x + width_top_h, y + height_top_h));

        di += INCREMENT;
    }

    draw_cross(prog);

    stream.autoslide();
}

// pub fn draw_oscilloscope(prog: &mut crate::Program, stream: &mut crate::AudioBuffer) {
//     let Ok(mut wave_scale_factor) = WAVE_SCALE_FACTOR.try_lock() else {
//         return;
//     };

//     let l = stream.input_size() * 2 / 3;
//     let (width, height) = prog.pix.size().as_tuple();
//     let width_top_h = width / 2;
//     let height_top_h = height / 2;

//     let scale = prog.pix.height() as f32 * prog.vol_scl * 0.45;

//     let mut bass_sum = 0.0;

//     let mut i = (-20_isize) as usize;

//     let mut ssmp = stream.get(i);
//     let mut old = ssmp;
//     let mut old2 = old;

//     while i != 0 {
//         ssmp = linearfc(ssmp, stream.get(i), 0.01);
//         old2 = old;
//         old = ssmp;
//         i = i.wrapping_add(1);
//     }

//     let zeroi = {
//         let mut zeros = ArrayVec::<usize, 6>::new();
//         zeros.push(0);

//         for i in 0..l {
//             let t = stream.get(i);
//             ssmp = linearfc(ssmp, t, 0.01);

//             if zeros.len() < 5 {
//                 if old2.0 > 0.0 && ssmp.0 < 0.0 {
//                     zeros.push(i);
//                 }
//                 if old2.1 > 0.0 && ssmp.1 < 0.0 {
//                     zeros.push(i);
//                 }
//             }

//             old2 = old;
//             old = ssmp;

//             bass_sum += ssmp.0.powi(2) + ssmp.1.powi(2);
//         }

//         zeros[zeros.len() - 1] - 50
//     };

//     let bass = (bass_sum / l as f32).sqrt();

//     let wsf = bass * 20.0 + 2.0;

//     *wave_scale_factor = math::interpolate::linear_decay(*wave_scale_factor, wsf, 0.5).max(2.0);

//     prog.pix.clear();

//     let wave_scale_factor = *wave_scale_factor as isize;

//     for x in 0..prog.pix.width() as i32 {
//         let di = (x - width_top_h) as isize * wave_scale_factor + zeroi as isize;

//         let mut smp_max = Cplx::new(-1000.0, -1000.0);
//         let mut smp_min = Cplx::new(1000.0, 1000.0);

//         for i in di..di + wave_scale_factor {
//             let smp = stream.get(i as usize);

//             smp_max.0 = smp_max.0.max(smp.0);
//             smp_min.0 = smp_min.0.min(smp.0);

//             smp_max.1 = smp_max.1.max(smp.1);
//             smp_min.1 = smp_min.1.min(smp.1);
//         }

//         let y1_max = height_top_h + (smp_max.0 * scale) as i32;
//         let y1_min = height_top_h + (smp_min.0 * scale) as i32;

//         let y2_max = height_top_h + (smp_max.1 * scale) as i32;
//         let y2_min = height_top_h + (smp_min.1 * scale) as i32;

//         let p1_max = P2::new(x, y1_max);
//         let p1_min = P2::new(x, y1_min);

//         let p2_max = P2::new(x, y2_max);
//         let p2_min = P2::new(x, y2_min);

//         prog.pix.line(p1_max, p1_min, 0xFF_55_FF_55, |a, b| a | b);
//         prog.pix.line(p2_max, p2_min, 0xFF_55_55_FF, |a, b| a | b);
//     }

//     let mut li = LOCALI.load(Relaxed);
//     let random = math::rng::random_int(3) as usize;

//     li = (li + random + 3) % prog.pix.width();

//     let rx = width - li as i32 - 1;

//     let width = width as usize;
//     let height = height as usize;

//     prog.pix.rect_wh(
//         P2::new(rx, height / 10),
//         1,
//         width - height / 4,
//         CROSS_COL,
//         Pixel::over,
//     );

//     prog.pix.rect_wh(
//         P2::new(rx, height / 2),
//         width >> 3,
//         1,
//         CROSS_COL,
//         Pixel::over,
//     );

//     LOCALI.store(li, Relaxed);

//     stream.autoslide();
// }


const BUFFER_SIZE: usize = 1024;
const PADDING: usize = BUFFER_SIZE;
const START: usize = BUFFER_SIZE / 2;
const PRESMOOTH: usize = BUFFER_SIZE / 16;
const LOWPASS_FACTOR: f32 = 0.01;
const SHIFTBACK: usize = 50;
const STORESIZE: usize = 6;

pub fn draw_oscilloscope(prog: &mut crate::Program, stream: &mut crate::AudioBuffer) {
    let mut buffer= [Cplx::zero(); BUFFER_SIZE + PADDING];
    stream.read(&mut buffer);

    let mut indices = ArrayVec::<usize, STORESIZE>::new();
    let mut smp1 = Cplx::zero();
    let mut smp2 = Cplx::zero();
    let mut smp3 = Cplx::zero();

    for i in START-PRESMOOTH..START {
        smp3 = linearfc(smp3, buffer[i], LOWPASS_FACTOR);
        smp1 = smp2;
        smp2 = smp3;
    }

    indices.push(START);
    for i in START..PADDING + START {
        if smp1.0 >= 0.0 && smp3.0 < 0.0 {
            indices.push(i);
        }

        if indices.is_full() { break }

        if smp1.0 >= 0.0 && smp3.0 < 0.0 {
            indices.push(i);
        }

        if indices.is_full() { break }

        smp3 = linearfc(smp3, buffer[i], LOWPASS_FACTOR);
        smp1 = smp2;
        smp2 = smp3;
    }

    let indexstart = indices.last().unwrap() - START - SHIFTBACK;

    stream.autoslide();

    prog.pix.clear();

    let size = prog.pix.size();

    let center = size.1 as f32 * 0.5;
    let scale = center * 0.7;
    let w = size.0.max(1) as usize;

    let buffer_size_smaller = BUFFER_SIZE as f32 * 0.8;
    let index_scale = buffer_size_smaller / w as f32;
    let base = (BUFFER_SIZE as f32 * 0.1) as usize;

    let samplexperpixel = (BUFFER_SIZE + w) / w;

    for x in 0..size.0 as usize {
        let istart = (x as f32 * index_scale) as usize + indexstart + base;
        let iend = istart + samplexperpixel;

        let mut lmin = 100.0f32;
        let mut lmax = -100.0f32;
        let mut rmin = 100.0f32;
        let mut rmax = -100.0f32;

        for i in istart..iend {
            let l = buffer[i].0;
            let r = buffer[i].1;

            lmin = lmin.min(l);
            lmax = lmax.max(l);

            rmin = rmin.min(r);
            rmax = rmax.max(r);
        }

        lmin = lmin * scale + center;
        lmax = lmax * scale + center;

        rmin = rmin * scale + center;
        rmax = rmax * scale + center;

        prog.pix.mixer(|a, b| a | b);
        prog.pix.color(u32::compose([255, 0,  55, 255]));
        prog.pix.rect(P2(x as i32, lmin as i32), 1, (lmax - lmin) as usize);
        prog.pix.color(u32::compose([255, 0, 255, 55]));
        prog.pix.rect(P2(x as i32, rmin as i32), 1, (rmax - rmin) as usize);
    }
}