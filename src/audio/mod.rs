use cpal::{Data, Sample, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

mod audio_buffer;
use audio_buffer::AudioBuffer;

use std::sync::{
    Arc, RwLock,
    atomic::{AtomicU8, AtomicBool, AtomicUsize, Ordering},
};

use crate::{
    data::{
        Program,
        SAMPLE_SIZE,
        SAMPLE_RATE
    },
    math::Cplx,
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

    let mut supported_configs_range = device.default_input_config()
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
    let l = data.len();

    let mut i = 0usize;
    let mut s = 0usize;

    let mut b = BUFFER.write().unwrap();

    let mut ns = get_no_sample().saturating_add(1);

    ns *= b.read_from_input(data) as u8;
    
    if get_normalizer() {
		b.normalize();
	}

	// println!("SAMPLE_READ");

    NO_SAMPLE.store(ns, Ordering::Relaxed);
}

pub fn get_buf() -> SampleArr {
    BUFFER.write().unwrap()
}

pub fn get_no_sample() -> u8 {
	NO_SAMPLE.load(Ordering::Relaxed)
}

pub fn limiter(
	a: &mut [Cplx<f32>],
	limit: f32,
	attack_samples: usize, 
	release_samples: usize,
	hold_samples: usize
) {
	use crate::math::interpolate::{envelope, subtractive_fall_hold};
	
	let mut index = 0usize;
	let mut replay_gain = 1.0f32;
	let mut hold_index = 0;
	let mut amp = 0.0;
	let mut l = a.len();
	
	let attack_amount = 1.0 / attack_samples as f32;
	let release_amount = 1.0 / release_samples as f32; 

	let mut peak_holder = 
		PeakHolder::init(
			limit, 
			release_amount,
			hold_samples
		);
		
	let hold_samples_half = hold_samples/2;
	
	let full_delay = hold_samples + attack_samples;
		
	let mut delay = DelaySmooth::init(full_delay, limit);
	
	while index < l {
		replay_gain = delay.update(peak_holder.update(a[index].l1_norm())).recip();
	
		if let Some(smp) = a.get_mut(index.wrapping_sub(full_delay)) {
			*smp = smp.scale(replay_gain);
		}
	
		index += 1;
	}
}

struct PeakHolder {
	amp: f32,
	limit: f32,
	release: f32, 
	hold_for: usize,
	hold: usize,
}

impl PeakHolder {
	pub fn init(
		limit: f32,
		release: f32,
		hold_for: usize
	) -> Self {
		Self {
			amp: limit,
			limit: limit,
			release: release, 
			hold_for: hold_for,
			hold: 0
		}
	}
	
	pub fn update(&mut self, new_amp: f32) -> f32 {
		use crate::math::interpolate::linearf;
	
		let new_amp = f32::max(new_amp, self.limit);
		
		if new_amp > self.amp {
			self.amp = new_amp;
			self.hold = 0;
		} else if self.hold >= self.hold_for {
			self.amp = linearf(self.amp, new_amp, self.release);
		} else {
			self.hold += 1;
		}
		
		self.amp
	}
	
	pub fn update_and_get_gain(&mut self, new_amp: f32) -> f32 {
		1.0 / self.update(new_amp)
	}
}

struct DelaySmooth {
	delay_for: usize,
	denominator: f32,
	index: usize,
	vec: Vec<f32>,
	sum: f32,
}

impl DelaySmooth {
	pub fn init(delay_for: usize, val: f32) -> Self {
		let size = delay_for+1;
		let denominator = 1.0 / size as f32; 
		Self {
			delay_for: size,
			denominator: denominator,
			index: 0,
			vec: vec![val; size],
			sum: size as f32 * val
		}
	}
	
	pub fn update(&mut self, val: f32) -> f32 {	
		let out = self.sum * self.denominator;
		
		self.sum += val;
		self.sum -= self.vec[self.index];
		
		self.vec[self.index] = val;
		
		self.index = 
			crate::math::increment_index(
				self.index, self.delay_for
			);
		
		out
	}
}
