use std::sync::RwLock;

use crate::data::FFT_SIZE;
use crate::graphics::{P2, blend::Blend};
use crate::math::{self, Cplx};

const color: [u32; 3] = [0x66ff66, 0xaaffff, 0xaaaaff];

const FFT_SIZE_HALF: usize = FFT_SIZE / 2;

static DATA: RwLock<Vec<f64>> = RwLock::new(Vec::new());
static MAX: RwLock<f64> = RwLock::new(0.0);

const FFT_SIZE_RECIP: f64 = 2.0 / FFT_SIZE as f64;
const normalize_factor: f64 = FFT_SIZE_RECIP;
const BARS: usize = 48;

fn dynamic_smooth(t: f64, a: f64) -> f64 {
	((t - a) / a).powi(2)
}

fn dynamic_smooth2(t: f64, b: f64) -> f64 {
	(b*t + 1.3).recip()
}

fn prepare(stream: &mut crate::audio::SampleArr, bar_num: usize, volume_scale: f64) {
	let bar_num = bar_num +1;
	
	let bnf = bar_num as f64;
	let l = stream.len();
	
	let mut LOCAL = DATA.write().unwrap();

	if bar_num != LOCAL.len() {
		LOCAL.resize(bar_num, 0.0);
	}

    let mut data_f = [Cplx::zero(); FFT_SIZE];
    data_f
    .iter_mut()
    .take(FFT_SIZE_HALF)
    .enumerate()
    .for_each(|(i, smp)| 
        *smp = stream[i*2].scale(FFT_SIZE_RECIP)
    );
    
    //~ math::blackmannuttall::perform_window(&mut data_f);
    
    math::fft_half(&mut data_f);
    
    let mut max = MAX.write().unwrap();
    
    // crate::math::highpass_inplace(&mut data_f[..bar_num]);
    
    let mut data_f = 
        data_f
        .iter_mut()
        .take(bar_num)
        .enumerate()
        .map(|(i, smp)| {
		    let scl = ((i+2) as f64).log2().powi(2);
		    let smp_f64: f64 = (*smp).into();
		    smp_f64 * (volume_scale * scl as f64)
	    })
	    .collect::<Vec<f64>>();
	
	//*max = math::normalize_max_cplx(&mut data_f, 0.01, 0.7, *max, 0.0035);
	
	crate::audio::limiter(&mut data_f[..bar_num], 0.9, 10, 0.98);
	
	//let scale_factor = stream.normalize_factor_peak()*FFT_SIZE_RECIP*7.0;
    
    let bnf = 1.0 / bnf;
    
    LOCAL
    .iter_mut()
    .zip(data_f.iter())
    .take(bar_num)
    .enumerate()
    .for_each(|(i, (w, r))| {
		let i_ = i as f64 * bnf;
		let dynamic_smoothing = 0.175*dynamic_smooth2(i_, 3.0);
        *w = math::interpolate::multiplicative_fall(*w, *r, 0.0, dynamic_smoothing);
		
		// *w = math::interpolate::linearf(*w, *r, dynamic_smoothing+0.25);
	});
	
	stream.rotate_left(FFT_SIZE_HALF/2);
}

pub fn draw_bars(
	prog: &mut crate::data::Program, 
	stream: &mut crate::audio::SampleArr
) {
	use crate::math::{interpolate::smooth_step, fast::cubed_sqrt};

	let bar_num = prog.pix.width() / 2;
	let bnf = bar_num as f64;
	let bnf_recip = 1.0 / bnf;
	let l = stream.len();
	
	prepare(stream, bar_num, prog.VOL_SCL);
	
	let mut LOCAL = DATA.write().unwrap();

    prog.pix.clear();
    let sizef = Cplx::new(prog.pix.width() as f64, prog.pix.height() as f64);

    let bnfh = bnf * 0.5;

	let mut iter: f64 = 0.4;
	
	let mut prev_index = 0;
	let mut smoothed_smp = 0f64;
	let mut num_of_smp_each = 0usize;

    loop {
        let i_ = iter * bnf_recip;
        let idxf = iter;
        
        iter += i_;
        
        let idx = idxf as usize;
        
        let t = idxf.fract();
        
        smoothed_smp = smoothed_smp.max({
			num_of_smp_each += 1;
			cubed_sqrt(smooth_step(LOCAL[idx], LOCAL[(idx+1).min(bar_num)], t))
		});
		
		if prev_index == idx {continue}
		
		prev_index = idx;
		
		let idx = idx-1;

        let bar = smoothed_smp*sizef.y;
        num_of_smp_each = 0;
        
        let bar = (bar as usize).clamp(1, prog.pix.height());
        
        let fade = (128.0 + stream[idx*3/2].x*256.0) as u8;
        let peak = (bar*255/prog.pix.height()) as u8;
        let red = (fade/2).saturating_add(128).max(peak);

        prog.pix.draw_rect_wh(
			P2::new(
				(prog.pix.width()*idx/bar_num) as i32, 
				(prog.pix.height()-bar.min(prog.pix.height()-1)) as i32
			), 
			2, 
			bar, 
			u32::from_be_bytes([0xFF, 0xFF, (fade /4 *3).max(peak), 0])
		);
		
		smoothed_smp = 0.0;
		
		if idx+1 == bar_num {break}
    }
}

pub fn draw_bars_circle(
	prog: &mut crate::data::Program, 
	stream: &mut crate::audio::SampleArr
) {
	let size = prog.pix.height().min(prog.pix.width()) as i32;
	let sizef = size as f64;

	let bar_num = math::fast::isqrt(prog.pix.sizel());
	let bnf = bar_num as f64;
	let bnf_recip = 1.0 / bnf;
	let l = stream.len();
	
	let wh = prog.pix.width() as i32 / 2;
	let hh = prog.pix.height() as i32 / 2;
	
	prepare(stream, bar_num, prog.VOL_SCL);
	
	let mut LOCAL = DATA.write().unwrap();

    prog.pix.clear();

    //let base_angle = math::cos_sin(1.0 / bnf);
    let mut angle = Cplx::one();
    const fft_window: f64 = (FFT_SIZE >> 6) as f64 * 1.5;

    for i in 0..bar_num 
    {

        let i_ = i as f64 * bnf_recip;

        let idxf = i_*fft_window;
        let idx = idxf as usize;
        let t = idxf.fract();
        let i_next = i+1;

        angle = math::cos_sin(i_);
        
        // let scalef = math::fft_scale_up(i, bar_num);

        let bar = (math::interpolate::linearf(LOCAL[i], LOCAL[i_next], t)*sizef) as i32;
        let bar = bar*7/10;

		let p1 = P2::new(wh + (sizef*angle.x) as i32 / 2, hh + (sizef*angle.y) as i32 / 2);
		let p2 = P2::new(wh + ((size-bar) as f64 *angle.x) as i32 / 2,  hh + ((size-bar) as f64*angle.y) as i32 / 2);
		
		let i2 = i << 2;

		let pulse = (stream[i*3/2].x*32768.0) as u8;
		let peak = (bar*255/size).min(255) as u8;

		let r: u8 = 0;
		let g: u8 = peak.saturating_add(pulse >> 1);
		//let b: u8 = 
		let b: u8 = 0xFF;
		
        let c = u32::compose([0xFF, r, g, b]);

        prog.pix.draw_line(p1, p2, c);
    }
}
