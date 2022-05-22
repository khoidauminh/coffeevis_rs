//use crate::constants::{WIN_H, WIN_W};
use crate::constants::Parameters;
use crate::math;
use crate::graphics::graphical_fn::{win_clear, draw_rect};

//static mut _i: usize = 0;
pub fn draw_exp1(pix: &mut [u32], stream: &[(f32, f32)], para: &mut Parameters ) {
    
    let l = stream.len();
    let mut incr = para._i;
    
    for i in 0..pix.len() {
        incr = math::advance_with_limit(incr, l);
        let j = (incr*l/pix.len()).min(l-1);
        
        let sl = stream[j].0;
        let sr = stream[j].1;
        
        let r = (sl.abs()*255.0) as u32;
        let b = (sr.abs()*255.0) as u32;
        let g = r.saturating_add(b)/2;

        pix[i] = (r << 16) | (g << 8) | b;

    }
    
    para._i = (para._i+2023) % stream.len();
}

const FT_SIZE: usize = 164;
const FREQ_RANGE: (f32, f32) = (6.0, 1926.0);

// this algorithm tries to approximate fft
fn approx_ft(inp: &[(f32, f32)], out: &mut[(f32, f32)], scaling_factor: usize) {
    let li = inp.len();
    let lif = li as f32;
    let lo = out.len();
    let lof = lo as f32;  
  
	let ceiling_power_of_2 = math::log2i(li)+1;
	let ln = scaling_factor << ceiling_power_of_2;
	
	for i in 0..ln {
		
	}
}

fn approx_ft2(inp: &[(f32, f32)], out: &mut[(f32, f32)], freq_start: f32, freq_end: f32, sample_rate: f32) {
	const LOWPAD_DEPTH: usize = 2;
	const LOWPAD_DEPTHL: usize = LOWPAD_DEPTH-1; 
	
    let l = inp.len();
    let lh = l/2; 
    let lf = l as f32;
    let lo = out.len();
    let lof = lo as f32;  
    let lflof = lf / lof;
  
    let mut old_smp = (0.0f32, 0.0f32);
    let rc = (crate::constants::pi2* sample_rate*0.5 / lof).recip();
    let dt = sample_rate.recip();
    let a = dt / (rc + dt);
    
    let sr_nth = crate::constants::pi2 / sample_rate;
   
    let mut lowpad_array: [(f32, f32); LOWPAD_DEPTH] = [(0.0, 0.0); LOWPAD_DEPTH];
    let mut lowpad_old = (0.0f32, 0.0f32);
    
    let mut bin = 0;
    
    for smp in 0..l {
        let ang = math::interpolate::linearf(freq_start, freq_end, bin as f32 / lof) * sr_nth;
        
        lowpad_old = lowpad_array[LOWPAD_DEPTHL];
        
        for i in 0..LOWPAD_DEPTH.min(l-smp) {
            lowpad_array[i] = inp[smp+i];
        }
        lowpad_array[0] = math::lowpass(lowpad_old, lowpad_array[0], a.powi(LOWPAD_DEPTH as i32));
	    for _ in 0..LOWPAD_DEPTH {
	        for i in 1..LOWPAD_DEPTH {
				lowpad_array[i] = math::lowpass(lowpad_array[i-1], lowpad_array[i], a);	
			}
		}
        
        out[bin] = math::complex_add(out[bin], math::euler_wrap(lowpad_array[LOWPAD_DEPTHL], ang *smp as f32));
        
        bin += 1;
        bin *= (bin < lo) as usize;
    }
    
    out.iter_mut().for_each(|mut bin| {bin.0 /= lflof; bin.1 /= lflof});
}

