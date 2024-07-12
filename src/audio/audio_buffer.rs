use crate::data::DEFAULT_ROTATE_SIZE;
use crate::math::Cplx;

const SILENCE_LIMIT: f64 = 0.001;
const AMP_PERSIST_LIMIT: f64 = 0.05;
const AMP_TRIGGER_THRESHOLD: f64 = 0.85;
const SILENCE_INDEX: u16 = 24;

const BUFFER_SIZE_POWER: usize = crate::data::POWER;
const BUFFER_SIZE: usize = 1 << BUFFER_SIZE_POWER;
const SIZE_MASK: usize = BUFFER_SIZE - 1;

const REACT_SPEED: f64 = 0.025;

type BufferArray = [Cplx; BUFFER_SIZE];

/// This is a struct that acts like a regular buffer
/// but uses an offset index.
///
/// By default, coffeevis displays at 144hz, but cpal can't
/// send input data that quickly between each rendering.
/// Moreover, the visualizers don't use all of the
/// data sent in in one rendering. Therefore, one
/// solution was to use one slice of then rotate
/// it to get to the next one.
///
/// AudioBuffer used the mentioned the offset index to
/// simulate rotating and bypass having to move elemenents.
///
///
/// To index as performantly as possible, AudioBuffer only allows
/// powers of 2 lengths.
pub struct AudioBuffer {
    buffer: BufferArray,

    size: usize,
    size_mask: usize,
    /// Where offset [0] starts
    offset: usize,
    /// To prevent "audio tearing" when writing input,
    /// `write_point` tells where the last write happened.
    /// Ensures that new data is written right after where
    /// the old one was.
    write_point: usize,

    rotate_size: usize,

    rotates_since_write: usize,
    average_rotates_since_write: usize,

    input_size: usize,

    max: f64,
    average: f64,
    silent: u8,
}

impl std::ops::Index<usize> for AudioBuffer {
    type Output = Cplx;
    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[index.wrapping_add(self.offset) & SIZE_MASK]
    }
}

impl std::ops::IndexMut<usize> for AudioBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buffer[index.wrapping_add(self.offset) & SIZE_MASK]
    }
}

impl std::ops::Index<isize> for AudioBuffer {
    type Output = Cplx;
    fn index(&self, index: isize) -> &Self::Output {
        &self.buffer[self.offset.wrapping_add_signed(index) & SIZE_MASK]
    }
}

impl std::ops::IndexMut<isize> for AudioBuffer {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        &mut self.buffer[self.offset.wrapping_add_signed(index) & SIZE_MASK]
    }
}

use std::ops::Range;

fn write_sample<T: cpal::Sample<Float = f32>>(smp: &mut Cplx, smp_in: &[T]) {
    smp.x = smp_in[0].to_float_sample() as f64;
    smp.y = smp_in[1].to_float_sample() as f64;
}

impl AudioBuffer {
    pub const fn new() -> Self {
        Self {
            buffer: [Cplx::zero(); BUFFER_SIZE],
            offset: 0,
            write_point: 0,
            input_size: 1000,

            size: BUFFER_SIZE,
            size_mask: SIZE_MASK,

            rotate_size: DEFAULT_ROTATE_SIZE,
            rotates_since_write: 0,
            average_rotates_since_write: 1,

            max: 0.0,
            average: 0.0,
            silent: 0,
        }
    }

    pub fn as_slice(&self) -> &[Cplx] {
        &self.buffer
    }

