use cpal::traits::{DeviceTrait, HostTrait};
use cpal::SampleFormat;

mod audio_buffer;
use audio_buffer::AudioBuffer;

use std::ops::*;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Mutex, MutexGuard, RwLock,
};

use crate::data::SAMPLE_RATE;

/// Global sample array
type GSA = Mutex<AudioBuffer>;

pub(crate) struct SampleArr<'a>(MutexGuard<'a, AudioBuffer>);

impl<'a> std::ops::Deref for SampleArr<'a> {
    type Target = MutexGuard<'a, AudioBuffer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> std::ops::DerefMut for SampleArr<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

static BUFFER: GSA = Mutex::new(AudioBuffer::new());

static NO_SAMPLE: AtomicU8 = AtomicU8::new(0);

pub static NORMALIZE: RwLock<bool> = RwLock::new(true);

pub fn set_normalizer(b: bool) {
    *NORMALIZE.write().unwrap() = b;
}

pub fn get_normalizer() -> bool {
    *NORMALIZE.read().unwrap()
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
    let _l = data.len();

    let _i = 0usize;
    let _s = 0usize;

    let mut b = BUFFER.lock().unwrap();

    b.read_from_input(data);
    b.checked_normalize();
    let ns = b.silent();
    NO_SAMPLE.store(ns, Ordering::Relaxed);
}

pub fn get_buf() -> SampleArr<'static> {
    SampleArr(BUFFER.lock().unwrap())
}

pub fn get_no_sample() -> u8 {
    NO_SAMPLE.load(Ordering::Relaxed)
}

pub fn is_silent() -> bool {
    get_no_sample() > 0
}

use crate::misc::stackvec::StackVec;

const MAX_STACK_VEC_SIZE: usize = 15;

pub struct MovingAverage<T, const N: usize> {
    size: usize,
    index: usize,
    vec: StackVec<T, N>,
    sum: T,
    denominator: f64,
    average: T,
}

impl<T, const N: usize> MovingAverage<T, N>
where
    T: Add<Output = T> + Sub<Output = T> + Mul<f64, Output = T> + std::marker::Copy,
    f64: Mul<T, Output = T>,
{
    pub fn init(val: T, size: usize) -> Self {
        assert!(size < N);

        Self {
            size,
            index: 0,
            vec: StackVec::init(val, size),
            denominator: (size as f64).recip(),
            sum: size as f64 * val,
            average: val,
        }
    }

    fn pop(&mut self, val: T) -> T {
        let out = self.vec[self.index];

        self.vec[self.index] = val;

        self.index = crate::math::increment(self.index, self.size);

        out
    }

    pub fn update(&mut self, val: T) -> T {
        let old = self.pop(val);

        self.sum = self.sum - old + val;

        self.average = self.denominator * self.sum;

        self.average
    }

    pub fn current(&self) -> T {
        self.average
    }

    pub fn force_compute_average(&self) -> T {
        let mut sum = self.vec[0];
        for i in 1..self.size {
            sum = sum + self.vec[i]
        }
        return sum * self.denominator;
    }
}

struct MovingMaximum<const N: usize> {
    buffer: StackVec<f64, N>,
    size: usize,
    index: usize,
    max: f64,
}

impl<const N: usize> MovingMaximum<N> {
    pub fn init(val: f64, size: usize) -> Self {
        assert!(size < N);

        Self {
            buffer: StackVec::init(val, size),
            size,
            index: 0,
            max: val,
        }
    }

    fn pop(&mut self, new: f64) -> f64 {
        let old = self.buffer[self.index];
        self.buffer[self.index] = new;

        self.index = crate::math::increment(self.index, self.size);

        old
    }

    pub fn update(&mut self, new: f64) -> f64 {
        let old = self.pop(new);

        if new > self.max {
            self.max = new;
            return self.max;
        }

        if old == self.max {
            self.max = self.buffer.slice().iter().fold(new, |acc, x| acc.max(*x));
        }

        return self.max;
    }

    pub fn current(&self) -> f64 {
        self.max
    }
}

pub fn limiter<T>(a: &mut [T], limit: f64, window: usize, gain: f64, flattener: fn(T) -> f64)
where
    T: Into<f64> + std::ops::Mul<f64, Output = T> + std::marker::Copy,
{
    let smoothing = window * 3 / 4;

    let mut mave = MovingAverage::<f64, 32>::init(limit, smoothing);
    let mut mmax = MovingMaximum::<32>::init(limit, window);

    for i in 0..a.len() + smoothing {
        let smp = if let Some(ele) = a.get(i) {
            flattener(*ele).abs().max(limit)
        } else {
            limit
        };

        mave.update(mmax.update(smp));

        let j = i.wrapping_sub(smoothing);

        if let Some(ele) = a.get_mut(j) {
            let mult = gain / mave.current();

            *ele = *ele * mult;
        }
    }
}
