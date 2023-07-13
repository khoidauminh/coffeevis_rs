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

static NORMALIZE: AtomicBool = AtomicBool::new(true);

pub fn set_normalizer(b: bool) {
	NORMALIZE.store(b, Ordering::Relaxed);
}

pub fn get_normalizer() -> bool {
	NORMALIZE.load(Ordering::Relaxed)
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

    ns *=
		if get_normalizer() {
			b.read_from_input_with_normalizer(data) as u8
		} else {
			b.read_from_input(data) as u8
		}
	;

    NO_SAMPLE.store(ns, Ordering::Relaxed);
}

pub fn get_buf() -> SampleArr {
    BUFFER.write().unwrap()
}

pub fn get_no_sample() -> u8 {
	NO_SAMPLE.load(Ordering::Relaxed)
}
