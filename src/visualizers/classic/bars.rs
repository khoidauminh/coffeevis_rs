use std::sync::RwLock;

use crate::data::{FFT_SIZE, INCREMENT, Program};
use crate::graphics::{P2, blend::Blend};
use crate::math::{self, Cplx, PIH, TAU};

const color: [u32; 3] = [0x66ff66, 0xaaffff, 0xaaaaff];

const FFT_SIZE_HALF: usize = FFT_SIZE / 2;

static DATA: RwLock<Vec<f32>> = RwLock::new(Vec::new());
static MAX: RwLock<f32> = RwLock::new(0.0);

const FFT_SIZE_RECIP: f32 = 1.0 / FFT_SIZE as f32;
const normalize_factor: f32 = FFT_SIZE_RECIP;
const BARS: usize = 48;

fn dynamic_smooth(t: f32, a: f32) -> f32 {
	((t - a) / a).powi(2)
}

fn dynamic_smooth2(t: f32, b: f32) -> f32 {
	(b*t + 1.0).recip()
}

fn prepare(stream: &mut crate::audio::SampleArr, bar_num: usize, volume_scale: f32) {
	let bar_num = bar_num +1;
	
	let bnf = bar_num as f32;
	let l = stream.len();
	
	let mut LOCAL = DATA.write().unwrap();

	if bar_num != LOCAL.len() {
		LOCAL.resize(bar_num, 0.0);
	}

    let mut data_f = [Cplx::<f32>::zero(); FFT_SIZE];
    data_f
    .iter_mut()
    .take(FFT_SIZE_HALF)
    .enumerate()
    .for_each(|(i, smp)| 
        *smp = stream[i*2].scale(FFT_SIZE_RECIP)
    );
    math::fft(&mut data_f);
    
    let mut max = MAX.write().unwrap();
    
    data_f
    .iter_mut()
    .take(bar_num)
    .enumerate()
    .for_each(|(i, smp)| {
		let log = math::fast::flog2((i+2) as f32);
		*smp = smp.scale(volume_scale * log as f32);
	});
	
	//*max = math::normalize_max_cplx(&mut data_f, 0.01, 0.7, *max, 0.0035);
	
	crate::audio::limiter(&mut data_f[..bar_num+5+10], 0.25, 5, 10, 10, 0.94);
	
	//let scale_factor = stream.normalize_factor_peak()*FFT_SIZE_RECIP*7.0;
    
    LOCAL
    .iter_mut()
    .zip(data_f.iter())
    .take(bar_num)
    .enumerate()
    .for_each(|(i, (w, r))| {
		let i_ = i as f32 / bnf;
		let dynamic_smoothing = 0.2*dynamic_smooth2(i_, 3.0);
		
        *w = math::interpolate::multiplicative_fall(*w, r.l1_norm(), 0.0, dynamic_smoothing);
		//*w = math::interpolate::linearf(*w, r.l1_norm(), dynamic_smoothing);
	});
	
	stream.rotate_left(FFT_SIZE_HALF/2);
}

pub const draw_bars: crate::VisFunc = |prog, stream| {
	use crate::math::{interpolate::bezierf, fast::cubed_sqrt, increment_index};
	
	let bar_num = prog.pix.width / 2;
	let bnf = bar_num as f32;
	let l = stream.len();
	
	prepare(stream, bar_num, prog.VOL_SCL);
	
	let mut LOCAL = DATA.write().unwrap();

    prog.clear_pix();
    let sizef = Cplx::<f32>::new(prog.pix.width as f32, prog.pix.height as f32);

    let bnfh = bnf * 0.5;

	let mut iter: f32 = 0.4;
	
	let mut prev_index = 0;
	let mut smoothed_smp = 0f32;

    loop {
        let i_ = iter / bnf;
        let idxf = iter;
        
        iter += i_;
        
        let idx = idxf as usize;
        
        let t = idxf.fract();
        
        smoothed_smp = smoothed_smp.max({
			cubed_sqrt(bezierf(LOCAL[idx], LOCAL[(idx+1).min(bar_num)], t))
		});
		
		if prev_index == idx {continue}
		
		prev_index = idx;
		
		let idx = idx-1;

        let bar = smoothed_smp*sizef.y;
        let bar = (bar as usize).clamp(1, prog.pix.height);
        
        let fade = (128.0 + stream[idx*3/2].x*32768.0) as u8;
        let peak = (bar*255/prog.pix.height) as u8;
        let red = (fade/2).saturating_add(128).max(peak);

        prog.pix.draw_rect_wh(
			P2::new(
				(prog.pix.width*idx/bar_num) as i32, 
				(prog.pix.height-bar.min(prog.pix.height-1)) as i32
			), 
			2, 
			bar, 
			u32::from_be_bytes([0xFF, 0xFF, (fade /4 *3).max(peak), 0])
		);
		
		smoothed_smp = 0.0;
		
		if idx+1 == bar_num {break}
    }
};

pub const draw_bars_circle: crate::VisFunc = |prog, stream| 
{
	let size = prog.pix.height.min(prog.pix.width) as i32;
	let sizef = size as f32;

	let bar_num = math::fast::isqrt(prog.pix.sizel());
	let bnf = bar_num as f32;
	let l = stream.len();
	
	let wh = prog.pix.width as i32 / 2;
	let hh = prog.pix.height as i32 / 2;
	
	prepare(stream, bar_num, prog.VOL_SCL);
	
	let mut LOCAL = DATA.write().unwrap();

    prog.clear_pix();

    let base_angle = math::cos_sin(1.0 / bnf);
    let mut angle = Cplx::<f32>::one();
    const fft_window: f32 = (FFT_SIZE >> 6) as f32 * 1.5;

    for i in 0..bar_num 
    {

        let i_ = i as f32 / bnf;

        let idxf = i_*fft_window;
        let idx = idxf as usize;
        let t = idxf.fract();
        let i_next = i+1;

        angle = angle*base_angle;
        
        // let scalef = math::fft_scale_up(i, bar_num);

        let bar = (math::fast::unit_exp2_0(math::interpolate::linearf(LOCAL[i], LOCAL[i_next], t))*sizef) as i32;
        let bar = bar*7/10;

		let p1 = P2::new(wh + (sizef*angle.x) as i32 / 2, hh + (sizef*angle.y) as i32 / 2);
		let p2 = P2::new(wh + ((size-bar) as f32 *angle.x) as i32 / 2,  hh + ((size-bar) as f32*angle.y) as i32 / 2);
		
		let i2 = i << 2;

		let pulse = ((stream[i*3/2].x*32768.0) as u8);
		let peak = (bar*255/size).min(255) as u8;

		let r: u8 = 0;
		let g: u8 = peak.saturating_add(pulse >> 1);
		//let b: u8 = 
		let b: u8 = 0xFF;
		
        let c = u32::compose([0xFF, r, g, b]);

        prog.pix.draw_line(p1, p2, c);
    }
};
