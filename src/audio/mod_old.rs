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
pub type GSA = RwLock<[Cplx<f32>; SAMPLE_SIZE]>;

pub type SampleArr = crate::WriteLock<AudioBuffer>;

static BUFFER: GSA = RwLock::new([Cplx::<f32>::zero(); SAMPLE_SIZE]);
// static BUFFER_TMP: GSA = RwLock::new([Cplx::<f32>::zero(); SAMPLE_SIZE]);

static NO_SAMPLE: AtomicU8 = AtomicU8::new(0);
static OFFSET: AtomicUsize = AtomicUsize::new(0);
// Timer that runs when input is silence.

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
         //.expect("no supported config?!")
        // .with_sample_rate(cpal::SampleRate(SAMPLE_RATE));
         // .with_max_sample_rate();

    let sample_format = supported_config.sample_format();

    let mut config: cpal::StreamConfig = supported_config.into();
    config.channels = 2;
    config.sample_rate = cpal::SampleRate(SAMPLE_RATE as u32);

    //unsafe{crate::config::SAMPLE_RATE = config.sample_rate.0 as u32}

    //println!("Sample rate: {:?}", config.sample_rate);

    //channels = config.channels as usize;



    match sample_format {
        SampleFormat::F32 => device.build_input_stream(&config, move |data, _: &_| read_samples::<f32>(data), err_fn),
        SampleFormat::I16 => device.build_input_stream(&config, move |data, _: &_| read_samples::<i16>(data), err_fn),
        SampleFormat::U16 => device.build_input_stream(&config, move |data, _: &_| read_samples::<u16>(data), err_fn),
    }.unwrap()
}

pub fn read_samples<T: cpal::Sample>(data: &[T]) {
    let l = data.len();

    let mut i = 0usize;
    let mut s = 0usize;

    let mut b = BUFFER.write().unwrap();

//    while s < SAMPLE_SIZE && i < l {
//		b[s].0 = data[i].to_f32();
//        i += 1;
//		b[s].1 = data[i].to_f32();
//	    i += 1;
//        s += 1;
//	    //i *= (i < l) as usize;
//	}

    let mut ns = get_no_sample().saturating_add(1);
    /*
    let left_right_sum = data[l-1].to_i16().abs() + data[l-2].to_i16().abs();

	if left_right_sum < 256 {
		NO_SAMPLE.store(ns.saturating_add(1), Ordering::Relaxed);
	} else {
		NO_SAMPLE.store(0, Ordering::Relaxed);
	}

	if ns > crate::data::SILENCE_LIMIT {
		return;
	}*/

	let l = data.len() /2;
	let fadel = l /4;
	let mut silence = true;

    b.iter_mut()
    .zip(data.chunks(2))
    .enumerate()
    .for_each(|(i, (mut s, chunk))| {

        let mut smp = Cplx::<f32>::new(chunk[0].to_f32(), chunk[1].to_f32());

        silence &= (smp.l1_norm() < 1e-2);

        if i < fadel
        {
			let fade = i as f32 / fadel as f32;
			smp = crate::math::interpolate::linearfc(*s, smp, fade);
		}
		else if l-i-1 < fadel
		{
			let fade = (l-i-1) as f32 / fadel as f32;
			smp = crate::math::interpolate::linearfc(*s, smp, fade);
		}

		*s = smp;
    });

    ns *= silence as u8;

    // BUFFER.write().unwrap().copy_from_slice(&*b);

	NO_SAMPLE.store(ns, Ordering::Relaxed);
}

pub fn get_buf() -> SampleArr {

    // refresh_buf();
    // let o = OFFSET.load(Ordering::Relaxed);
    // OFFSET.store((o+crate::ROTATE_SIZE) % crate::SAMPLE_SIZE, Ordering::Relaxed);

   /* let sum =
		buf
		.read()
		.unwrap()
		.first()
		.unwrap_or(&Cplx::zero())
		.mag();
		*/
    BUFFER.write().unwrap()
}

pub fn get_no_sample() -> u8 {
	NO_SAMPLE.load(Ordering::Relaxed)
}
