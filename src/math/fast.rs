use super::*;

pub fn wrap(x: f32) -> f32 {
    if x < 0.5 && x >= -0.5 { return x }
	x - x.round()
}

pub fn radian_wrap(x: f32) -> f32 {
	let x = x*TAU_RECIP;
	wrap(x)*TAU
}

pub fn sin_norm(x: f32) -> f32 {
	let y = x*(8.0-16.0*x.abs());
    y*(0.7703 + 0.229703*y.abs())
}

pub fn cos_norm(x: f32) -> f32 {
	sin_norm(x + 0.25)
}

pub fn bit_reverse(n: usize, power: usize) -> usize {
	n.reverse_bits() >> usize::BITS.saturating_sub(power as u32) as usize
}

pub fn fsqrt(x: f32) -> f32 {
	const BIAS: u32 = 127 << 23;
    let mut xi = x.to_bits() & 0x7F_FF_FF_FF; // discarding the sign, allowing x to be negative
    xi = (xi+BIAS) >> 1;
    f32::from_bits(xi)
}

pub fn cubed_sqrt(x: f32) -> f32 {
	x * fsqrt(x)
}

pub fn unit_exp2_0(x: f32) -> f32 {
    x*(0.3431*x + 0.6568)
}

// Used in rage 0..=1
pub fn unit_exp2(x: f32) -> f32 {
    unit_exp2_0(x) + 1.0
}

pub fn isqrt(x: usize) -> usize {
    if (x < 2) {
        return x;
    }

    let small_cand = isqrt(x >> 2) << 1;
    let large_cand = small_cand + 1;
    if large_cand.pow(2) > x {
        return small_cand;
    }
    return large_cand

    /*
    let low = 1;
    let high = x >> 1;

    while (low+1 < high) {
        let mid = (low+high) / 2;

    }*/
}

pub fn flog2(x: f32) -> f32 {
   /* let mut xi = x.to_bits() & 0x7F_FF_FF_FF;
    let log2 = ((xi >> 23) as i64 - 128) as f32;

    xi &= !(255 << 23);
    xi += BIAS;

    let xi = f32::from_bits(xi);

    log2 + (-0.34484843*xi+2.02466578)*xi -0.67487759 */

    const mask: u32 = (1 << 23)-1;
    const ratio_recip: f32 = 1.0 / (mask+1) as f32;

    let xi = x.to_bits();


    let exp = (xi >> 23) as f32 - 128.0;
    let fract = ((xi & mask) as f32)*ratio_recip;

    exp + fract
}
