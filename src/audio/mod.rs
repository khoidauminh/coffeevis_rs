use cpal::Device;
use cpal::traits::{DeviceTrait, HostTrait};

pub mod audio_buffer;
pub(crate) use audio_buffer::AudioBuffer;

use std::ops::*;
use std::sync::{Mutex, MutexGuard};

pub(crate) fn get_buf<'a>() -> MutexGuard<'a, AudioBuffer> {
    static BUFFER: Mutex<AudioBuffer> = Mutex::new(AudioBuffer::new());
    BUFFER.lock().unwrap()
}

#[cfg(target_os = "linux")]
pub fn get_device_linux() -> Device {
    cpal::default_host()
        .default_input_device()
        .expect("No loopback device available")
}

#[cfg(target_os = "windows")]
pub fn get_device_windows() -> Device {
    let default_device = cpal::default_host().default_input_device();

    let input_devices = cpal::default_host()
        .input_devices()
        .expect("Failed to probe all input devices.")
        .collect::<Vec<_>>();

    for d in &input_devices {
        println!("I see {}", d.name().unwrap_or("<unknown>".to_string()));
    }

    let query = input_devices.iter().find(|d| {
        d.name()
            .map(|name| name.contains("Stereo Mix"))
            .unwrap_or(false)
    });

    if let Some(q) = query {
        crate::data::log::info!("Found Stereo Mix.");
        return q.clone();
    }

    let d = default_device.expect("Failed to get default device.");
    crate::data::log::alert!(
        "No Stereo Mix found. Going with default input device: {}",
        d.name().unwrap_or("<Unknown>".to_string())
    );

    d
}

pub fn get_source() -> cpal::Stream {
    #[cfg(target_os = "linux")]
    let device = get_device_linux();

    #[cfg(target_os = "windows")]
    let device = get_device_windows();

    let config: cpal::StreamConfig = device
        .default_input_config()
        .expect("error while querying configs")
        .config();

    crate::data::log::info!("Using {}", device.name().unwrap_or("<unkown>".to_owned()));

    device
        .build_input_stream(
            &config,
            |data: &[f32], _| {
                let mut b = get_buf();
                b.read_from_input(data);
            },
            |err| eprintln!("an error occurred on the input audio stream: {}", err),
            None,
        )
        .unwrap()
}

pub struct MovingAverage<T, const N: usize> {
    index: usize,
    data: [T; N],
    sum: T,
    denominator: f32,
}

impl<T, const N: usize> MovingAverage<T, N>
where
    T: Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + Copy,
    f32: Mul<T, Output = T>,
{
    pub fn init(val: T) -> Self {
        Self {
            index: 0,
            data: [val; N],
            denominator: (N as f32).recip(),
            sum: N as f32 * val,
        }
    }

    pub fn update(&mut self, val: T) -> T {
        self.sum = self.sum - self.data[self.index] + val;

        self.data[self.index] = val;

        self.index += 1;
        if self.index == N {
            self.index = 0;
        }

        self.denominator * self.sum
    }
}

#[derive(Default, Clone, Copy)]
struct NumPair<T> {
    pub index: usize,
    pub value: T,
}

struct MovingMaximum<T, const N: usize> {
    data: [NumPair<T>; N],

    head: usize,
    tail: usize,

    len: usize,
    index: usize,
}

impl<T: Default + Copy + PartialOrd, const N: usize> MovingMaximum<T, N> {
    pub fn init() -> Self {
        Self {
            data: [NumPair::default(); N],
            head: 0,
            tail: N - 1,
            len: 0,
            index: 0,
        }
    }

    fn enqueue_tail(&mut self, value: T) {
        self.len += 1;

        self.tail += 1;
        if self.tail == N {
            self.tail = 0;
        }

        self.data[self.tail] = NumPair {
            index: self.index,
            value,
        };

        self.index += 1;
    }

    fn dequeue_head(&mut self) {
        self.len -= 1;

        self.head += 1;
        if self.head == N {
            self.head = 0;
        }
    }

    fn dequeue_tail(&mut self) {
        self.len -= 1;

        if self.tail == 0 {
            self.tail = N;
        }

        self.tail -= 1;
    }

    pub fn update(&mut self, new: T) -> T {
        if self.data[self.head].index + N <= self.index && self.len > 0 {
            self.dequeue_head();
        }

        while self.data[self.tail].value < new && self.len > 0 {
            self.dequeue_tail();
        }

        self.enqueue_tail(new);

        self.data[self.head].value
    }
}

pub fn limiter<T>(a: &mut [T], lo: f32, hi: f32, flattener: fn(T) -> f32)
where
    T: Into<f32> + Mul<f32, Output = T> + Copy,
{
    const SMOOTHING: usize = 10;
    const DELAY: usize = SMOOTHING - 1;

    let mut mave = MovingAverage::<_, SMOOTHING>::init(0.0);
    let mut mmax = MovingMaximum::<_, SMOOTHING>::init();

    for i in 0..a.len() + SMOOTHING {
        let smp = if let Some(ele) = a.get(i) {
            flattener(*ele).abs()
        } else {
            0.0
        };

        let smp2 = mave.update(mmax.update(smp));

        let scale = if smp2 > hi {
            hi / smp2
        } else if smp2 < lo {
            lo / smp2.max(0.001)
        } else {
            1.0
        };

        let j = i.wrapping_sub(DELAY);

        if let Some(ele) = a.get_mut(j) {
            *ele = *ele * scale;
        }
    }
}
