use crate::data::DEFAULT_ROTATE_SIZE;
use crate::math::Cplx;
use crate::math::interpolate::decay;

const SILENCE_LIMIT: f32 = 0.0001;
const AMP_PERSIST_LIMIT: f32 = 0.001;

const BUFFER_CAPACITY: usize = 1 << 16;
const BUFFER_MASK: usize = BUFFER_CAPACITY - 1;
const REACT_SPEED: f32 = 0.99;

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
        self.readend = self.writeend;

        let (sleft, sright) = self.data.split_at_mut(self.writeend);

        sright
            .iter_mut()
            .chain(sleft.iter_mut())
            .zip(in_buffer.chunks_exact(2))
            .for_each(|(i, chunk)| {
                *i = Cplx::new(chunk[0].to_float_sample(), chunk[1].to_float_sample());
            });

        self.writeend = self.writeend.wrapping_add(inputsize) & BUFFER_MASK;

        self.autorotatesize = inputsize / (self.rotatessincewrite.next_power_of_two() * 3 / 2);
        self.rotatessincewrite = 0;

        self.lastinputsize = inputsize;

        self.normalize();
    }

    fn normalize(&mut self) {
        let mut max = 0.0f32;
        for n in 0..self.lastinputsize {
            let i = n.wrapping_add(self.oldwriteend) & BUFFER_MASK;
            max = max.max(self.data[i].max());
        }

        self.max = decay(self.max, max, REACT_SPEED);

        if self.max < SILENCE_LIMIT {
            self.silent = self.silent.saturating_add(1);
            return;
        }

        self.silent = 0;

        let scale: f32 = 1.0 / self.max.max(AMP_PERSIST_LIMIT);

        for n in 0..self.lastinputsize {
            let i = n.wrapping_add(self.oldwriteend) & BUFFER_MASK;
            self.data[i] *= scale;
        }
    }

    pub fn read(&self, out: &mut [Cplx]) {
        let len = out.len().min(BUFFER_CAPACITY);
        let start = self.readend.wrapping_sub(len) & BUFFER_MASK;

        let (sleft, sright) = self.data.split_at(start);

        out.iter_mut()
            .zip(sright.iter().chain(sleft.iter()))
            .for_each(|(d, s)| *d = *s);
    }

    pub fn input_size(&self) -> usize {
        self.lastinputsize
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn autoslide(&mut self) {
        self.readend = self.readend.wrapping_add(self.autorotatesize) & BUFFER_MASK;
        self.rotatessincewrite += 1;
    }

    pub fn get(&self, i: usize) -> Cplx {
        self.data[self.readend.wrapping_sub(i) & BUFFER_MASK]
    }
}
