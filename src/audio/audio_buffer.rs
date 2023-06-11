use cpal;
use crate::math::Cplx;
use crate::data::SAMPLE_SIZE;
use std::{iter, sync::atomic::AtomicBool};

const SILENCE_LIMIT: f32 = 0.01;
const AMP_PERSIST_LIMIT: f32 = 0.05;
const AMP_TRIGGER_THRESHOLD: f32 = 0.85;
const SILENCE_INDEX: u32 = 7;

const BUFFER_SIZE: usize = 1 << 12;
const SIZE_MASK: usize = BUFFER_SIZE - 1;

type BufferArray = [Cplx<f32>; BUFFER_SIZE];

// pub static EXPANDER: 

/// This is a struct that acts like a regular buffer 
/// but uses an offset index point to prevent moving 
/// elements when rotating buffer.
/// 
/// By default, kvis displays at 144hz, but cpal can't
/// send input data that quickly between each rendering.
/// Besides that, the visualizers don't use all of the
/// data sent in in one rendering. Therefore, one 
/// solution was to use the first partial slice of the 
/// buffer, then rotate it.
/// 
/// But that will cost a lot of performance because each 
/// rotation requires moves and possibly allocations.
///
/// AudioBuffer is used to prevent unnecessary performance
/// cost. 
///
/// Powers of 2 should be chosen for the buffer size
/// size to allow for the fastest index wrapping methods.
pub struct AudioBuffer {
    buffer: BufferArray,
    /// Tells where the "first index" should be.
    offset: usize,
    /// To prevent "audio tearing" when writing input,
    /// `write_point` tells where the last write happened.
    /// Ensures that new data is written right after where
    /// the old one was. 
    write_point: usize,
    
    /// Experimental feature.
    /// 
    /// kvis is configured to run with music, which is loud. 
    /// Sometimes regular audio might be too quiet.
    /// `normalizer` analyzes the current write and tries to normalize 
    /// the amplitude.
    /// Amplitude must be below 0.5 for `normalizer` to start scaling up. 
    normalizer: f32,
    max: f32,
    average: f32
}

pub struct AudioBufferIterator<'a> {
    reference: &'a AudioBuffer,
    index: usize,
    take: usize
}

impl<'a> Iterator for AudioBufferIterator<'a> {
    type Item = Cplx<f32>;

    fn next(&mut self) -> Option<Cplx<f32>> {
        if self.index >= self.take {return None}
        let o = self.reference[self.index];
        self.index = self.index.wrapping_add(1);
        Some(o)
    }
}

impl std::ops::Index<usize> for AudioBuffer {
    type Output = Cplx<f32>;
    fn index(&self, index: usize) -> &Self::Output {
        /// Unsafe allowed because this indexing cannot fail. 
        unsafe{self.buffer.get_unchecked(index.wrapping_add(self.offset)&SIZE_MASK)}
    }
}

impl std::ops::IndexMut<usize> for AudioBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe{self.buffer.get_unchecked_mut(index.wrapping_add(self.offset)&SIZE_MASK)}
    } 
}

use std::ops::Range;

fn write_sample<T: cpal::Sample<Float = f32>>(smp: &mut Cplx<f32>, smp_in: &[T]) {
    smp.x = smp_in[0].to_float_sample();
    smp.y = smp_in[1].to_float_sample();
}

impl AudioBuffer {
    pub const fn new() -> Self {
        Self {
            buffer: [Cplx::<f32>::zero(); BUFFER_SIZE],
            offset: 0,
            write_point: 0,
            normalizer: 1.0,
            max: 0.0,
            average: 0.0,
        }
    }

