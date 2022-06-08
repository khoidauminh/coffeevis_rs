//use crate::constants::{WIN_H, WIN_W};
use crate::constants::{Parameters, INCREMENT};
use crate::graphics::graphical_fn::{self, draw_rect, win_clear};
use crate::math;

//static mut _i: usize = 0;
pub fn draw_exp1(pix: &mut [u32], stream: &[(f32, f32)], para: &mut Parameters) {
    let l = stream.len();
    // let pl = pix.len();
    // let ll = l-1;
    // let mut incr = para._i;
    
    //let scale: usiez = 2;

    for i in 0..pix.len() {
        para._i = math::advance_with_limit(para._i, l);
        let j = para._i >> 1; //incr * l / pix.len();
	    let jj = math::advance_with_limit(j, l);

        let sl = stream[j].0;
        let sr = stream[j].1;

	    let sa = sl-sr;
	    let ss = sl+sr;
	    let sd = ((stream[jj].0 + stream[jj].1) - ss)*128.0;
	
        let r = (ss.abs() * 255.0) as u32;
        let b = (sd.abs() * 255.0) as u32;
        //let g = (sa.abs() *2.0 * 255.0) as u32;
	    let g = r.saturating_add(b) as u32 / 3;

        pix[i] = (r << 16) | (g << 8) | b;
    }

    // para._i = (para._i + 2023) % stream.len();
}

const FT_SIZE: usize = 164;
const FREQ_RANGE: (f32, f32) = (6.0, 1926.0);

// this algorithm tries to approximate fft
fn approx_ft(inp: &[(f32, f32)], out: &mut [(f32, f32)], scaling_factor: usize) {
    let li = inp.len();
    let lif = li as f32;
    let lo = out.len();
    let lof = lo as f32;

    let ceiling_power_of_2 = math::log2i(li) + 1;
    let ln = scaling_factor << ceiling_power_of_2;

    for i in 0..ln {}
}

fn approx_ft2(
    inp: &[(f32, f32)],
    out: &mut [(f32, f32)],
    freq_start: f32,
    freq_end: f32,
    sample_rate: f32,
) {
    const LOWPAD_DEPTH: usize = 2;
    const LOWPAD_DEPTHL: usize = LOWPAD_DEPTH - 1;

    let l = inp.len();
    let lh = l / 2;
    let lf = l as f32;
    let lo = out.len();
    let lof = lo as f32;
    let lflof = lf / lof;

    let mut old_smp = (0.0f32, 0.0f32);
    let rc = (crate::constants::pi2 * sample_rate * 0.5 / lof).recip();
    let dt = sample_rate.recip();
    let a = dt / (rc + dt);

    let sr_nth = crate::constants::pi2 / sample_rate;

    let mut lowpad_array: [(f32, f32); LOWPAD_DEPTH] = [(0.0, 0.0); LOWPAD_DEPTH];
    let mut lowpad_old = (0.0f32, 0.0f32);

    let mut bin = 0;

    for smp in 0..l {
        let ang = math::interpolate::linearf(freq_start, freq_end, bin as f32 / lof) * sr_nth;

        lowpad_old = lowpad_array[LOWPAD_DEPTHL];

        for i in 0..LOWPAD_DEPTH.min(l - smp) {
            lowpad_array[i] = inp[smp + i];
        }
        lowpad_array[0] = math::lowpass(lowpad_old, lowpad_array[0], a.powi(LOWPAD_DEPTH as i32));
        for _ in 0..LOWPAD_DEPTH {
            for i in 1..LOWPAD_DEPTH {
                lowpad_array[i] = math::lowpass(lowpad_array[i - 1], lowpad_array[i], a);
            }
        }

        out[bin] = math::complex_add(
            out[bin],
            math::euler_wrap(lowpad_array[LOWPAD_DEPTHL], ang * smp as f32),
        );

        bin += 1;
        bin *= (bin < lo) as usize;
    }

    out.iter_mut().for_each(|mut bin| {
        bin.0 /= lflof;
        bin.1 /= lflof
    });
}

// This function literally visualizes the bit array of f32 samples.
pub fn draw_f32(pix: &mut [u32], stream: &[(f32, f32)], para: &mut Parameters) {
    const bits: usize = u32::BITS as usize;

    let bar_width: usize = (para.WIN_W / u32::BITS as usize).max(1) as usize;

    graphical_fn::win_clear(pix);

    let samplef = math::complex_mag(stream[para._i & 0xFF]);
    let sample = samplef.to_bits();

    for bit in 0..bits {
        let bitsample = (sample >> (bits - bit)) & 1;
        let x = bit * para.WIN_W / bits;

        let red = ((samplef + 1.0) * 127.999) as u8;
        let green = (bit * 255 / bits) as u8;
        let blue = (255 - green) as u8;

        if bitsample != 0 {
            graphical_fn::draw_rect(
                pix,
                x,
                0,
                bar_width,
                para.WIN_H,
                graphical_fn::rgb_to_u32(red, green, blue),
                para,
            );
        }
    }

    para._i = (para._i + INCREMENT) % stream.len();
}
// same of above but for u32
pub fn draw_u16(pix: &mut [u32], stream: &[(f32, f32)], para: &mut Parameters) {
    const bits: usize = u16::BITS as usize;

    let bar_width: usize = (para.WIN_W / bits).max(1) as usize;

    graphical_fn::win_clear(pix);

    let sample = ((stream[para._i & 0xFF].0 + 1.0) * (1 << bits - 1) as f32) as u16;

    for bit in 0..bits {
        let bitsample = (sample >> (bits - bit)) & 1;
        let x = bit * para.WIN_W / bits;

        let red = (sample >> 8) as u8;
        let green = (bit * 255 / bits) as u8;
        let blue = (255 - green) as u8;

        if bitsample != 0 {
            graphical_fn::draw_rect(
                pix,
                x,
                0,
                bar_width,
                para.WIN_H,
                graphical_fn::rgb_to_u32(red, green, blue),
                para,
            );
        }
    }

    para._i = (para._i + INCREMENT) % stream.len();
}

