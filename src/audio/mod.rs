use cpal::Device;
use cpal::traits::{DeviceTrait, HostTrait};

pub mod audio_buffer;
pub(crate) use audio_buffer::AudioBuffer;

use std::cell::Cell;
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
        .expect("No input device available")
}

#[cfg(target_os = "windows")]
pub fn get_device_windows() -> Device {
    let default_device = cpal::default_host().default_input_device();

    let input_devices = cpal::default_host()
        .input_devices()
        .expect("Failed to probe all input devices.")
        .collect::<Vec<_>>();

    for d in &input_devices {
        println!(
            "I see {}",
            d.description()
                .as_ref()
                .map(|v| v.name())
                .unwrap_or("<unknown>")
        );
    }

    let query = input_devices.iter().find(|d| {
        d.description()
            .as_ref()
            .map(|name| name.name().contains("Stereo Mix"))
            .unwrap_or(false)
    });

    if let Some(q) = query {
        crate::data::log::info!("Found Stereo Mix.");
        return q.clone();
    }

    let d = default_device.expect("Failed to get default device.");
    crate::data::log::alert!(
        "No Stereo Mix found. Going with default input device: {}",
        d.description()
            .as_ref()
            .map(|v| v.name())
            .unwrap_or("<unknown>")
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

    crate::data::log::info!(
        "Using {}",
        device
            .description()
            .as_ref()
            .map(|v| v.name())
            .unwrap_or("<Unknown device>")
    );

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
    T: Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + Copy + Default,
    f32: Mul<T, Output = T>,
{
    pub fn init() -> Self {
        Self {
            index: 0,
            data: [T::default(); N],
            denominator: (N as f32).recip(),
            sum: T::default(),
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

pub fn limiter<T>(a: &mut [T], lo: f32, hi: f32, flat: fn(T) -> f32)
where
    T: Mul<f32, Output = T> + Copy,
{
    const DEFAULT_SMOOTHING: usize = 10;
    const DEFAULT_DELAY: usize = DEFAULT_SMOOTHING - 1;

    let l = a.len();
    let delay = l.min(DEFAULT_DELAY);
    let overshoot_bound = l - delay;

    let mut mave = MovingAverage::<_, DEFAULT_SMOOTHING>::init();
    let mut mmax = MovingMaximum::<_, DEFAULT_SMOOTHING>::init();

    let scaled = |smp: f32| {
        if smp > hi {
            hi / smp
        } else if smp < lo {
            lo / smp.max(0.001)
        } else {
            1.0
        }
    };

    let mut run = |s: f32| mave.update(mmax.update(s));

    for s in &a[..delay] {
        run(flat(*s));
    }

    // Allows simultaneous read and write on slice.
    let cells = Cell::from_mut(a).as_slice_of_cells();
    for (s1, s2) in cells[..overshoot_bound].iter().zip(cells[delay..].iter()) {
        s1.update(|s| s * scaled(run(flat(s2.get()))));
    }

    for s in &mut a[overshoot_bound..] {
        *s = *s * scaled(run(0.0));
    }
}
