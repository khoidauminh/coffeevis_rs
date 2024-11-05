use cpal::traits::{DeviceTrait, HostTrait};
use cpal::SampleFormat;

mod audio_buffer;
use crate::math::increment;
use audio_buffer::AudioBuffer;

use std::ops::*;
use std::sync::{
    atomic::{AtomicBool, AtomicU8, Ordering::Relaxed},
    Mutex, MutexGuard,
};

use crate::data::SAMPLE_RATE;

/// Global sample array
type GlobalSampleBuffer = Mutex<AudioBuffer>;

pub(crate) type SampleArr<'a> = MutexGuard<'a, AudioBuffer>;

static NO_SAMPLE: AtomicU8 = AtomicU8::new(0);

pub(crate) fn get_buf() -> SampleArr<'static> {
    static BUFFER: GlobalSampleBuffer = Mutex::new(AudioBuffer::new());
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
    let host = cpal::default_host();
    #[cfg(target_os = "windows")]
    {
        host = cpal::host_from_id(cpal::HostId::Asio).expect("failed to initialise ASIO host");
    }

    let device = host
        .default_input_device()
        .expect("no input device available");

    let err_fn = |err| eprintln!("an error occurred on the input audio stream: {}", err);

    let supported_configs_range = device
        .default_input_config()
        .expect("error while querying configs");

    let supported_config = supported_configs_range; //.next()

    let sample_format = supported_config.sample_format();

    let mut config: cpal::StreamConfig = supported_config.into();
    config.channels = 2;
    config.sample_rate = cpal::SampleRate(SAMPLE_RATE as u32);

    match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data, _: &_| read_samples::<f32>(data),
            err_fn,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data, _: &_| read_samples::<i16>(data),
            err_fn,
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            &config,
            move |data, _: &_| read_samples::<u16>(data),
            err_fn,
            None,
        ),
        _ => todo!(),
    }
    .unwrap()
}

pub fn read_samples<T: cpal::Sample<Float = f32>>(data: &[T]) {
    let mut b = get_buf();

    b.read_from_input(data);
    b.checked_normalize();

    set_no_sample(b.silent());
}

pub struct MovingAverage<T> {
    size: usize,
    index: usize,
    vec: Vec<T>,
    sum: T,
    denominator: f32,
    average: T,
}

impl<T> MovingAverage<T>
where
    T: Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + std::marker::Copy,
    f32: Mul<T, Output = T>,
{
    pub fn init(val: T, size: usize) -> Self {
        Self {
            size,
            index: 0,
            vec: vec![val; size],
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

use std::cmp::Ordering;

trait MMVal: PartialOrd + PartialEq + Copy + Clone + std::fmt::Display {}

impl MMVal for f32 {}

#[derive(Debug)]
struct NumPair<T: MMVal> {
    pub index: usize,
    pub value: T,
}

impl<T: MMVal> PartialEq for NumPair<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: MMVal> Eq for NumPair<T> {}

impl<T: MMVal> PartialOrd for NumPair<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.value < other.value {
            return Some(Ordering::Less);
        }

        if self.value > other.value {
            return Some(Ordering::Greater);
        }

        Some(Ordering::Equal)
    }
}

impl<T: MMVal> Ord for NumPair<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.value < other.value {
            return Ordering::Less;
        }

        if self.value > other.value {
            return Ordering::Greater;
        }

        Ordering::Equal
    }
}

struct MovingMaximum<T: MMVal> {
    pub heap: std::collections::BinaryHeap<NumPair<T>>,
    index: usize,
    size: usize,
}

impl<T: MMVal> MovingMaximum<T> {
    pub fn init(size: usize) -> Self {
        Self {
            heap: std::collections::BinaryHeap::with_capacity(size),
            index: 0,
            size,
        }
    }

    pub fn update(&mut self, new: T) -> T {
        self.heap.push(NumPair::<T> {
            index: self.index,
            value: new,
        });

        if unsafe {
            self.heap
                .peek()
                .unwrap_unchecked()
                .index
                .wrapping_add(self.size)
                <= self.index
        } {
            self.heap.pop();
        }

        self.index = self.index.wrapping_add(1);

        unsafe { self.heap.peek().unwrap_unchecked().value }
    }
}

pub fn limiter<T>(a: &mut [T], limit: f32, window: usize, gain: f32, flattener: fn(T) -> f32)
where
    T: Into<f32> + std::ops::Mul<f32, Output = T> + std::marker::Copy,
{
    let smoothing = window * 3 / 4;

    let mut mave = MovingAverage::init(limit, smoothing);
    let mut mmax = MovingMaximum::init(window);

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
