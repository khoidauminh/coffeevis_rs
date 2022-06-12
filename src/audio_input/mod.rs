use cpal::{Data, Sample, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crate::{buf, constants::{Parameters, SAMPLE_SIZE}};
//use rustfft::num_complex::Complex;

pub fn get_source<T, D>(f: D, para: &mut Parameters) -> cpal::Stream
where
    T : Sample,
    D : FnMut(&[T], &cpal::InputCallbackInfo) + Send + 'static,
{

    let host = cpal::default_host();
    #[cfg(target_os = "windows")] {
        host = cpal::host_from_id(cpal::HostId::Asio).expect("failed to initialise ASIO host");
    }

    let device = host.default_input_device().expect("no input device available");

    let err_fn = |err| eprintln!("an error occurred on the input audio stream: {}", err);

    let mut supported_configs_range = device.supported_input_configs()
        .expect("error while querying configs");

    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    let sample_format = supported_config.sample_format();


    let mut config: cpal::StreamConfig = supported_config.into();
    config.channels = 2;

    //unsafe{crate::constants::SAMPLE_RATE = config.sample_rate.0 as u32}

    //println!("Sample rate: {:?}", config.sample_rate);

    //channels = config.channels as usize;

    match sample_format {
        SampleFormat::F32 => device.build_input_stream(&config, f, err_fn),
        SampleFormat::I16 => device.build_input_stream(&config, f, err_fn),
        SampleFormat::U16 => device.build_input_stream(&config, f, err_fn),
    }.unwrap()
}

// static mut channels: usize = 2;

pub fn read_samples<T: cpal::Sample>(data : &[T], _ : &cpal::InputCallbackInfo) {
    let mut i = 0;
	for sample in 0..SAMPLE_SIZE {
		unsafe{buf[sample].0 = data[i].to_f32()};
		i += 1;
		unsafe{buf[sample].1 = data[i].to_f32()};
	    i += 1;
		i *= (i < data.len()) as usize;
	}
}
