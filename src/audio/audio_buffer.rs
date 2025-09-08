use crate::data::{DEFAULT_ROTATE_SIZE, foreign::ForeignAudioCommunicator};
use crate::math::Cplx;

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
    readend: usize,

    autorotatesize: usize,
    rotatessincewrite: usize,

    lastinputsize: usize,

    silent: u8,

    max: f32
}

impl AudioBuffer {
    pub const fn new() -> Self {
        Self {
            data: [Cplx::zero(); _],
            
            writeend: 0,
            readend: 0,

            autorotatesize: DEFAULT_ROTATE_SIZE,
            rotatessincewrite: 0,

            lastinputsize: 0,

            silent: 0,

            max: 0.0,
        }
    }

    pub fn init_audio_communicator(&mut self) {}

    pub fn silent(&self) -> u8 {
        self.silent
    }

    fn write_sample<T: cpal::Sample<Float = f32>>(smp: &mut Cplx, smp_in: &[T]) {
        smp.x = smp_in[0].to_float_sample();
        smp.y = smp_in[1].to_float_sample();
    }

    pub fn read_from_input<T: cpal::Sample<Float = f32>>(&mut self, in_buffer: &[T]) {
        let inputlen = in_buffer.len();

        let inputsize = inputlen / 2;

        let mut writeend = self.writeend;
        
        in_buffer.chunks_exact(2).for_each(|chunk| {
            Self::write_sample(&mut self.data[writeend], chunk);
            writeend += 1;
            writeend &= BUFFER_MASK;
        });

        self.writeend = writeend;
        self.readend = writeend.wrapping_sub(inputsize) & BUFFER_MASK;

        self.autorotatesize = inputsize / self.rotatessincewrite.next_power_of_two();
        self.rotatessincewrite = 0;

        self.lastinputsize = inputsize;
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
        self.readend += self.autorotatesize;
        self.readend &= BUFFER_MASK;
        self.rotatessincewrite += 1;
    }

    pub fn checked_normalize(&mut self) {}

    pub fn get(&self, i: usize) -> Cplx {
        self.data[(self.readend + i) & BUFFER_CAPACITY]
    }
}

