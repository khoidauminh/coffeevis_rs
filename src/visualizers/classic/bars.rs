use std::sync::RwLock;

use crate::data::{POWER, FFT_SIZE, INCREMENT, Program};
use crate::graphics::{P2, blend::Blend};
use crate::math::{self, Cplx, PIH, TAU};

const color: [u32; 3] = [0x66ff66, 0xaaffff, 0xaaaaff];

static DATA: RwLock<Vec<f32>> = RwLock::new(Vec::new());
static MAX: RwLock<f32> = RwLock::new(0.0);

const FFT_SIZE_RECIP: f32 = 1.0 / FFT_SIZE as f32;
const normalize_factor: f32 = FFT_SIZE_RECIP * 2.1;
const BARS: usize = 48;

fn dynamic_smooth(t: f32, a: f32) -> f32 {
	((t - a) / a).powi(2)
}

fn dynamic_smooth2(t: f32, b: f32) -> f32 {
	(b*t + 1.0).recip()
}

fn prepare(stream: &mut crate::audio::SampleArr, bar_num: usize, volume_scale: f32) {
	let bnf = bar_num as f32;
	let l = stream.len();
	
	let mut LOCAL = DATA.write().unwrap();

	if bar_num != LOCAL.len() {
		LOCAL.resize(bar_num, 0.0);
	}

    let mut data_f = [Cplx::<f32>::zero(); FFT_SIZE];
    data_f
    .iter_mut()
    .enumerate()
    .for_each(|(i, smp)| 
        *smp = stream[i*2].scale(normalize_factor)
    );
    math::fft(&mut data_f);
    
    let mut max = MAX.write().unwrap();
    
    data_f
    .iter_mut()
    .take(bar_num)
    .enumerate()
    .for_each(|(i, smp)| {
		let log = math::fast::fsqrt(i as f32);
		*smp = smp.scale(volume_scale * log as f32);
	});
	
	*max = math::normalize_max_cplx(&mut data_f[..bar_num], 0.01, 0.7, *max, 0.0035);
    
    LOCAL
    .iter_mut()
    .zip(data_f.iter())
    .enumerate()
    .for_each(|(i, (w, r))| {
		let i_ = i as f32 / bnf;
		let dynamic_smoothing = 0.2*dynamic_smooth2(i_, 3.0);
		
        *w = math::interpolate::multiplicative_fall(*w, r.l1_norm(), 0.0, dynamic_smoothing);
		//*w = math::interpolate::linearf(*w, r.l1_norm(), dynamic_smoothing);
	});
	
	stream.rotate_left(FFT_SIZE / 2);
}

pub const draw_bars: crate::VisFunc = |prog, stream| {
    const divider: usize = 2;

	let bar_num = prog.pix.width / divider;
	let bnf = bar_num as f32;
	let l = stream.len();
	
	prepare(stream, bar_num, prog.VOL_SCL);
	
	let mut LOCAL = DATA.write().unwrap();

    prog.clear_pix();

    let mut iter: f32 = 0.4;

    let sizef = Cplx::<f32>::new(prog.pix.width as f32, prog.pix.height as f32);

    let bnfh = bnf * 0.5;

    while iter < bnf {
        let i = iter as usize;

        let i_ = iter / bnf;

        let idxf = i_*bnf;
        let idx = idxf as usize;
        let t = idxf.fract();
        let idx_next = if idx+1 == bar_num {idx} else {idx+1};

        // this progmeter makes the bass region in the spectrum animaton look more aggresive

        //~ let scaling = (i_+ 1.0).log2() * (prog.PIX_R << 2) as f32;
        // let scaling = math::fast_flog2(iter) * prog.VOL_SCL * sizef.i.powi(2);
        // let scaling = (2f32).powf(math::fast_fsqrt(iter)-1.0) * prog.VOL_SCL * sizef.i * 13.0;

        let bar = math::fast::cubed_sqrt(math::interpolate::bezierf(LOCAL[idx], LOCAL[idx_next], t))*sizef.y;
        let bar = (bar as usize).clamp(1, prog.pix.height);
        
        let fade = (128.0 + stream[i*3/2].x*32768.0) as u8;
        let peak = (bar*255/prog.pix.height) as u8;
        let red = (fade/2).saturating_add(128).max(peak);

        prog.pix.draw_rect_wh(
			P2::new(
				(divider*i) as i32, 
				(prog.pix.height-bar.min(prog.pix.height-1)) as i32
			), 
			2, 
			bar, 
			u32::from_be_bytes([0xFF, 0xFF, (fade /4 *3).max(peak), 0])
		);

        iter += i_;
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
        let i_next = if i+1 == bar_num {i} else {i+1};

        angle = angle*base_angle;

        let bar = (math::fast::unit_exp2_0(math::interpolate::linearf(LOCAL[i], LOCAL[i_next], t))*sizef) as i32;
        let bar = bar.min(size*7/10);

		let p1 = P2::new(wh + (sizef*angle.x) as i32 / 2, hh + (sizef*angle.y) as i32 / 2);
		let p2 = P2::new(wh + ((size-bar) as f32 *angle.x) as i32 / 2,  hh + ((size-bar) as f32*angle.y) as i32 / 2);
		
		let i2 = (i << 1) & 0xFF;
		
        let c = (0xFF << 24) | ((bar*255/size).min(255) << 8) as u32 | (((stream[i2].x+stream[i2].y)*64.0+192.0) as u32).min(255);

        prog.pix.draw_line(p1, p2, c);
    }
};
