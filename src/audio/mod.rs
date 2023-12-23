use cpal::{SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait};

mod audio_buffer;
use audio_buffer::AudioBuffer;

use std::ops::*;
use std::sync::{
    RwLock,
    atomic::{AtomicU8, Ordering},
};

use crate::{
    data::{
        SAMPLE_RATE
    },
};

/// Global sample array
pub type GSA = RwLock<AudioBuffer>;

pub type SampleArr = crate::WriteLock<AudioBuffer>;

static BUFFER: GSA = RwLock::new(AudioBuffer::new());

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

    let mut b = BUFFER.write().unwrap();

    let mut ns = get_no_sample().saturating_add(1);

    if get_normalizer() && ns < 2 {
        ns *= b.read_from_input(data) as u8;
        b.normalize();
	} else {
	    ns *= b.read_from_input_quiet(data) as u8;
	}

	// dbg!(b.input_size());

    NO_SAMPLE.store(ns, Ordering::Relaxed);
}

pub fn get_buf() -> SampleArr {
    BUFFER.write().unwrap()
}

pub fn get_no_sample() -> u8 {
	NO_SAMPLE.load(Ordering::Relaxed)
}

pub fn is_silent() -> bool {
	get_no_sample() > 0
}
/*
pub fn limiter<T>(
	a: &mut [T],
	limit: f64,
	hold_samples: usize,
	gain: f64
)
where T: Into<f64> + std::ops::Mul<f64, Output = T> + std::marker::Copy
{
	let mut replay_gain;
	let mut index = 0usize;
	let _hold_index = 0;
	let _amp = 0.0;
	let l = a.len();

    let hold_samples_double = hold_samples*2;

	let full_delay = hold_samples;

	let mut peak = Peak::init(limit, hold_samples_double);
	let mut moving_average = MovingAverage::init(limit, full_delay);

	let bound = l + full_delay;

	let mut getrg = |smp| {
		gain / moving_average.update(peak.update(smp))
	};

	while index < full_delay {
		let _ = getrg(a[index].into());
		index += 1;
	}

	while index < l {
		replay_gain = getrg(a[index].into());

		let smp = &mut a[index-full_delay];

		*smp = *smp *replay_gain;

		index += 1;
	}
	
	let smp = a[l-1].into();

	while index < bound {
		replay_gain = getrg(smp);

		let smp = &mut a[index-full_delay];
		*smp = *smp *replay_gain;
		
		index += 1;
	}
}

pub fn limiter_hard<T>(
	a: &mut [T],
	limit: f64,
	hold_samples: usize,
	gain: f64
) 
where T: Into<f64> + std::ops::Mul<f64, Output = T> + std::marker::Copy
{	
	let mut index = 0usize;
	let mut replay_gain;
	let _hold_index = 0;
	let _amp = 0.0;
	let l = a.len();

    let hold_samples_double = hold_samples;
		
	let full_delay = hold_samples;
		
	let mut peak = Peak::init(limit, hold_samples_double);

	let _bound = l + full_delay;
	
	let mut getrg = |smp| {
		gain / peak.update(smp)
	};

	while index < l {
		replay_gain = getrg(a[index].into());
		
		let smp = &mut a[index];
		
		*smp = *smp *replay_gain;
		
		index += 1;
	}
	
}

struct Peak {
    peak: f64,
	amp: f64,
	limit: f64,
	hold_for: usize,
	hold: usize,
}

impl Peak {
	pub fn init(
		limit: f64,
		hold_for: usize
	) -> Self {
		Self {
		    peak: limit,
			amp: limit,
			limit,
			hold_for,
			hold: 0
		}
	}
	
	pub fn update(&mut self, inp: f64) -> f64 {
		use crate::math::{
			fast::abs
		};
		
		let inp = f64::max(abs(inp), self.limit);
		
		if self.peak < inp || self.hold >= self.hold_for {
			self.peak = inp;
			self.hold = 0;
	    } else {
			self.hold += 1;
		}
		
		self.amp = self.peak;
		
	    // self.amp = interpolate::multiplicative_fall(self.amp, self.peak, self.limit, 1.0 / self.hold_for as f64);
		
		self.amp
	}
	
	// freeze the peakholder
	pub fn stall(&self) -> f64 {
		self.amp
	}
	
	pub fn update_and_get_gain(&mut self, new_amp: f64) -> f64 {
		1.0 / self.update(new_amp)
	}
}
*/
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

pub fn limiter<T>(
	a: &mut [T],
	limit: f64,
	hold_samples: usize,
	gain: f64
)
where T: Into<f64> + std::ops::Mul<f64, Output = T> + std::marker::Copy 
{
	use crate::math::{fast::abs, interpolate::smooth_step};
	
	struct PeakPoint {
		amp: f64,
		pos: usize
	}
	
	fn peak(amp: f64, pos: usize) -> PeakPoint {
		PeakPoint {amp, pos}
	}
	
	let mut peaks: Vec<PeakPoint> = Vec::with_capacity(12);
	
	peaks.push(peak(limit.max(abs((a[0]).into())), 0));
	
	let mut expo_amp = limit;
	
	let fall_factor = 1.0 - 1.0 / hold_samples as f64;
	
	for (i, ele) in a.iter().enumerate().skip(1) {
		let smp = abs((*ele).into());
		
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
