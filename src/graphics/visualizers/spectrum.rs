use std::io::Write;
use cpal::Sample;

use crate::audio_input::input_stream::{linear, shrink_stream_i16};

use crate::math;

use crate::constants::{VOL_SCL, FFT_SIZE, SMOOTHING, WIN_H, WIN_W, INCREMENT, pi2, pih};
use crate::graphics::graphical_fn;

static mut data_f1 : [(i32, i32); FFT_SIZE+1] = [(0, 0); FFT_SIZE+1];
static mut _i : usize = 0;

static mut index_vec : [(f32, usize, usize, i32, i32); WIN_W] = [(0.0, 0, 0, 0, 0); WIN_W];

pub unsafe fn prepare_index() {
	for i in 0..WIN_W {
		let idxf = (i*FFT_SIZE / WIN_W) as f32 * 0.0128;
		let idx = idxf.floor() as usize;
		let idx_next = idx+1;
		let t = (idxf.fract()*255.9) as i32;
        let scaling = (math::log2i(idx + 2) as i32)*3;

		index_vec[i] = (idxf, idx, idx_next, t, scaling);
	}
}

// Somewhere in this function and/or the fft one occasionally returns data_f with huge values which draws all over the screen, then comes back normal.
// Is this a bug?
pub unsafe fn draw_spectrum_pow2_std(buf : &mut Vec<u32>, stream : Vec<(f32, f32)>) {
    if stream.len() < FFT_SIZE { return; } 

    let scale = FFT_SIZE as f32 * WIN_H as f32 * 0.0625;
    let winhh = WIN_H >> 1;

    let wf = WIN_W as f32;
	let hf = WIN_H as f32;

	let smoothi = (SMOOTHING*255.9) as i32;

    let mut data_f = vec![(0.0f32, 0.0f32); FFT_SIZE];
    for i in 0..FFT_SIZE {
        data_f[i] = math::complex_mul(stream[(i+_i)%stream.len()], (VOL_SCL, 0.0));
    }
    //triangular(&mut data_f);

    //blackman_harris(&mut data_f);
    data_f = math::cooley_turky_fft_recursive(data_f);

    for sample in 0..FFT_SIZE {
        data_f1[sample].0 = math::interpolate::lineari(data_f1[sample].0, data_f[sample].0.abs() as i32, smoothi);
        data_f1[sample].1 = math::interpolate::lineari(data_f1[sample].1, data_f[sample].1.abs() as i32, smoothi);
    }
    

    //math::lowpass_bi_array(&mut data_f1, 0.9);

    graphical_fn::win_clear(buf);

    for i in 0..WIN_W {
        let idxf = index_vec[i].0;
        let idx = index_vec[i].1;
        let idx_next = index_vec[i].2;
        let t = index_vec[i].3;

        let scaling = index_vec[i].4;

        let bar_height1 =  ((math::interpolate::lineari(data_f1[idx].0, data_f1[idx_next].0, t)*scaling) as usize / 12);
        let bar_height2 =  ((math::interpolate::lineari(data_f1[idx].1, data_f1[idx_next].1, t)*scaling) as usize / 12);

        // let bar_height1 = math::fast_isqrt(bar_height1.pow(2) + bar_height2.pow(2));
        // let bar_height2 = bar_height1;

       // let bar_height1 =  (data_f1[idx].0 as usize /3).min(WIN_H);
       // let bar_height2 =  (data_f1[idx].1 as usize /3).min(WIN_H);

        let color1 = graphical_fn::rgb_to_u32((255 - 255*i/WIN_W) as u8, (255*bar_height1.min(WIN_H)/WIN_H) as u8, 128);
        let color2 = graphical_fn::rgb_to_u32((255 - 255*i/WIN_W) as u8, (255*bar_height2.min(WIN_H)/WIN_H) as u8, 128);

        graphical_fn::draw_rect(buf, i, (WIN_H - bar_height1.min(WIN_H)) >> 1, 1, bar_height1 >> 1, color1);
        graphical_fn::draw_rect(buf, i, winhh+1, 1, bar_height2 >> 1, color2);

        //graphical_fn::draw_rect(buf, i, winhh-1, 1, 2, color1);

        graphical_fn::draw_rect(buf, i, winhh-1, 1, 2, graphical_fn::apply_alpha(color1, (stream[((_i+i) >> 3) & 0xFF].0 * 32768.0) as i16 as u8));

    }

    _i = (_i+INCREMENT) & 1023;
}

