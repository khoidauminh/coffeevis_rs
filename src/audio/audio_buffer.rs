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
/// but uses an offset index.
/// 
/// By default, coffeevis displays at 144hz, but cpal can't
/// send input data that quickly between each rendering.
/// Moreover, the visualizers don't use all of the
/// data sent in in one rendering. Therefore, one 
/// solution was to use the a slice of the 
/// buffer, then rotate it to get to the next one.
///
/// AudioBuffer used the mentioned the offset index to
/// simulate rotating and bypass having to move elemenents.
/// 
///
/// To index as performantly as possible, AudioBuffer only allows 
/// powers of 2 lengths.
pub struct AudioBuffer {
    buffer: BufferArray,
    /// Where offset [0] starts
    offset: usize,
    /// To prevent "audio tearing" when writing input,
    /// `write_point` tells where the last write happened.
    /// Ensures that new data is written right after where
    /// the old one was. 
    write_point: usize,
    
    /// Experimental feature.
    /// 
    /// Coffeevis normalizes the audio input when it's too quiet
    /// so that the visualizers doesn't get boring. 
    /// Triggers when amplitude < AMP_TRIGGER_THRESHOLD
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
        /// Unsafe allowed because this cannot fail. 
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
