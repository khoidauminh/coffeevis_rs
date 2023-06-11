use cpal;
use crate::math::Cplx;
use crate::data::SAMPLE_SIZE;
use std::iter;

const BUFFER_SIZE: usize = 1 << 12;
const SIZE_MASK: usize = BUFFER_SIZE - 1;

type BufferArray = [Cplx<f32>; BUFFER_SIZE];

pub struct AudioBuffer {
    buffer: BufferArray,
//    to_be_exposed: BufferArray, // cache
    offset: usize,
    write_point: usize,
    update: bool,
}

impl std::ops::Index<usize> for AudioBuffer {
    type Output = Cplx<f32>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[(index + self.offset)&SIZE_MASK]
    }
}

use std::ops::Range;
/*
impl std::ops::Index<Range<usize>> for AudioBuffer {
    type Output = Vec<Cplx<f32>>;
  
}
*/
impl AudioBuffer {
    pub const fn new() -> Self {
        Self {
            buffer: [Cplx::<f32>::zero(); BUFFER_SIZE],
//            to_be_exposed: [Cplx::<f32>::zero(); BUFFER_SIZE],
            offset: 0,
            write_point: 0,
            update: false
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn rotate_left(&mut self, n: usize) {
        self.offset += n;
        self.offset &= SIZE_MASK;
        self.set_update();
    }

    pub fn rotate_right(&mut self, n: usize) {
        self.offset = (self.offset + BUFFER_SIZE - n) & SIZE_MASK;
    }
    
    /*
    pub fn get(&mut self) -> BufferArray {
        self.buffer
        .iter()
        .cycle()
        .skip(self.start)
        .take(BUFFER_SIZE)
        .collect::<Vec<Cplx<f32>>>()

        self.perform_update();
        
        self.to_be_exposed
    }
    
    fn perform_update(&mut self) {
        if self.update {
            self.to_be_exposed = self.buffer;
            self.to_be_exposed.rotate_left(self.start);
        }
        self.unset_update();        
    }

    pub fn get_ref<'a>(&'a mut self) -> &'a BufferArray {
        self.perform_update();
        &self.to_be_exposed
    }
*/

    pub fn to_vec(&mut self) -> Vec<Cplx<f32>> {
        let mut o = vec![Cplx::<f32>::zero(); BUFFER_SIZE];
        o
        .iter_mut()
        .zip(self.buffer.iter().cycle().skip(self.start))
        .for_each(|(out, inp)| *out = *inp);
        o
    }

    fn set_update(&mut self) {
        self.update = true;
    }

    fn unset_update(&mut self) {
        self.update = false;
    }

//    pub fn iter(&self) -> impl Iterator<Item = Cplx<f32>> {
//        let mut it = std::iter::repeat(self.buffer.iter());
//        let _ = it.nth(if self.start == 0 {0} else {self.start-1});
//        it.take(BUFFER_SIZE)
//        
//    }
    
    /*
    pub fn write_to_global_buffer(&self) {
        let global_buffer_result = super::BUFFER.write();
        if let Ok(mut gbuffer) = global_buffer_result {
            let size = gbuffer.len().min(self.buffer.len());
            gbuffer[..size].copy_from_slice(&self.buffer[..size]); 
        }
    }*/

    pub fn reset_offset(&mut self) {
        self.offset = 0;
    }

    // returns boolean indicating silence.
    pub fn read_from_input<T: cpal::Sample>(&mut self, data: &[T]) -> bool {
        let input_size = data.len();
        let mut silence = true;
        self.buffer.rotate_right(input_size);
        data
        .chunks_exact(2)
        .zip(self.buffer.iter_mut())
        .for_each(|(inp, smp)| {
            smp.x = inp[0].to_f32();
            smp.y = inp[1].to_f32();
            silence = silence && (smp.x < 1e-2) && (smp.y < 1e-2);
        });
        self.reset_offset();
        self.set_update();
        silence
    }

    pub fn read_from_input_inplace<T: cpal::Sample>(&mut self, data: &[T]) -> bool {
        let input_size = data.len();
        let mut silence = true;
        self.offset = (self.offset + BUFFFER_SIZE - input_size) & SIZE_MASK;
        data
        .chunks_exact(2)
        .enumerate()
        .for_each(|(i, chunk)| {
            let mut smp = &mut self.buffer[i];
            smp.x = chunk[0].to_f32();
            smp.y = chunk[1].to_f32();
            silence = silence && (smp.x < 1e-2) && (smp.y < 1e-2);
        });
        self.set_update();
        silence
    }

    pub fn range(&self, index: Range<usize>) -> Vec<Cplx<f32>> {
        let range_ = index.start - index.end;
        let mut o = vec![Cplx::<f32>::zero(); range_];
        self.buffer
        .iter()
        .cycle()
        .skip(if index.start == 0 {0} else {index.start - 1})
        .take(range_)
        .zip(o.iter_mut())
        .for_each(|(inp, out)| *out = *inp);

        o
    }    
}
