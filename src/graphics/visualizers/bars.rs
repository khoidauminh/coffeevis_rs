use crate::config::{FFT_SIZE, INCREMENT, pi2, pih, pif32, POWER};
use crate::config::Parameters;
use crate::graphics::{graphical_fn, graphical_fn::P2};
use crate::math;
use math::cplx_0;

const color: [u32; 3] = [0x66ff66, 0xaaffff, 0xaaaaff];
pub const this_power: usize = POWER+2;
pub const this_fft_size: usize = 1 << this_power;

// [(index float, index usize, slider t float, scaling float, dynamic para.SMOOTHING float); bar_num]
// static mut idx_table: [(f32, usize, f32, f32, f32); bar_num] = [(0.0f32, 0usize, 0.0f32, 0.0f32, 0.0f32); bar_num];

//~ pub unsafe fn prepare_index_bar() {

    //~ for i in 0..bar_num {
        //~ let i_ = i as f32 / bnf;

        //~ let idxf = i_.powf(1.414)*bnf*2.0;
        //~ let idx = idxf as usize;
        //~ let t = idxf.fract();

        //~ // this parameter makes the bass region in the spectrum animaton look more aggresive
        //~ let d = math::interpolate::linearf(1.2, 0.5, (i as f32 / bnf).sqrt());

        //~ idx_table[i] = (idxf, idx, t, (i_ + 1.189).log2()*para.VOL_SCL, d);
    //~ }

//~ }

//static mut _i : usize = 0;

//pub static mut para.bars_smoothing_ft : Vec<f32> = Vec::new();

pub fn draw_bars(buf: &mut [u32], stream: &[(f32, f32)], para: &mut Parameters ) {
    const divider: usize = 3;

	let bar_num = para.WIN_W / divider;
	let bnf = bar_num as f32;
	let l = stream.len();

	if bar_num+1 != para.bars_smoothing_ft.len() {
		para.bars_smoothing_ft.resize(bar_num+1, 0.0);
	}

    let mut data_f = [(0.0f32, 0.0f32); this_fft_size];
    let mut _i = para._i;
	for i in 0..l {
		data_f[i] = stream[_i];
		_i = math::advance_with_limit(_i, l);
	}

    math::fft_inplace(&mut data_f, this_power);

    graphical_fn::win_clear(buf);

    for i in 0..bar_num {
        let i_ = i as f32 / bnf;

        let idxf = i_.powf(1.414)*bnf*2.0;
        let idx = idxf as usize;
        let t = idxf.fract();

        // this parameter makes the bass region in the spectrum animaton look more aggresive
        let dynamic_smoothing = math::interpolate::linearf(1.2, 0.5, (i as f32 / bnf).sqrt());

        let scaling = math::fast_flog2(i_+ 1.0) as f32 * (para.WIN_R << 2) as f32;

        let val = (math::cplx_mag(data_f[idx]) / this_fft_size as f32).powi(2)*scaling*para.VOL_SCL*2.0;

        para.bars_smoothing_ft[i] = math::interpolate::linearf(para.bars_smoothing_ft[i], val, dynamic_smoothing);
        let bar = (math::interpolate::linearf(para.bars_smoothing_ft[i], para.bars_smoothing_ft[i+1], t) as usize).min(para.WIN_H);

        graphical_fn::draw_rect(buf, divider*i, para.WIN_H-bar.min(para.WIN_H-1), 2, bar, (255 << 16) | (((bar*255/para.WIN_H) << 8) as u32), para);
    }

    para._i = (para._i+INCREMENT) & 1023;
}


pub fn draw_bars_circle(buf: &mut [u32], stream: &[(f32, f32)], para: &mut Parameters ) {
	let size = para.WIN_H.min(para.WIN_W) as i32;
	let sizef = size as f32;

	let bar_num = math::fast_isqrt(para.WIN_R);
	let bnf = bar_num as f32;
	let l = stream.len();

	if bar_num+1 != para.bars_smoothing_ft.len() {
		para.bars_smoothing_ft_circle.resize(bar_num+1, 0.0);
	}

    let wh = para.WIN_W as i32 / 2;
    let hh = para.WIN_H as i32 / 2;

    let mut data_f = [(0.0f32, 0.0f32); this_fft_size];

    let mut _i = para._i;
	for i in 0..l {
		data_f[i] = stream[_i];
		_i = math::advance_with_limit(_i, l);
	}

    math::fft_inplace(&mut data_f, this_power);

    graphical_fn::win_clear(buf);

    let base_angle = math::euler((1.0, 0.0), pi2 / bnf);
    let mut angle = (1.0, 0.0);

    for i in 0..bar_num {

        let i_ = i as f32 / bnf;

        let idxf = i_*(this_fft_size >> 6) as f32;
        let idx = idxf as usize;
        let t = idxf.fract();

        angle = math::cplx_mul(angle, base_angle);

        // this parameter makes the bass region in the spectrum animaton look more aggresive
        let dynamic_smoothing = math::interpolate::linearf(1.2, 0.5, math::fast_fsqrt(i as f32 / bnf));

        let scaling = math::fast_flog2(i_ + 1.1) * 0.1;

        let val = (math::cplx_mag(data_f[idx]) *scaling).powi(2) *para.VOL_SCL;

        para.bars_smoothing_ft_circle[i] = math::interpolate::linearf(para.bars_smoothing_ft_circle[i], val, dynamic_smoothing);
        let bar = (math::interpolate::linearf(para.bars_smoothing_ft_circle[i], para.bars_smoothing_ft_circle[i+1], t) as i32).min(size*7/10);

		let p1 = P2(wh + (sizef*angle.0) as i32 / 2, hh + (sizef*angle.1) as i32 / 2);
		let p2 = P2(wh + ((size-bar) as f32 *angle.0) as i32 / 2,  hh + ((size-bar) as f32*angle.1) as i32 / 2);
        let c = ((bar*255/size).min(255) << 8) as u32 | (((stream[i].0+stream[i].1)*64.0+192.0) as u32).min(255);

        graphical_fn::draw_line_direct(buf, p1, p2, c , para);
    }

    para._i = (para._i+INCREMENT) & 1023;
}
