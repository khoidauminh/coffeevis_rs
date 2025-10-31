use crate::data::DEFAULT_ROTATE_SIZE;
use crate::math::Cplx;
use crate::math::interpolate::decay;

const SILENCE_LIMIT: f32 = 0.001;
const AMP_PERSIST_LIMIT: f32 = 0.05;
const AMP_TRIGGER_THRESHOLD: f32 = 0.85;
const SILENCE_CHECK_SIZE: u8 = 24;

const BUFFER_CAPACITY: usize = 1 << 20;
const BUFFER_MASK: usize = BUFFER_CAPACITY -1;
const REACT_SPEED: f32 = 0.025;

pub struct AudioBuffer {
    data: [Cplx; BUFFER_CAPACITY],

    writeend: usize,
    oldwriteend: usize,
    readend: usize,

    autorotatesize: usize,
    rotatessincewrite: usize,

    lastinputsize: usize,

    silent: u8,

    max: f32,
}

impl AudioBuffer {
    pub const fn new() -> Self {
        Self {
            data: [Cplx::zero(); _],
            
            writeend: 0,
            oldwriteend: 0,
            readend: 0,

            autorotatesize: DEFAULT_ROTATE_SIZE,
            rotatessincewrite: 0,

            lastinputsize: 0,

            silent: 0,

            max: 0.0,
        }
    }

    pub fn silent(&self) -> u8 {
        self.silent
    }

    pub fn read_from_input<T: cpal::Sample<Float = f32>>(&mut self, in_buffer: &[T]) {
        let inputsize = in_buffer.len() / 2;

        self.oldwriteend = self.writeend;
        let mut writeend = self.writeend;
        
        in_buffer.chunks_exact(2).for_each(|chunk| {
            self.data[writeend] = Cplx::new(chunk[0].to_float_sample(), chunk[1].to_float_sample());
            writeend = (writeend + 1) & BUFFER_MASK;
        });

        self.writeend = writeend;
        self.readend = writeend.wrapping_sub(inputsize) & BUFFER_MASK;

        self.autorotatesize = inputsize / (self.rotatessincewrite.next_power_of_two()*3/2);
        self.rotatessincewrite = 0;

        self.lastinputsize = inputsize;

        self.normalize();
    }

    fn normalize(&mut self) {
        let mut max = 0.0f32;
        for n in 0..self.lastinputsize {
            let i = (n + self.oldwriteend) & BUFFER_MASK;
            max = max.max(self.data[i].max());
        }

        self.max = decay(self.max, max, 0.99);

        if self.max < 0.0001 || self.max >= 1.0 {
            return
        }
        
        let scale: f32 = 1.0 / self.max.max(0.001);

        for n in 0..self.lastinputsize {
            let i = (n + self.oldwriteend) & BUFFER_MASK;
            self.data[i] *= scale;
        }
    }

    pub fn read(&self, out: &mut [Cplx]) {
        let len = out.len().min(BUFFER_CAPACITY);

        let mut start = self.readend.wrapping_sub(len) & BUFFER_MASK;

        out.iter_mut().for_each(|smp| {
            *smp = self.data[start];

            start += 1;
            start &= BUFFER_MASK;
        });
    }

    pub fn input_size(&self) -> usize {
        self.lastinputsize
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn autoslide(&mut self) {
        self.readend = (self.readend + self.autorotatesize) & BUFFER_MASK;
        self.rotatessincewrite += 1;
    }

    pub fn get(&self, i: usize) -> Cplx {
        self.data[(self.readend - i) & BUFFER_MASK]
    }
}

