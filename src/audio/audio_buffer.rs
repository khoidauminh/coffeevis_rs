use std::sync::mpsc::{self, Receiver, SyncSender};

use crate::math::Cplx;
use crate::math::interpolate::decay;

const SILENCE_LIMIT: f32 = 0.0001;
const AMP_PERSIST_LIMIT: f32 = 0.005;
const REACT_FACTOR: f32 = 0.98;

const BUFFER_CAPACITY: usize = 1 << 16;
const BUFFER_MASK: usize = BUFFER_CAPACITY - 1;
const AUDIO_NOTIFIER_PADDING: u16 = 10;

pub type AudioNotifier = SyncSender<u16>;

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

    notifier: Option<AudioNotifier>,
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

            notifier: None,
        }
    }

    pub fn init_notifier(&mut self) -> Receiver<u16> {
        if self.notifier.is_some() {
            panic!("Only one pair of sender/receiver is allowed!");
        }

        let (s, r) = mpsc::sync_channel(0);

        self.notifier = Some(s);

        r
    }

    pub fn close_notifier(&mut self) {
        self.notifier = None;
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

    #[allow(unreachable_code)]
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

        if let Some(s) = self.notifier.as_ref() {
            let _ =
                s.try_send((self.rotatessincewrite as u16).saturating_add(AUDIO_NOTIFIER_PADDING));
        }

        self.autorotatesize = self.lastinputsize / self.rotatessincewrite.max(3);

        self.rotatessincewrite = 0;

        self.silent = 0;

        let scale: f32 = 1.0 / self.max.max(AMP_PERSIST_LIMIT);

        for n in 0..self.lastinputsize {
            let i = n.wrapping_add(self.oldwriteend) & BUFFER_MASK;
            self.data[i] *= scale;
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
