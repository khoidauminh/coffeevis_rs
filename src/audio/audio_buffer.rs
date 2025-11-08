use crate::data::DEFAULT_ROTATE_SIZE;
use crate::math::Cplx;
use crate::math::interpolate::decay;

const SILENCE_LIMIT: f32 = 0.0001;
const AMP_PERSIST_LIMIT: f32 = 0.001;

const BUFFER_CAPACITY: usize = 1 << 16;
const BUFFER_MASK: usize = BUFFER_CAPACITY - 1;
const BUFFER_EVEN: usize = BUFFER_MASK - 1; // because all indices must be even.
const REACT_SPEED: f32 = 0.99;

pub struct AudioBuffer {
    data: [f32; BUFFER_CAPACITY],

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
            data: [0.0; _],

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

    pub fn read_from_input(&mut self, in_buffer: &[f32]) {
        let copysize = in_buffer.len();

        self.oldwriteend = self.writeend;
        self.readend = self.writeend;

        let (sleft, sright) = self.data.split_at_mut(self.writeend);

        let in_buffer_split = copysize.min(sright.len());
        sright[..in_buffer_split].copy_from_slice(&in_buffer[..in_buffer_split]);

        let in_buffer_left = copysize - in_buffer_split;
        sleft[..in_buffer_left].copy_from_slice(&in_buffer[in_buffer_split..]);

        self.writeend = self.writeend.wrapping_add(copysize) & BUFFER_EVEN;

        self.autorotatesize = copysize / (self.rotatessincewrite.next_power_of_two() * 2);

        self.rotatessincewrite = 0;

        self.lastinputsize = copysize;

        self.normalize();
    }

    fn normalize(&mut self) {
        let mut max = 0.0f32;
        for n in 0..self.lastinputsize {
            let i = n.wrapping_add(self.oldwriteend) & BUFFER_MASK;
            max = max.max(self.data[i]);
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
        let start = self.readend.wrapping_sub(len * 2) & BUFFER_EVEN;

        for i in 0..len {
            let si = (start + i * 2) & BUFFER_EVEN;
            out[i] = Cplx::from_slice(&self.data[si..si + 2]);
        }
    }

    pub fn input_size(&self) -> usize {
        self.lastinputsize
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn autoslide(&mut self) {
        self.readend = self.readend.wrapping_add(self.autorotatesize) & BUFFER_EVEN;
        self.rotatessincewrite += 1;
    }

    pub fn get(&self, i: usize) -> Cplx {
        let i = self.readend.wrapping_sub(i * 2) & BUFFER_EVEN;
        Cplx::from_slice(&self.data[i..i + 2])
    }
}
