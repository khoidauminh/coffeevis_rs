use cpal::SampleRate;
use cpal::traits::{DeviceTrait, HostTrait};

pub mod audio_buffer;
use crate::math::increment;
pub(crate) use audio_buffer::AudioBuffer;

use std::ops::*;
use std::sync::{
    Mutex, MutexGuard,
    atomic::{AtomicBool, AtomicU8, Ordering::Relaxed},
};

static NO_SAMPLE: AtomicU8 = AtomicU8::new(0);

pub(crate) fn get_buf<'a>() -> MutexGuard<'a, AudioBuffer> {
    static BUFFER: Mutex<AudioBuffer> = Mutex::new(AudioBuffer::new());
    BUFFER.lock().unwrap()
}

pub(crate) fn get_no_sample() -> u8 {
    NO_SAMPLE.load(Relaxed)
}

pub fn set_no_sample(ns: u8) {
    NO_SAMPLE.store(ns, Relaxed);
}

pub static NORMALIZE: AtomicBool = AtomicBool::new(true);

pub fn set_normalizer(b: bool) {
    NORMALIZE.store(b, Relaxed);
}

pub fn get_source() -> cpal::Stream {
    let device = cpal::default_host()
        .default_input_device()
        .expect("no input device available");

    let mut config: cpal::StreamConfig = device
        .default_input_config()
        .expect("error while querying configs")
        .config();

    config.sample_rate = SampleRate(crate::data::SAMPLE_RATE as u32);

    device
        .build_input_stream(
            &config,
            |data: &[f32], _| {
                let mut b = get_buf();
                b.read_from_input(data);
                if NORMALIZE.load(Relaxed) {
                    b.checked_normalize();
                }
                set_no_sample(b.silent());
            },
            |err| eprintln!("an error occurred on the input audio stream: {}", err),
            None,
        )
        .unwrap()
}

pub struct MovingAverage<T, const N: usize> {
    size: usize,
    index: usize,
    vec: [T; N],
    sum: T,
    denominator: f32,
    average: T,
}

impl<T, const N: usize> MovingAverage<T, N>
where
    T: Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + Copy,
    f32: Mul<T, Output = T>,
{
    pub fn init(val: T, size: usize) -> Self {
        Self {
            size,
            index: 0,
            vec: [val; N],
            denominator: (size as f32).recip(),
            sum: size as f32 * val,
            average: val,
        }
    }

    fn pop(&mut self, val: T) -> T {
        let out = self.vec[self.index];

        self.vec[self.index] = val;

        self.index = increment(self.index, self.size);

        out
    }

    pub fn update(&mut self, val: T) -> T {
        let old = self.pop(val);

        self.sum = self.sum - old + val;

        self.average = self.denominator * self.sum;

        self.average
    }
}

#[derive(Default, Clone, Copy)]
struct NumPair<T> {
    pub index: usize,
    pub value: T,
}

struct MovingMaximum<T, const N: usize> {
    pub heap: [NumPair<T>; N],
    len: usize,
    index: usize,
    size: usize,
}

impl<T, const N: usize> MovingMaximum<T, N>
where
    T: Default + Copy + PartialOrd,
{
    pub fn init(size: usize) -> Self {
        Self {
            heap: [NumPair::default(); N],
            len: 0,
            index: 0,
            size,
        }
    }

    pub fn push(&mut self, new: NumPair<T>) {
        let mut i = self.len;
        self.len += 1;
        while i > 0 {
            let p = (i - 1) / 2;
            if self.heap[p].value >= new.value {
                break;
            }

            self.heap[i] = self.heap[p];

            i = p;
        }

        self.heap[i] = new;
    }

    pub fn peek(&self) -> &NumPair<T> {
        &self.heap[0]
    }

    pub fn pop(&mut self, mut p: usize) -> NumPair<T> {
        self.len -= 1;
        let out = self.heap[0];
        self.heap[0] = self.heap[self.len];

        let bound = self.len - 2;
        let mut i = 2 * p + 1;
        while i < bound {
            i += (self.heap[i].value <= self.heap[i + 1].value) as usize;

            if self.heap[p].value >= self.heap[i].value {
                return out;
            }

            self.heap[p] = self.heap[i];
            p = i;
            i = i * 2 + 1;
        }

        if i == self.len - 1 && self.heap[p].value < self.heap[i].value {
            self.heap[p] = self.heap[i];
        }

        out
    }

    pub fn update(&mut self, new: T) -> T {
        self.push(NumPair {
            index: self.index,
            value: new,
        });

        let max_age = self.peek().index.wrapping_add(self.size - 1);

        if max_age <= self.index {
            self.pop(0);
        }

        self.index += 1;

        self.peek().value
    }
}

pub fn limiter<T, const N: usize>(
    a: &mut [T],
    limit: f32,
    window: usize,
    gain: f32,
    flattener: fn(T) -> f32,
) where
    T: Into<f32> + Mul<f32, Output = T> + Copy,
{
    assert!(a.len() <= N);

    let smoothing = window * 3 / 4;

    let mut mave = MovingAverage::<_, N>::init(limit, smoothing);
    let mut mmax = MovingMaximum::<_, N>::init(window);

    for i in 0..a.len() + smoothing {
        let smp = if let Some(ele) = a.get(i) {
            flattener(*ele).abs().max(limit)
        } else {
            limit
        };

        let mult = mave.update(mmax.update(smp));

        let j = i.wrapping_sub(smoothing);

        if let Some(ele) = a.get_mut(j) {
            let mult = gain / mult;
            *ele = *ele * mult;
        }
    }
}
