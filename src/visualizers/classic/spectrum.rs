

use std::sync::RwLock;

use crate::math::{self, Cplx, interpolate::*};
use crate::data::{FFT_SIZE};
use crate::graphics::{P2, blend::Blend};

// const COPY_SIZE: usize = FFT_SIZE / 2;

const RANGE: usize = 64;
const RANGEF: f64   = RANGE as f64;
const FFT_SIZEF: f64 = FFT_SIZE as f64;
const FFT_SIZEF_RECIP: f64 = 1.0 / FFT_SIZEF;

static DATA: RwLock<[Cplx; RANGE+1]> = RwLock::new([Cplx::zero(); RANGE+1]);
static MAX: RwLock<f64> = RwLock::new(1.0);

fn l1_norm_slide(a: Cplx, t: f64) -> f64 {
	a.x.abs()*t + a.y.abs()*(1.0-t)
}


fn index_scale(x: f64) -> f64 {
    //math::fast::fsqrt(x)*x
    math::fast::unit_exp2_0(x)
    //x.sqrt()*x
}

fn volume_scale(x: f64) -> f64 {
    2.0*x//crate::math::fast::fsqrt(x)
    //let x1 = x*0.25 + 0.25;
    //(x - 2.5).max(0.0)*3.0
    //(x*3.0).powi(2)
    //math::fast::fsqrt(math::fast::fsqrt(x))*x
	//math::fast::unit_exp2_0(2.0*x)
}

fn prepare(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    const WINDOW: usize = 2*FFT_SIZE/3;
    const NORMALIZE: f64 = 1.0 / FFT_SIZE as f64;
	let fall_factor = 0.5*prog.SMOOTHING.powi(2) * prog.FPS as f64 * 0.006944444;
	let mut LOCAL = DATA.write().unwrap();
    let mut fft = [Cplx::zero(); FFT_SIZE];

    fft.iter_mut()
    .take(WINDOW)
    .enumerate()
    .for_each(|(i, smp)| {
        let idx = i*2;
        *smp = stream[idx];
    });

    math::fft_stereo(&mut fft, RANGE, true);
    
    let RANGE1 = RANGE+1;
    	
	for i in 0..RANGE1 {
		let scalef = math::fft_scale_up(i, RANGE);
		fft[i] = fft[i]*scalef;
	}
	
	crate::audio::limiter(&mut fft[0..RANGE1], 1.35, 20, 1.);

    LOCAL
    .iter_mut()
    .zip(fft.iter())
	.for_each(
		|(smp, si)| {
    	    smp.x = multiplicative_fall(smp.x, si.x, 0.0, fall_factor);
			smp.y = multiplicative_fall(smp.y, si.y, 0.0, fall_factor);

		}
	);

	stream.auto_rotate();
}

pub fn draw_spectrum(
	prog: &mut crate::data::Program, 
	stream: &mut crate::audio::SampleArr
) {
    let _l = stream.len();

    let (w, h) = prog.pix.sizet();

    // let scale = FFT_SIZE as f64 * prog.pix.height() as f64 * 0.0625;
    let winwh = w >> 1;
    
    prepare(prog, stream);
    
    let wf = w as f64;
	let hf = h as f64;
	
	let wf_recip = 1.0 / wf;
	let hf_recip = 1.0 / hf;
	
	let binding = DATA.read().unwrap();
    let normalized = binding.as_slice();

//	let mut normalized = [Cplx::zero(); RANGE+1];
//	normalized.copy_from_slice(LOCAL.as_slice());
	
//	let mut current_max = MAX.write().unwrap();
//	*current_max = math::normalize_max(&mut normalized, 0.01, 1.0, *current_max, 0.002);

    // prog.pix.clear();

    let _winlog = wf.log2();

    let _if64: f64 = 0.0;

    const INTERVAL: f64 = 1.0;

    let height_prescale = wf * prog.VOL_SCL;
    
    prog.pix.clear();

    for i in 0..h {
        let i_rev = h - i;

        let i_ratio = i_rev as f64 * hf_recip;

        let slide_output = index_scale(i_ratio);

        let idxf = slide_output * RANGEF;

	    let idx = idxf.floor() as usize;
	    let idx_next = idxf.ceil() as usize;
	    let t = idxf.fract();

        let scale = /*(math::fast::fsqrt(idxf) + 0.5)*2.0**/ height_prescale;

        let bar_temp1 = smooth_step(normalized[idx].x, normalized[idx_next].x, t)*scale;
        let bar_temp2 = smooth_step(normalized[idx].y, normalized[idx_next].y, t)*scale;

        let bar_width_l = bar_temp1;
        let bar_width_r = bar_temp2;

		let channel_l = (255.0 *bar_width_l.min(wf) * wf_recip) as u32;
		let channel_r = (255.0 *bar_width_r.min(wf) * wf_recip) as u32;

		let color  = u32::from_be_bytes([0xFF, (255 - 255*i_rev/h) as u8, 0, (128 + 96*i_rev/h) as u8]);
		
        let color1 = color | channel_l << 8;
        let color2 = color | channel_r << 8;

        let rect_l = P2::new((wf - bar_width_l).max(0.0) as i32 / 2, i);
        // let rect_l_size = (bar_width_l as usize / 2, 1);

        let rect_r = P2::new(((bar_width_r as i32 + 1) / 2 + winwh).min(w), i);

        let middle = P2::new(winwh+1, i);
        // let rect_r_size = (bar_width_r as usize / 2, 1);
		
		//prog.pix.clear_row(i as usize);
		
		//prog.pix.draw_rect_xy(P2::new(0, i), P2::new(rect_l.x, i), prog.background);
		//prog.pix.draw_rect_xy(P2::new(w - rect_r.x, i), P2::new(w, i), prog.background);
		
        prog.pix.draw_rect_xy(rect_l, middle, color1);
        prog.pix.draw_rect_xy(middle, rect_r, color2);

		let alpha = (128.0 + stream[i as usize / 2].x*32768.0) as u8;
		// let bg = prog.pix.background & 0x00_FF_FF_FF | alpha;
        prog.pix.draw_rect_wh_by(
			P2::new(winwh -1, i), 
			2, 1, 
			u32::mix(prog.pix.background, color.set_alpha(alpha)).copy_alpha(prog.pix.background), 
			u32::over
		);
    }
}
