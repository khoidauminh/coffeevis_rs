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

// This function literally visualizes the bit array of f32 samples.
pub fn draw_f32(pix: &mut [u32], stream: &[(f32, f32)], para: &mut Parameters) {
    const bits: usize = u32::BITS as usize;

    let bar_width: usize = (para.WIN_W / u32::BITS as usize).max(1) as usize;

    graphical_fn::win_clear(pix);

    let samplef = math::cplx_mag(stream[para._i & 0xFF]);
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