    pub fn iter<'a>(&'a self) -> AudioBufferIterator<'a> {
        AudioBufferIterator {
            reference: &self,
            index: 0,
            take: self.buffer.len()
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn rotate_left(&mut self, n: usize) {
        self.offset = self.offset.wrapping_add(n) & SIZE_MASK;
    }

    pub fn rotate_right(&mut self, n: usize) {
        self.offset = self.offset.wrapping_add(BUFFER_SIZE).wrapping_sub(n) & SIZE_MASK;
    }

    pub fn amplitude(&self) -> f32 {
        self.max
    }

    pub fn average_volume(&self) -> f32 {
        self.average
    }
    
    pub fn to_vec(&mut self) -> Vec<Cplx<f32>> {
        let mut o = vec![Cplx::<f32>::zero(); BUFFER_SIZE];
        o
        .iter_mut()
        .zip(self.buffer.iter().cycle().skip(self.offset))
        .for_each(|(out, inp)| *out = *inp);
        o
    }

    #[doc(hidden)]
    pub fn reset_offset(&mut self) {
        self.offset = 0;
    }

	/*
    /// Ignores `write_point`. Rotates the buffer and writes to the start.
    /// Returns boolean indicating silence. 
    pub fn read_from_input<T: cpal::Sample<Float = f32>>(&mut self, data: &[T]) -> bool {
        let input_size = data.len();
        let mut silence = true;
        self.buffer.rotate_right(input_size);
                
        data
        .chunks_exact(2)
        .zip(self.buffer.iter_mut())
        .for_each(|(inp, smp)| {
            write_sample(smp, inp);
            silence = silence && (smp.x < 1e-2) && (smp.y < 1e-2);
        });
        self.reset_offset();
        silence
    }*/

    /// Writes to buffer from the write_point onwards instead of rotating the buffer.
    /// Returns boolean indicating silence.
    pub fn read_from_input<T: cpal::Sample<Float = f32>>(&mut self, data: &[T]) -> bool {
        let input_size = data.len();
        let mut silence_index: u32 = 0;
        
        self.offset = self.write_point;
        self.write_point = (self.write_point + input_size) & SIZE_MASK;

        let mut di = self.write_point;
        data
        .chunks_exact(2)
        .enumerate()
        .for_each(|(i, chunk)| {
            let mut smp = &mut self.buffer[di];
            write_sample(smp, chunk);
            silence_index += ((smp.x > SILENCE_LIMIT) || (smp.y > SILENCE_LIMIT)) as u32;
            di = crate::math::increment_index(di, BUFFER_SIZE);
        });
        
        silence_index < SILENCE_INDEX
    }

    /// With `normalizer`
    pub fn read_from_input_with_normalizer<T: cpal::Sample<Float = f32>>(&mut self, data: &[T]) -> bool {
        let input_size = data.len();
        let mut silence_index: u32 = 0;
        
        self.offset = self.write_point;
        self.write_point = (self.write_point + input_size) & SIZE_MASK;

        let mut max = 0.0f32;

        let mut di = self.write_point;
        data
        .chunks_exact(2)
        .enumerate()
        .for_each(|(i, chunk)| {
            let mut smp = &mut self.buffer[di];
            write_sample(smp, chunk);
            
            max = max.max(smp.x.abs()).max(smp.y.abs());
            
            silence_index += ((smp.x > SILENCE_LIMIT) || (smp.y > SILENCE_LIMIT)) as u32;
            di = crate::math::increment_index(di, BUFFER_SIZE);
        });
        
        let silence = silence_index < SILENCE_INDEX;
        

        if silence {
            return silence;
        }
        
        self.max = 
            crate::math::interpolate::subtractive_fall(
                self.max,
                max,
                AMP_PERSIST_LIMIT,
                0.017
            );

        self.normalizer = AMP_TRIGGER_THRESHOLD / self.max.min(AMP_TRIGGER_THRESHOLD);
          
        di = self.write_point;
        for _ in 0..input_size/2 {
            let mut smp = &mut self.buffer[di];
            *smp = smp.scale(self.normalizer);
            di = crate::math::increment_index(di, BUFFER_SIZE);
        }
        
        silence
    }


    #[doc(hidden)]
    pub fn range(&self, index: Range<usize>) -> Vec<Cplx<f32>> {
        let range_ = index.start - index.end;
        let mut o = vec![Cplx::<f32>::zero(); range_];
        self.buffer
        .iter()
        .cycle()
        .skip(if index.start == 0 {0} else {index.end - 1})
        .take(range_)
        .zip(o.iter_mut())
        .for_each(|(inp, out)| *out = *inp);

        o
    }    
}