    pub fn input_size(&self) -> usize {
        self.input_size
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn index_add(&self, a: usize, b: usize) -> usize {
        a.wrapping_add(b) & self.size_mask
    }

    pub fn index_sub(&self, a: usize, b: usize) -> usize {
        a.wrapping_sub(b) & self.size_mask
    }

    pub fn rotate_left(&mut self, n: usize) {
        self.offset = self.index_add(self.offset, n);
    }

    pub fn rotate_right(&mut self, n: usize) {
        self.offset = self.index_sub(self.offset, n);
    }

    pub fn peak(&self) -> f64 {
        self.max
    }

    pub fn average(&self) -> f64 {
        self.average
    }

    pub fn readpoint(&self) -> usize {
        self.offset
    }

    pub fn is_silent(&self, x: u8) -> bool {
        self.silent > x
    }

    pub fn silent(&self) -> u8 {
        self.silent
    }

    pub fn normalize_factor_peak(&self) -> f64 {
        const MAX_FACTOR: f64 = AMP_TRIGGER_THRESHOLD / AMP_PERSIST_LIMIT;

        if self.max > AMP_TRIGGER_THRESHOLD {
            1.0
        } else if self.max < AMP_PERSIST_LIMIT {
            MAX_FACTOR
        } else {
            AMP_TRIGGER_THRESHOLD / self.max
        }
    }

    pub fn normalize_factor_average(&self) -> f64 {
        AMP_TRIGGER_THRESHOLD
            / self
                .average
                .min(AMP_TRIGGER_THRESHOLD)
                .max(AMP_PERSIST_LIMIT)
    }

    pub fn to_vec(&mut self) -> Vec<Cplx> {
        let mut o = vec![Cplx::zero(); BUFFER_SIZE];
        o.iter_mut()
            .zip(self.buffer.iter().cycle().skip(self.offset))
            .for_each(|(out, inp)| *out = *inp);
        o
    }

    pub fn auto_rotate(&mut self) {
        self.rotate_left(self.rotate_size);
        self.rotates_since_write += 1;
    }

    #[doc(hidden)]
    pub fn reset_offset(&mut self) {
        self.offset = 0;
    }

    pub fn set_to_writepoint(&mut self) {
        self.offset = self.write_point;
    }

    pub fn read_from_input<T: cpal::Sample<Float = f32>>(&mut self, data: &[T]) {
        let input_size = data.len();
        self.input_size = input_size / 2;

        let mut max_l = 0.0f64;
        let mut max_r = 0.0f64;

        //self.write_point = self.index_add(self.write_point, self.input_size);
        //let mut write_point = self.write_point;

        let mut silent_samples = 0u16;

        // Stop reading once the input is quiet enough to fill the buffer with "zeros".
        let stop_reading = self.is_silent((BUFFER_SIZE / self.input_size).min(255) as u8);

        for chunk in data.chunks_exact(2) {
            let smp = &mut self.buffer[self.write_point];
            write_sample(smp, chunk);

            let left = smp.x.abs();
            let right = smp.y.abs();

            max_l = max_l.max(left);
            max_r = max_r.max(right);

            silent_samples += (max_l < SILENCE_LIMIT && max_r < SILENCE_LIMIT) as u16;

            // Only check the first SILENCE_INDEX samples
            if silent_samples >= SILENCE_INDEX && stop_reading {
                break;
            }

            self.write_point = self.index_add(self.write_point, 1);
        }

        // self.write_point = write_point;

        let max = max_r.max(max_l);

        self.max = crate::math::interpolate::multiplicative_fall(
            self.max,
            max,
            AMP_PERSIST_LIMIT,
            REACT_SPEED,
        );

        self.post_process(max < SILENCE_LIMIT)
    }

    pub fn post_process(&mut self, silent: bool) {
        const BSIZE_HALF: usize = BUFFER_SIZE / 2;

        if self.input_size >= BSIZE_HALF {
            self.offset = self.write_point;
        } else {
            self.offset = self.index_sub(self.write_point, self.input_size * 2);
        }

        if silent {
            self.silent = self.silent.saturating_add(1);
        } else {
            self.silent = 0;
        }

        self.average_rotates_since_write =
            (self.average_rotates_since_write + self.rotates_since_write).max(2) / 2;

        self.rotates_since_write = 0;

        self.rotate_size = self.input_size / self.average_rotates_since_write + 1;
    }

    pub fn checked_normalize(&mut self) {
        let scale_up_factor = self.normalize_factor_peak();

        if self.is_silent(0) || scale_up_factor <= 1.0 {
            return;
        }

        let mut write_point = self.index_sub(self.write_point, self.input_size);

        for _ in 0..self.input_size {
            let smp = &mut self.buffer[write_point];
            *smp = smp.scale(scale_up_factor);
            write_point = self.index_add(write_point, 1);
        }
    }

    pub fn checked_fill_zero(&mut self) {
        if self.is_silent(3) {
            self.buffer.fill(Cplx::zero())
        }
    }

    #[doc(hidden)]
    pub fn range(&self, index: Range<usize>) -> Vec<Cplx> {
        let range_ = index.start - index.end;
        let mut o = vec![Cplx::zero(); range_];
        self.buffer
            .iter()
            .cycle()
            .skip(if index.start == 0 { 0 } else { index.end - 1 })
            .take(range_)
            .zip(o.iter_mut())
            .for_each(|(inp, out)| *out = *inp);

        o
    }
}
