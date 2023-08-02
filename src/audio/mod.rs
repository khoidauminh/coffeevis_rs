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
	
	let first_iter = attack_samples + (hold_samples | 1)/2;
	while 
		index < first_iter && 
		index < l
	{
		replay_gain = replay_gain1(a[index], limit, &mut amp, attack_amount, release_amount, hold_samples, &mut hold_index);
		
		index += 1;
	}
	
	while index < l {
		let prev = &mut a[index-first_iter];
		*prev = prev.scale(replay_gain);
		
		replay_gain = replay_gain1(a[index], limit, &mut amp, attack_amount, release_amount, hold_samples, &mut hold_index);
	
		index += 1;
	}
}

fn replay_gain1(smp: Cplx<f32>, limit: f32, amp: &mut f32, attack: f32, release: f32, hold: usize, hold_index: &mut usize) -> f32 {
	use crate::math::interpolate::linearf;
	
	let tmp_amp = smp.l1_norm().max(limit);
	
	if tmp_amp > *amp {
		*amp = tmp_amp; //(*amp + tmp_amp*attack).min(tmp_amp);
		*hold_index = 0;
	} else if *hold_index >= hold {
		*amp = crate::math::interpolate::linearf(*amp, tmp_amp, release);
	} else {
		*hold_index += 1;
	}
	
	/*if tmp_amp > *amp || *hold_index >= hold {
		*amp = tmp_amp;
		*hold_index = 0;
	} else {
		*hold_index += 1;
	}*/

	
	1.0 / *amp
}
