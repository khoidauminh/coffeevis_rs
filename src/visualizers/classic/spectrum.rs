use cpal::Sample;

use std::sync::RwLock;

use crate::math::{self, Cplx, interpolate::*, TAU, PIH};
use crate::data::{FFT_SIZE, INCREMENT, FFT_POWER, Program};
use crate::graphics::{P2, blend};

// const COPY_SIZE: usize = FFT_SIZE / 2;

const RANGE: usize = 64;
const RANGEF: f32   = RANGE as f32;
const FFT_SIZEF: f32 = FFT_SIZE as f32;
const FFT_SIZEF_RECIP: f32 = 1.0 / FFT_SIZEF;

static DATA: RwLock<[Cplx<f32>; RANGE+1]> = RwLock::new([Cplx::<f32>::zero(); RANGE+1]);
static MAX: RwLock<f32> = RwLock::new(1.0);

fn l1_norm_slide(a: Cplx<f32>, t: f32) -> f32 {
	a.x.abs()*t + a.y.abs()*(1.0-t)
}


fn index_scale(x: f32) -> f32 {
    //math::fast::fsqrt(x)*x
    math::fast::unit_exp2_0(x)
}

fn volume_scale(x: f32) -> f32 {
    2.0*x//crate::math::fast::fsqrt(x)
    //let x1 = x*0.25 + 0.25;
    //(x - 2.5).max(0.0)*3.0
    //(x*3.0).powi(2)
    //math::fast::fsqrt(math::fast::fsqrt(x))*x
	//math::fast::unit_exp2_0(2.0*x)
}

fn prepare(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    const WINDOW: usize = 2*FFT_SIZE/3;
    const NORMALIZE: f32 = 1.0 / WINDOW as f32;

    let mut fft = [Cplx::<f32>::zero(); FFT_SIZE];

    fft.iter_mut()
    .take(WINDOW)
    .enumerate()
    .for_each(|(i, smp)| {
        let idx = i*2;
        *smp = stream[idx];
    });

    math::fft(&mut fft);
    
    let mut LOCAL = DATA.write().unwrap();
    
    math::highpass_inplace(&mut fft[..LOCAL.len()]);
    math::highpass_inplace(&mut fft[FFT_SIZE-LOCAL.len()-1..]);

    let fall_factor = 0.333*prog.SMOOTHING.powi(2);

    // let pre_scale = 0.6*(0.5 + stream.amplitude()*0.5);

    LOCAL
    .iter_mut()
	.enumerate()
	.for_each(
		|(i, smp)| {
			let rev_i = FFT_SIZE-i-1;
            
            // Avoids having to evaluate a 2nd FFT.
            //
            // This leverages the the linear and symetrical
            // property of the FFT.
            // 
            // Errors accumulating in the output (due to 
            // approximating sin and cos) can be tolerated.
			let fft_1 = fft[i];
			let fft_2 = fft[rev_i].conj();
						
            let y = (fft_1 + fft_2).l1_norm();
            let x = (fft_1 - fft_2).times_i().l1_norm();

            let scalef = math::fft_scale_up(i, RANGE)* NORMALIZE;

    	    smp.x = multiplicative_fall(smp.x, x*scalef, 0.0, fall_factor);
			smp.y = multiplicative_fall(smp.y, y*scalef, 0.0, fall_factor);

		}
	);
	
	stream.rotate_left(WINDOW);
}

pub const draw_spectrum: crate::VisFunc = |prog, stream| {
    let l = stream.len();

    let (w, h) = prog.pix.sizet();

    // let scale = FFT_SIZE as f32 * prog.pix.height() as f32 * 0.0625;
    let winwh = w >> 1;
    
    prepare(prog, stream);
    
    let wf = w as f32;
	let hf = h as f32;
	
	let binding = DATA.read().unwrap();
    let normalized = binding.as_slice();

//	let mut normalized = [Cplx::<f32>::zero(); RANGE+1];
//	normalized.copy_from_slice(LOCAL.as_slice());
	
//	let mut current_max = MAX.write().unwrap();
//	*current_max = math::normalize_max(&mut normalized, 0.01, 1.0, *current_max, 0.002);

    prog.clear_pix();

    let winlog = math::fast::flog2(wf);

    let mut if32: f32 = 0.0;

    const INTERVAL: f32 = 1.0;

    let height_prescale = wf * prog.VOL_SCL *0.333;

    for i in 0..h as i32 {
        let i_rev = (h as i32) - i;

        let i_ratio = i_rev as f32 / hf;

        let slide_output = index_scale(i_ratio);

        let idxf = slide_output * RANGEF;

	    let idx = idxf.floor() as usize;
	    let idx_next = idxf.ceil() as usize;
	    let t = idxf.fract();

        let scale = /*(math::fast::fsqrt(idxf) + 0.5)*2.0**/ height_prescale;

        let bar_temp1 = linearf(normalized[idx].x, normalized[idx_next].x, t)*scale;
        let bar_temp2 = linearf(normalized[idx].y, normalized[idx_next].y, t)*scale;

        let bar_width_l = bar_temp1;
        let bar_width_r = bar_temp2;

		let color  = u32::from_be_bytes([0xFF, (255 - 255*i_rev/h) as u8, 0, 128]);
        let color1 = color | ((255.0 *bar_width_l.min(wf) / wf) as u32) << 8;
        let color2 = color | ((255.0 *bar_width_r.min(wf) / wf) as u32) << 8;

        let rect_l = P2::new((wf - bar_width_l).max(0.0) as i32 / 2, i);
        // let rect_l_size = (bar_width_l as usize / 2, 1);

        let rect_r = P2::new((bar_width_r as i32 / 2 + winwh).min(w), i);

        let middle = P2::new(winwh, i);
        // let rect_r_size = (bar_width_r as usize / 2, 1);

        prog.pix.draw_rect_xy(rect_l, middle, color1);
        prog.pix.draw_rect_xy(middle, rect_r, color2);

		let alpha = (128.0 + stream[i as usize / 2].x*32768.0) as u8;
        prog.pix.draw_rect_wh(P2::new(winwh -1, i), 2, 1, blend::u32_fade(color, alpha));
    }
};
