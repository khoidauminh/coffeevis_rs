
use crate::math::{Cplx, fast};
use crate::data::{DEFAULT_ROTATE_SIZE};


const SILENCE_LIMIT: f64 = 0.01;
const AMP_PERSIST_LIMIT: f64 = 0.05;
const AMP_TRIGGER_THRESHOLD: f64 = 0.85;
const SILENCE_INDEX: u32 = 7;

const BUFFER_SIZE_POWER: usize = crate::data::POWER;
const BUFFER_SIZE: usize = 1 << BUFFER_SIZE_POWER;
const SIZE_MASK: usize = BUFFER_SIZE - 1;

const REACT_SPEED: f64 = 0.025;

type BufferArray = [Cplx; BUFFER_SIZE];

// pub static EXPANDER:

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

    input_size: usize,

    max: f64,
    average: f64,
    silent: bool
}

pub struct AudioBufferIterator<'a> {
    reference: &'a AudioBuffer,
    index: usize,
    take: usize
}

impl<'a> Iterator for AudioBufferIterator<'a> {
    type Item = Cplx;

    fn next(&mut self) -> Option<Cplx> {
        if self.index >= self.take {return None}
        let o = self.reference[self.index];
        self.index = self.index.wrapping_add(1);
        Some(o)
    }
}

impl std::ops::Index<usize> for AudioBuffer {
    type Output = Cplx;
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
            
            max: 0.0,
            average: 0.0,
            silent: true
        }
    }

    pub fn iter(&self) -> AudioBufferIterator<'_> {
        AudioBufferIterator {
            reference: self,
            index: 0,
            take: self.buffer.len()
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
		// let c = if b > BUFFER_SIZE { (b/BUFFER_SIZE) * BUFFER_SIZE } else { 0 };
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
		AMP_TRIGGER_THRESHOLD / 
			self.average
			.min(AMP_TRIGGER_THRESHOLD)
			.max(AMP_PERSIST_LIMIT)
	}

    pub fn to_vec(&mut self) -> Vec<Cplx> {
        let mut o = vec![Cplx::zero(); BUFFER_SIZE];
        o
        .iter_mut()
        .zip(self.buffer.iter().cycle().skip(self.offset))
        .for_each(|(out, inp)| *out = *inp);
        o
    }
    
    pub fn auto_rotate(&mut self) {
		self.rotate_left(self.rotate_size);
	}

    #[doc(hidden)]
    pub fn reset_offset(&mut self) {
        self.offset = 0;
    }
    
    pub fn set_to_writepoint(&mut self) {
        self.offset = self.write_point;
    }
    
	/*
    /// Ignores `write_point`. Rotates the buffer and writes to the start.
    /// Returns boolean indicating silence.
    pub fn read_from_input<T: cpal::Sample<Float = f64>>(&mut self, data: &[T]) -> bool {
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

    /// Returns boolean indicating silence,
    /// Doesn't compute max and average.
    pub fn read_from_input_quiet<T: cpal::Sample<Float = f32>>(&mut self, data: &[T]) -> bool {
        let input_size = data.len();
        self.input_size = input_size /2;
        let mut silence_index: u32 = 0;

        let mut di = self.write_point;
        data
        .chunks_exact(2)
        .enumerate()
        .for_each(|(_i, chunk)| {
            let smp = &mut unsafe{self.buffer.get_unchecked_mut(di)};
            write_sample(smp, chunk);

			let left  = fast::abs(smp.x);
			let right = fast::abs(smp.y);
			
            silence_index += ((left > SILENCE_LIMIT) || (right > SILENCE_LIMIT)) as u32;
	        di = self.index_add(di, 1);
        });
        
        self.write_point = di;


        if input_size >= BUFFER_SIZE {
            self.offset = self.write_point;
        } else {
            self.offset = self.index_sub(self.write_point, input_size);
        }

		self.rotate_size = self.input_size / 4 + 1;

        self.silent = silence_index < SILENCE_INDEX;

        self.silent
    }

    /// With `normalizer`
    pub fn read_from_input<T: cpal::Sample<Float = f32>>(&mut self, data: &[T]) -> bool {
        let input_size = data.len();
        self.input_size = input_size /2;

        let mut max_l = 0.0f64;
        let mut max_r = 0.0f64;
        
        //~ let mut sum_l = 0.0f64;
        //~ let mut sum_r = 0.0f64;

        let mut di = self.write_point;
        data
        .chunks_exact(2)
        .enumerate()
        .for_each(|(_i, chunk)| {
            let smp = &mut unsafe{self.buffer.get_unchecked_mut(di)};
            write_sample(smp, chunk);

			let left  = fast::abs(smp.x);
			let right = fast::abs(smp.y);
			
            max_l = max_l.max(left);
            max_r = max_r.max(right);
            
            //~ sum_l += left;
            //~ sum_r += right;

            // silence_index += ((left > SILENCE_LIMIT) || (right > SILENCE_LIMIT)) as u32;
	        di = self.index_add(di, 1);
        });
        
        let max = max_r.max(max_l);
        //~ let sum = sum_l + sum_r;

        self.write_point = di;

        //let new_offset = self.index_add(self.write_point, BUFFER_SIZE/2);
        //let new_offset = (new_offset+BUFFER_SIZE).max(self.write_point+BUFFER_SIZE)-BUFFER_SIZE;

        if input_size >= BUFFER_SIZE {
            self.offset = self.write_point;
        } else {
            self.offset = self.index_sub(self.write_point, input_size);
        }
		//~ self.average =
			//~ crate::math::interpolate::multiplicative_fall(
                //~ self.average,
                //~ sum / self.input_size as f64,
                //~ AMP_PERSIST_LIMIT,
                //~ REACT_SPEED
            //~ );

		self.silent = max < SILENCE_LIMIT;

        self.max =
            crate::math::interpolate::multiplicative_fall(
                self.max,
                max,
                AMP_PERSIST_LIMIT,
                REACT_SPEED
            );
        
        self.rotate_size = self.input_size / 4 + 1;

        self.silent
    }

    pub fn normalize(&mut self) {
		let scale_up_factor = self.normalize_factor_peak();

		// println!("{}, {}", self.max, scale_up_factor);

		if self.silent || scale_up_factor <= 1.0 {return}

		let mut write_point = self.index_sub(self.write_point, self.input_size);

        for _ in 0..self.input_size {
            let smp = unsafe {self.buffer.get_unchecked_mut(write_point)};
            *smp = smp.scale(scale_up_factor);
            write_point = self.index_add(write_point, 1);
        }

        // self.scale_up_factor = 0.0;
	}

    #[doc(hidden)]
    pub fn range(&self, index: Range<usize>) -> Vec<Cplx> {
        let range_ = index.start - index.end;
        let mut o = vec![Cplx::zero(); range_];
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
