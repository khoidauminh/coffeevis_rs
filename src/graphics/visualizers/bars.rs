use crate::constants::{VOL_SCL, FFT_SIZE, SMOOTHING, WIN_H, WIN_W, INCREMENT, pi2, pih};
use crate::graphics::graphical_fn;
use crate::math;

const color: [u32; 3] = [0x66ff66, 0xaaffff, 0xaaaaff];
const this_fft_size: usize = FFT_SIZE<<2;
const bar_num: usize = 48;
const bnf: f32 = bar_num as f32;

// [(index float, index usize, slider t float, scaling float, dynamic smoothing float); bar_num]
static mut idx_table: [(f32, usize, f32, f32, f32); bar_num] = [(0.0f32, 0usize, 0.0f32, 0.0f32, 0.0f32); bar_num];

pub unsafe fn prepare_index_bar() {
    
    for i in 0..bar_num {
        let i_ = i as f32 / bnf;

        let idxf = i_*i_*bnf*3.0;
        let idx = idxf as usize;
        let t = idxf.fract();
        
        // this parameter makes the bass region in the spectrum animaton look more aggresive 
        let d = math::interpolate::linearf(1.2, 0.5, (i as f32 / bnf).sqrt());
        
        idx_table[i] = (idxf, idx, t, (i_ + 1.414214).log2(), d);
    }
    
}

static mut _i : usize = 0;

static mut data_f1 : [f32; bar_num+1] = [0.0; bar_num+1];

pub unsafe fn draw_bars(buf: &mut Vec<u32>, stream: Vec<(f32, f32)>) {
    if stream.len() < FFT_SIZE { return (); }
    
    let scale = FFT_SIZE as f32 * WIN_H as f32 * 0.0625;
    let winhh = WIN_H >> 1;

    let wf = WIN_W as f32;
	let hf = WIN_H as f32;

	let smoothi = (SMOOTHING*255.9) as i32;
    
    let mut data_f = vec![(0.0f32, 0.0f32); this_fft_size];
    for i in 0..this_fft_size {
        data_f[i] = math::complex_mul(stream[(i+_i)%stream.len()], (VOL_SCL, 0.0));
    }
    
    //triangular(&mut data_f);

    //blackman_harris(&mut data_f);
    data_f = math::cooley_turky_fft_recursive(data_f);

    graphical_fn::win_clear(buf);
    
    for i in 0..bar_num {
        let idxf = idx_table[i].0;
        let idx = idx_table[i].1;
        let t = idx_table[i].2;
        let scaling = idx_table[i].3;
        let dynamic_smoothing = idx_table[i].4;
        
        data_f1[i] = math::interpolate::linearf(data_f1[i], (data_f[idx].0.powi(2) + data_f[idx].1.powi(2)).sqrt()*0.13*scaling, dynamic_smoothing);
        let bar = math::interpolate::linearf(data_f1[i], data_f1[i+1], t) as usize;

        graphical_fn::draw_rect(buf, 4*i, WIN_H-bar.min(WIN_H-1), 2, bar, color[0]);  
    }
    
    _i = (_i+INCREMENT) & 1023;
} 
