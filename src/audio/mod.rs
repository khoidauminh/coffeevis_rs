use cpal::{SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait};

mod audio_buffer;
use audio_buffer::AudioBuffer;

use std::ops::*;
use std::sync::{
    RwLock,
    Mutex,
    MutexGuard,
    atomic::{AtomicU8, Ordering},
};

use crate::{
    data::{
        SAMPLE_RATE
    },
};

/// Global sample array
pub type GSA = Mutex<AudioBuffer>;

pub type SampleArr<'a> = MutexGuard<'a, AudioBuffer>;

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
    #[cfg(target_os = "windows")] {
        host = cpal::host_from_id(cpal::HostId::Asio).expect("failed to initialise ASIO host");
    }

    let device = host.default_input_device().expect("no input device available");

    let err_fn = |err| eprintln!("an error occurred on the input audio stream: {}", err);

    let supported_configs_range = device.default_input_config()
        .expect("error while querying configs");

    let supported_config = supported_configs_range; //.next()

    let sample_format = supported_config.sample_format();

    let mut config: cpal::StreamConfig = supported_config.into();
    config.channels = 2;
    config.sample_rate = cpal::SampleRate(SAMPLE_RATE as u32);

    match sample_format {
        SampleFormat::F32 => device.build_input_stream(&config, move |data, _: &_| read_samples::<f32>(data), err_fn, None),
        SampleFormat::I16 => device.build_input_stream(&config, move |data, _: &_| read_samples::<i16>(data), err_fn, None),
        SampleFormat::U16 => device.build_input_stream(&config, move |data, _: &_| read_samples::<u16>(data), err_fn, None),
        _ => todo!()
    }.unwrap()
}

pub fn read_samples<T: cpal::Sample<Float = f32>>(data: &[T]) {
    let _l = data.len();

    let _i = 0usize;
    let _s = 0usize;

    let mut b = BUFFER.lock().unwrap();

    let mut ns = get_no_sample().saturating_add(1);

    if get_normalizer() && ns < 2 {
        ns *= b.read_from_input(data) as u8;
        b.normalize();
	} else {
	    ns *= b.read_from_input_quiet(data) as u8;
	}
	
    NO_SAMPLE.store(ns, Ordering::Relaxed);
}

pub fn get_buf() -> SampleArr<'static> {
    BUFFER.lock().unwrap()
}

pub fn get_no_sample() -> u8 {
	NO_SAMPLE.load(Ordering::Relaxed)
}

pub fn is_silent() -> bool {
	get_no_sample() > 0
}

use crate::misc::stackvec::StackVec;

pub struct MovingAverage<T> {
	size: usize,
	index: usize,
	vec: StackVec<T, 15>,
	sum: T,
	denominator: f64,
	average: T
}

impl<T> MovingAverage<T>
where 
    T:
        Add<Output = T> +
        Sub<Output = T> +
        Mul<f64, Output = T> +
        std::marker::Copy,
    f64: 
        Mul<T, Output = T>
{
	pub fn init(val: T, size: usize) -> Self {
		Self {
			size,
			index: 0,
			vec: StackVec::init(val, size),
			denominator: (size as f64).recip(),
			sum: size as f64 * val,
		    average: val
		}
	}
	
	fn pop(&mut self, val: T) -> T {
	    let out = self.vec[self.index];
	    
	    self.vec[self.index] = val;
		
		self.index = 
			crate::math::increment(
				self.index, self.size
		    );
		
		out
	}
	
	pub fn update(&mut self, val: T) -> T {
        let old = self.pop(val);
        
        self.average = self.denominator * self.sum;
        
        self.sum = self.sum - old + val;
		
		self.average
	}

	pub fn current(&self) -> T {
        self.average
	}
}

use crate::math::interpolate::smooth_step;

struct PeakPoint {
	amp: f64,
	pos: usize
}

fn peak(amp: f64, pos: usize) -> PeakPoint {
	PeakPoint {amp, pos}
}

pub fn limiter<T>(
	a: &mut [T],
	limit: f64,
	hold_samples: usize,
	gain: f64,
	flattener: fn(T) -> f64 
)
where T: Into<f64> + std::ops::Mul<f64, Output = T> + std::marker::Copy 
{
	let mut peaks: Vec<PeakPoint> = Vec::with_capacity(12);
	
	peaks.push(peak(limit.max( flattener(a[0]).abs() ), 0));
	
	let mut expo_amp = limit;
	
	let fall_factor = 1.0 - 1.0 / hold_samples as f64;

	for (i, ele) in a.iter().enumerate().skip(1) {
		let smp = flattener(*ele).abs();
		
		if expo_amp > limit {
			expo_amp = limit.max(expo_amp * fall_factor);
		}
		
		if smp > expo_amp {
			expo_amp = smp;
			peaks.push(peak(smp, i));
		}
	}
	
	match peaks.last() {
		Some(p) if p.pos < a.len() 
			=> peaks.push(peak(expo_amp, a.len())),
		_ 	=> {}
	}
	
	peaks.iter_mut().for_each(|peak| {
		peak.amp = 1.0 / peak.amp;
	});
	
	peaks.windows(2).for_each(|window| {
		let head = &window[0];
		let tail = &window[1];
		
		let range = tail.pos - head.pos +1;
		let rangef = range as f64;
		let range_recip = 1.0 / rangef;
		
		a[head.pos..tail.pos]
		.iter_mut()
		.enumerate()
		.for_each(|(i, smp)| {
			let t = i as f64 * range_recip;
			let scale = smooth_step(head.amp, tail.amp, t);
			*smp = *smp *scale * gain;
		});
	});
}