/*
// above implementation but fuller.
fn approximated_ft(inp: &[(f32, f32)], out: &mut[(f32, f32)], freq_start: f32, freq_end: f32, sample_rate: f32) {
    let li = inp.len();
    let lo = out.len();
    let lf = out.len();
    
    let lif = li as f32;
    let lof = lo as f32;
    
    // implementation for discrete frequency bins
    
    // preparation
    // this sub-algorithm will determine how much sample each bin will get based
    // on it's specified frequency. Higher frequency bins usually receive more 
    // samples.  
    /*let sum = out.1.sum();
    let freq = vec![(0usize, 0.0); lo];
    for i in 0..lo-1 {
        freq[i].1 = out.1[i] / 44100.0 * crate::constants::pi2 / 8.0;
        let b_temp = (lof*out.1[i]/sum) as usize;
        freq[i].0 = b_temp + (b_temp == 0) as usize;
    }
    freq.last_mut().0 = sum - freq.sum();
    
    let mut bin = 0;
    let mut smp = 0;
    
    // modifed fourier transform
    // each bin will receive a block of adjacent samples with determined length. 
    // The formula to determine the amount of samples is given to a bin is: 
    // number_of_bins * frequency_of_bin / sum_of_all frequencies
    
    // let N = 4;
    // Suppose we need to calculate N bins with freqs of : 1 4 18 23
    // The sum of those freqs are 46
    // each frequency will receive this many samples on a block of N samples:
    // bin 0 freq 1: 1 (since 4/46 < 0 so we give it a 1)
    // bin 1 freq 4: 1 
    // bin 2 freq 18: 1
    // bin 3 freq 7: 2 (note that this bin is acutally only given 1, as we are 
    // confined to N. This can be allowed).
    
    // sample   : 0  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 
    // bin index: 0  1  2  3  0  1  2  3  0  1  2  3  0  1  2  3  0  
    while smp < li {
        let cbn = freq[bin].0;
        let f = freq[bin].1;
        
        let lp = cbn.min(li-smp);
        for _ in 0..lp {
            out[bin] = complex_add(out[f], euler_wrap(inp.0[smp], smp as f32 * f));
        }
        
        bin += 1;
        bin *= ((bin < lo) as usize);
        
        smp += lp;
    }*/
    
    
    // implementation for discrete frequency ranges.
    let sum = 0.5*lof*(freq_start+freq_end);
    let liflof = lif/lof;
    
    for i in 0..lo {
        let freq = math::interpolate::linearf(freq_start, freq_end, i as f32 / lof);
        let angle = freq * crate::constants::pi2 / (sample_rate);
        
        let mut num = (lof*freq/sum) as usize;
                num += (num == 0) as usize;
        
        let mut smp = 0;
        
        let rest = lo - num;
        while smp < li {
            for _ in 0..num.min(li-smp) {
                out[i] = math::complex_add(out[i], math::euler_wrap(inp[smp], angle *smp as f32));
                smp += 1;
            }
            
            //inp[smp..
            
            smp += rest;
        }
        
        out[i] = math::complex_mul(out[i], (num as f32 / liflof, 0.0));
    }
}*/

//~ static mut smooth_ft: [f32; FT_SIZE] = [0.0; FT_SIZE];

//~ pub unsafe fn draw_approx_ft(pix: &mut [u32], stream: &[(f32, f32)]) {
    //~ let mut out = vec![(0.0, 0.0); FT_SIZE];
    //~ //math::dft(stream, &mut out, 1024.0);
    
    //~ //approx_ft(stream, &mut out, FREQ_RANGE.0, FREQ_RANGE.1, crate::constants::SAMPLE_RATE as f32);
    
    //~ win_clear(pix);
    
    //~ out.iter().enumerate().for_each(|(idx, bin)| {
        //~ let y = idx*WIN_H/FT_SIZE;
        //~ smooth_ft[idx] = math::interpolate::linearf(smooth_ft[idx], (
            //~ //(3f32).powf(1.0+bin.0.abs()*0.1)
            //~ //(bin.0.abs()*0.6).powi(2)
            //~ (WIN_W as f32 *math::complex_mag(*bin)).powi(2)
            //~ *(1.1 + idx as f32 / FT_SIZE as f32).log2()), 0.5);
        //~ let x = smooth_ft[idx] as usize;
        
        //~ let g = (smooth_ft[idx] as u32).min(255);
        
        //~ draw_rect(pix, 0, y, x, 2,  (255 << 16) | (g << 8));
    //~ });
    //~ //println!("{}", smooth_ft[0]);
//~ }
