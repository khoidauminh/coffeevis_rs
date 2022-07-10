use cpal::Sample;

use crate::math;
use math::{Cplx, cplx_0};

use crate::config::{FFT_SIZE, INCREMENT, pi2, pih, POWER};
use crate::config::Parameters;
use crate::graphics::graphical_fn;

//static mut _i : usize = 0;

pub fn draw_spectrum(buf : &mut [u32], stream : &[Cplx], para: &mut Parameters ) {
    if stream.len() < FFT_SIZE { return; }

    let l = stream.len();

    let scale = FFT_SIZE as f32 * para.WIN_H as f32 * 0.0625;
    let winhh = para.WIN_H >> 1;

    let wf = para.WIN_W as f32;
	let hf = para.WIN_H as f32;

	//let smoothi = (SMOOTHING*255.9) as i32;

    let mut data_l = [cplx_0(); FFT_SIZE];
    let mut data_r = [cplx_0(); FFT_SIZE];

    let mut si = para._i;

    for i in 0..FFT_SIZE {
        si = math::advance_with_limit(si, l);

        let l = (stream[si].0*para.VOL_SCL, 0.0);
        let r = (stream[si].1*para.VOL_SCL, 0.0);

        data_l[i] = l;
        data_r[i] = r;
    }

    math::fft_inplace(&mut data_l, POWER);
    math::fft_inplace(&mut data_r, POWER);
    //triangular(&mut data_f);

    //blackman_harris(&mut data_f);

    for sample in 0..FFT_SIZE {
        para.spectrum_smoothing_ft[sample].0 = math::interpolate::linearf(
            para.spectrum_smoothing_ft[sample].0,
            (data_l[sample].0.abs()+data_l[sample].1.abs()*0.2),
            para.SMOOTHING
        );
        para.spectrum_smoothing_ft[sample].1 = math::interpolate::linearf(
            para.spectrum_smoothing_ft[sample].1,
            (data_r[sample].0.abs()*0.2+data_r[sample].1.abs()),
            para.SMOOTHING);
    }

    //math::lowpass_bi_array(&mut para.spectrum_smoothing_ft, 0.9);

    graphical_fn::win_clear(buf);

    let winlog = wf.log2();

    for i in 0..para.WIN_W {
        let idxf = (i as f32 / wf)* winlog *8.0;
		let idx = idxf.floor() as usize;
		let idx_next = idxf.ceil() as usize;
		let t = idxf.fract();
        let scaling = math::fast_flog2(idxf + 2.0) *hf * 0.02;

        let bar_height1 =  ((math::interpolate::bezierf(para.spectrum_smoothing_ft[idx].0, para.spectrum_smoothing_ft[idx_next].0, t)*scaling) as usize / 12);
        let bar_height2 =  ((math::interpolate::bezierf(para.spectrum_smoothing_ft[idx].1, para.spectrum_smoothing_ft[idx_next].1, t)*scaling) as usize / 12);

        // let bar_height1 = math::fast_isqrt(bar_height1.pow(2) + bar_height2.pow(2));
        // let bar_height2 = bar_height1;

       // let bar_height1 =  (para.spectrum_smoothing_ft[idx].0 as usize /3).min(para.WIN_H);
       // let bar_height2 =  (para.spectrum_smoothing_ft[idx].1 as usize /3).min(para.WIN_H);

        let color1 = graphical_fn::rgb_to_u32((255 - 255*i/para.WIN_W) as u8, (255*bar_height1.min(para.WIN_H)/para.WIN_H) as u8, 128);
        let color2 = graphical_fn::rgb_to_u32((255 - 255*i/para.WIN_W) as u8, (255*bar_height2.min(para.WIN_H)/para.WIN_H) as u8, 128);

        graphical_fn::draw_rect(buf, i, (para.WIN_H - bar_height1.min(para.WIN_H)) >> 1, 1, bar_height1 >> 1, color1, para);
        graphical_fn::draw_rect(buf, i, winhh+1, 1, bar_height2 >> 1, color2, para);

        //graphical_fn::draw_rect(buf, i, winhh-1, 1, 2, color1);

        graphical_fn::draw_rect(buf, i, winhh-1, 1, 2, graphical_fn::apply_alpha(color1, (stream[((para._i+i) >> 3) & 0xFF].0 * 32768.0) as i16 as u8), para);

    }

    para._i = (para._i+INCREMENT) & 1023;
}
