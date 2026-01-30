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
    oldwriteend: usize,
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
            oldwriteend: 0,
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

        self.oldwriteend = self.writeend;
        self.readend = self.writeend;

        let (src_l, src_r) = in_buffer.split_at(
            in_buffer
                .len()
                .min(2 * BUFFER_CAPACITY.saturating_sub(self.writeend)),
        );

        let (dst_l, dst_r) = self.data.split_at_mut(self.writeend);

        dst_r
            .iter_mut()
            .zip(src_l.chunks_exact(2))
            .for_each(|(d, s)| *d = Cplx::from_slice(s));

        dst_l
            .iter_mut()
            .zip(src_r.chunks_exact(2))
            .for_each(|(d, s)| *d = Cplx::from_slice(s));

        self.writeend = self.writeend.wrapping_add(copysize) & BUFFER_MASK;

        self.lastinputsize = copysize;

        self.post_process();
    }

    pub fn set_normalize(&mut self, b: bool) {
        self.normalize = b;
    }

    /// The scaling factor formula is as follows:
    /// {   1 / x                   x >= L
    /// {   A / x + (1 - A) / L     x < L
    fn get_scaling_factor(max: f32) -> f32 {
        let max = max.max(AMP_PERSIST_LIMIT);

        const L: f32 = 0.6;
        const A: f32 = 0.33;

        if max >= 0.5 {
            return max.recip();
        }

        A / max + (1.0 - A) / L
    }

    fn post_process(&mut self) {
        let mut max = self.data[self.oldwriteend].max();
        for n in 1..self.lastinputsize {
            let i = n.wrapping_add(self.oldwriteend) & BUFFER_MASK;
            max = max.max(self.data[i].max());
        }

        self.max = decay(self.max, max, REACT_FACTOR);

        if self.max < SILENCE_LIMIT {
            self.silent = self.silent.saturating_add(1);
            return;
        }

        if let Some(w) = self.window.as_ref() {
            w.request_redraw();
        }

        self.autorotatesize = self.lastinputsize / self.rotatessincewrite.max(3);
        self.rotatessincewrite = 0;
        self.silent = 0;

        if self.normalize {
            let scale: f32 = Self::get_scaling_factor(self.max);

            for n in 0..self.lastinputsize {
                let i = n.wrapping_add(self.oldwriteend) & BUFFER_MASK;
                self.data[i] *= scale;
            }
        }
    }

    pub fn read(&self, out: &mut [Cplx]) {
        let len = out.len();

        let start = self.readend.wrapping_sub(len) & BUFFER_MASK;
        let (sleft, sright) = self.data.split_at(start);

        let outsplit = len.min(sright.len());
        let outremaining = len - outsplit;

        out[..outsplit].copy_from_slice(&sright[..outsplit]);
        out[outsplit..].copy_from_slice(&sleft[..outremaining]);
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
        let i = self.readend.wrapping_sub(i) & BUFFER_MASK;
        self.data[i]
    }
}
