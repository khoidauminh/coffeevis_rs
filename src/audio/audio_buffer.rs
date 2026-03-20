use winit::window::Window;

use crate::math::Cplx;
use crate::math::interpolate::decay;

const SILENCE_LIMIT: f32 = 0.0001;
const AMP_PERSIST_LIMIT: f32 = 0.01;
const REACT_FACTOR: f32 = 0.98;

const BUFFER_CAPACITY: usize = 1 << 16;
const BUFFER_MASK: usize = BUFFER_CAPACITY - 1;

pub struct AudioBuffer {
    data: [Cplx; BUFFER_CAPACITY],

    writeend: usize,
    readend: usize,

    autorotatesize: usize,
    rotatessincewrite: usize,

    lastinputsize: usize,

    silent: u8,

    max: f32,

    normalize: bool,

    window: Option<&'static dyn Window>,
}

impl AudioBuffer {
    pub const fn new() -> Self {
        Self {
            data: [Cplx::zero(); _],

            writeend: 0,
            readend: 0,

            autorotatesize: 0,
            rotatessincewrite: 0,

            lastinputsize: 0,

            silent: 0,

            max: 0.0,

            normalize: true,

            window: None,
        }
    }

    pub fn init_realtime_wakeup(&mut self, w: &'static dyn Window) {
        if self.window.is_some() {
            panic!("Already initialized!");
        }

        self.window = Some(w);
    }

    pub fn deinit_realtime_wakeup(&mut self) {
        self.window = None;
    }

    pub fn silent(&self) -> u8 {
        self.silent
    }

    pub fn read_from_input(&mut self, in_buffer: &[f32]) {
        let copysize = in_buffer.len() / 2;

        let mut src_iter = in_buffer.chunks_exact(2);
        let (dst_l, dst_r) = self.data.split_at_mut(self.writeend & BUFFER_MASK);

        dst_r
            .iter_mut()
            .chain(dst_l.iter_mut())
            .zip(&mut src_iter)
            .for_each(|(d, s)| *d = Cplx::from_slice(s));

        self.writeend = self.writeend + copysize;

        self.autorotatesize = copysize / self.rotatessincewrite.max(3);
        self.lastinputsize = copysize;
        self.rotatessincewrite = 0;

        self.post_process();
    }

    pub fn set_normalize(&mut self, b: bool) {
        self.normalize = b;
    }

    fn post_process(&mut self) {
        let oldwriteend = (self.writeend - self.lastinputsize) & BUFFER_MASK;
        let (left, right) = self.data.split_at_mut(oldwriteend);
        let max = right
            .iter()
            .chain(left.iter())
            .take(self.lastinputsize)
            .fold(0.0f32, |a, c| a.max(c.max()));

        self.max = decay(self.max, max, REACT_FACTOR);

        if self.max < SILENCE_LIMIT {
            self.silent = self.silent.saturating_add(1);
            return;
        }

        self.silent = 0;

        if let Some(w) = self.window.as_ref() {
            w.request_redraw();
        }

        if self.normalize {
            let scale: f32 = 1.0 / self.max.max(AMP_PERSIST_LIMIT);

            right
                .iter_mut()
                .chain(left.iter_mut())
                .take(self.lastinputsize)
                .for_each(|s| *s *= scale);
        }
    }

    pub fn read(&self, out: &mut [Cplx]) {
        let start = (self.readend - out.len()) & BUFFER_MASK;
        let (sleft, sright) = self.data.split_at(start);

        out.iter_mut()
            .zip(sright.iter().chain(sleft.iter()))
            .for_each(|(o, i)| *o = *i);
    }

    pub fn input_size(&self) -> usize {
        self.lastinputsize
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn autoslide(&mut self) {
        let max_delay = self.writeend.saturating_sub(self.lastinputsize * 3);
        let run_delay = self.writeend.saturating_sub(self.lastinputsize * 2);
        let min_delay = self.writeend.saturating_sub(self.lastinputsize / 2);

        if self.readend < max_delay {
            self.readend = max_delay;
        } else {
            if self.readend < min_delay {
                self.readend += self.autorotatesize;

                if self.readend < run_delay {
                    self.readend += self.autorotatesize;
                }
            }
        }

        self.rotatessincewrite += 1;
    }

    pub fn get(&self, i: usize) -> Cplx {
        let i = self.readend.wrapping_sub(i) & BUFFER_MASK;
        self.data[i]
    }
}
