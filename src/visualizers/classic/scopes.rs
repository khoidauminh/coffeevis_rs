use arrayvec::ArrayVec;

use crate::graphics::Pixel;

use crate::audio::MovingAverage;

use crate::graphics::P2;
use crate::math::interpolate::linearfc;
use crate::visualizers::Visualizer;
use crate::visualizers::classic::cross::draw_cross;

use crate::math::Cplx;

pub const INCREMENT: usize = 2;
pub const DEFAULT_WAV_WIN: usize = 64 * INCREMENT;
pub const PHASE_OFFSET: usize = crate::data::SAMPLE_RATE / 50 / 4;

const SMOOTH_SIZE: usize = 7;
const SMOOTH_BUFFER_SIZE: usize = 512;

fn to_color(s: i32, size: i32) -> u8 {
    (s.abs() * 256 / size).min(255) as u8
}

pub struct Vectorscope;

impl Visualizer for Vectorscope {
    fn name(&self) -> &'static str {
        "Vectorscope"
    }

    fn perform(
        &mut self,
        pix: &mut crate::graphics::PixelBuffer,
        stream: &mut crate::audio::AudioBuffer,
    ) {
        let size = pix.height().min(pix.width()) as i32;
        let sizei = size;
        let scale = size as f32 * 0.5;

        let P2(width, height) = pix.size();

        let width_top_h = width >> 1;
        let height_top_h = height >> 1;

        let mut di: usize = 0;

        pix.clear();

        let mut smoothed_sample = MovingAverage::<_, SMOOTH_SIZE>::init(Cplx::zero());

        for _ in 0..SMOOTH_SIZE {
            let sample = Cplx::new(stream.get(di).0, stream.get(di + PHASE_OFFSET).1);
            _ = smoothed_sample.update(sample);
            di += INCREMENT;
        }

        while di < DEFAULT_WAV_WIN {
            let sample = Cplx::new(stream.get(di).0, stream.get(di + PHASE_OFFSET).1);

            let sample = smoothed_sample.update(sample);

            let x = (sample.0 * scale) as i32;
            let y = (sample.1 * scale) as i32;
            let amp = (x.abs() + y.abs()) * 3 / 2;

            pix.color(u32::from_be_bytes([255, to_color(amp, sizei), 255, 64]));
            pix.mixerd();
            pix.plot(P2(x + width_top_h, y + height_top_h));

            di += INCREMENT;
        }

        draw_cross(pix);

        stream.autoslide();
    }
}

const BUFFER_SIZE: usize = 1024;
const PADDING: usize = BUFFER_SIZE;
const START: usize = BUFFER_SIZE / 2;
const PRESMOOTH: usize = BUFFER_SIZE / 16;
const LOWPASS_FACTOR: f32 = 0.01;
const SHIFTBACK: usize = 50;
const STORESIZE: usize = 6;

pub struct Oscilloscope;

impl Visualizer for Oscilloscope {
    fn name(&self) -> &'static str {
        "Oscilloscope"
    }

    fn perform(
        &mut self,
        pix: &mut crate::graphics::PixelBuffer,
        stream: &mut crate::audio::AudioBuffer,
    ) {
        let mut buffer = [Cplx::zero(); BUFFER_SIZE + PADDING];
        stream.read(&mut buffer);

        let mut indices = ArrayVec::<usize, STORESIZE>::new();
        let mut smp1 = Cplx::zero();
        let mut smp2 = Cplx::zero();
        let mut smp3 = Cplx::zero();

        for &smp in buffer.iter().take(START).skip(START - PRESMOOTH) {
            smp3 = linearfc(smp3, smp, LOWPASS_FACTOR);
            smp1 = smp2;
            smp2 = smp3;
        }

        indices.push(START);
        for (i, &smp) in buffer.iter().enumerate().skip(START).take(PADDING) {
            if smp1.0 >= 0.0 && smp3.0 < 0.0 {
                indices.push(i);
            }

            if indices.is_full() {
                break;
            }

            if smp1.0 >= 0.0 && smp3.0 < 0.0 {
                indices.push(i);
            }

            if indices.is_full() {
                break;
            }

            smp3 = linearfc(smp3, smp, LOWPASS_FACTOR);
            smp1 = smp2;
            smp2 = smp3;
        }

        let indexstart = indices.last().unwrap() - START - SHIFTBACK;

        stream.autoslide();

        pix.clear();

        let size = pix.size();

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

            for smp in buffer.iter().take(iend).skip(istart) {
                let l = smp.0;
                let r = smp.1;

                lmin = lmin.min(l);
                lmax = lmax.max(l);

                rmin = rmin.min(r);
                rmax = rmax.max(r);
            }

            lmin = lmin * scale + center;
            lmax = lmax * scale + center;

            rmin = rmin * scale + center;
            rmax = rmax * scale + center;

            pix.mixer(u32::or);
            pix.color(u32::compose([255, 0, 55, 255]));
            pix.rect(P2(x as i32, lmin as i32), 1, (lmax - lmin) as usize);
            pix.color(u32::compose([255, 0, 255, 55]));
            pix.rect(P2(x as i32, rmin as i32), 1, (rmax - rmin) as usize);
        }
    }
}
