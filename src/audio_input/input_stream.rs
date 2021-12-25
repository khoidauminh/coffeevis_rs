use cpal::{Data, Sample, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
//use rustfft::num_complex::Complex;

pub unsafe fn get_source<T, D>(f : D) -> cpal::Stream
where
    T : Sample,
    D : FnMut(&[T], &cpal::InputCallbackInfo) + Send + 'static,
{

    let host = cpal::default_host();
    #[cfg(target_os = "windows")] {
        host = cpal::host_from_id(cpal::HostId::Asio).expect("failed to initialise ASIO host");
    }

    let device = host.default_input_device().expect("no output device available");

    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);

    let mut supported_configs_range = device.supported_input_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    let sample_format = supported_config.sample_format();
    let config = supported_config.into();

    match sample_format {
        SampleFormat::F32 => device.build_input_stream(&config, f, err_fn),
        SampleFormat::I16 => device.build_input_stream(&config, f, err_fn),
        SampleFormat::U16 => device.build_input_stream(&config, f, err_fn),
    }.unwrap()
}

pub fn shrink_stream_i16(stream : &Vec<i16>, new_size : usize) -> Vec<i16> {
    let l = stream.len();
    let mut new_stream = vec![0i16 ; new_size];
    let average = (l / new_size) as i16;

    for i in 0..l {
        let di = i*new_size/l;
        new_stream[di] += stream[i] / average;
    }

    new_stream
}

pub fn linear(x1 : f32, x2 : f32, t : f32) -> f32 {
    x1 + (x2-x1)*t
}
