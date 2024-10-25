use crate::data::DEFAULT_ROTATE_SIZE;
use crate::math::Cplx;

const SILENCE_LIMIT: f32 = 0.001;
const AMP_PERSIST_LIMIT: f32 = 0.05;
const AMP_TRIGGER_THRESHOLD: f32 = 0.85;
const SILENCE_CHECK_SIZE: u8 = 24;

pub const BUFFER_CAPACITY: usize = 6210;

const REACT_SPEED: f32 = 0.025;

/// This is a struct that acts like a regular buffer
/// but uses an offset index.
///
/// By default, coffeevis displays at 144hz, but cpal can't
/// send input data that quickly between each rendering.
/// Moreover, the visualizers don't use all of the
/// data sent in in one rendering. Therefore, one
/// solution is to use one slice of then rotate
/// it to get to the next one.
///
/// AudioBuffer used the mentioned offset index to
/// simulate rotating and bypass having to move elemenents.
///
///
/// To index as performantly as possible, AudioBuffer only allows
/// powers of 2 lengths.
pub struct AudioBuffer {
    pub buffer: [Cplx; BUFFER_CAPACITY],

    size_mask: usize,
    /// Where offset [0] starts
    offset: usize,
    /// To prevent "audio tearing" when writing input,
    /// `write_point` tells where the last write happened.
    /// Ensures that new data is written right after where
    /// the old one was.
    write_point_old: usize,
    write_point: usize,

    rotate_size: usize,

    rotates_since_write: usize,
    average_rotates_since_write: usize,

    input_size: usize,

    max: f32,
    average: f32,
    silent: u8,
}

impl std::ops::Index<usize> for AudioBuffer {
    type Output = Cplx;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            self.buffer
                .get_unchecked(index.wrapping_add(self.offset) & self.size_mask)
        }
    }
}

impl std::ops::IndexMut<usize> for AudioBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe {
            self.buffer
                .get_unchecked_mut(index.wrapping_add(self.offset) & self.size_mask)
        }
    }
}

fn write_sample<T: cpal::Sample<Float = f32>>(smp: &mut Cplx, smp_in: &[T]) {
    smp.x = smp_in[0].to_float_sample();
    smp.y = smp_in[1].to_float_sample();
}

impl AudioBuffer {
    pub const fn new() -> Self {

        let size_mask: usize = BUFFER_CAPACITY.next_power_of_two()/2 - 1;

        Self {
            buffer: [Cplx::zero(); BUFFER_CAPACITY],
            offset: 0,
            write_point_old: 0,
            write_point: 0,
            input_size: 1000,

            size_mask,

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

    pub fn peak(&self) -> f32 {
        self.max
    }

    pub fn average(&self) -> f32 {
        self.average
    }

    pub fn readpoint(&self) -> usize {
        self.offset
    }

    pub fn is_silent_for(&self, x: u8) -> bool {
        self.silent > x
    }

    pub fn silent(&self) -> u8 {
        self.silent
    }

    pub fn normalize_factor_peak(&self) -> f32 {
        const MAX_FACTOR: f32 = AMP_TRIGGER_THRESHOLD / AMP_PERSIST_LIMIT;

        if self.max > AMP_TRIGGER_THRESHOLD {
            return 1.0;
        }

        if self.max < AMP_PERSIST_LIMIT {
            return MAX_FACTOR;
        }

        AMP_TRIGGER_THRESHOLD / self.max
    }

    pub fn to_vec(&mut self) -> Vec<Cplx> {
        let mut o = vec![Cplx::zero(); self.buffer.len()];
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

        // self.write_point_old = self.write_point;

        let mut max = 0.0f32;

        let stop_reading = self.is_silent_for(
            (self.size_mask / self.input_size).min(255) as u8
        );

        let mut silent_samples = 0;

        for chunk in data.chunks_exact(2) {
            let smp = unsafe { self.buffer.get_unchecked_mut(self.write_point) };
            write_sample(smp, chunk);

            max = max.max(smp.x.abs()).max(smp.y.abs());

            silent_samples += (max < SILENCE_LIMIT) as u8;

            // Only check the first SILENCE_CHECK_SIZE samples
            if stop_reading {
                if silent_samples >= SILENCE_CHECK_SIZE {break}
                silent_samples += (max < SILENCE_LIMIT) as u8;
            }

            self.write_point = self.index_add(self.write_point, 1);
        }

        self.max = crate::math::interpolate::multiplicative_fall(
            self.max,
            max,
            AMP_PERSIST_LIMIT,
            REACT_SPEED,
        );

        self.post_process(max < SILENCE_LIMIT)
    }

    pub fn post_process(&mut self, silent: bool) {

        self.offset = self.index_sub(self.write_point, self.input_size * 2);

        self.silent = if silent {
            self.silent.saturating_add(1)
        } else {
            0
        };

        self.average_rotates_since_write =
            (self.average_rotates_since_write + self.rotates_since_write).max(2) / 2;

        self.rotates_since_write = 0;

        self.rotate_size = self.input_size / self.average_rotates_since_write + 1;
    }

    pub fn checked_normalize(&mut self) {
        let scale_up_factor = self.normalize_factor_peak();

        if self.is_silent_for(0) || scale_up_factor <= 1.0 {
            return;
        }

        let mut write_point = self.index_sub(self.write_point, self.input_size);

        for _ in 0..self.input_size {
            let smp = unsafe { self.buffer.get_unchecked_mut(write_point) };
            *smp *= scale_up_factor;
            write_point = self.index_add(write_point, 1);
        }
    }
}
