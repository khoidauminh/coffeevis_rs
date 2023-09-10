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
    (x*TAU).sin()
}

pub fn cos_norm(x: f32) -> f32 {
    (x*TAU).cos()
}

pub fn bit_reverse(n: usize, power: usize) -> usize {
	n.reverse_bits() >> usize::BITS.saturating_sub(power as u32) as usize
}

pub fn fsqrt(x: f32) -> f32 {
	const BIAS: u32 = 127 << 23;
    let mut xi = x.to_bits();
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

pub fn isqrt(val: usize) -> usize {
    if val < 2 {return val}

    let mut a = 2;
    let mut b;

    let mut z = a*a;
    while z < val {a = z; z *= a} // Blow up the low bound by tetration,
    
    while a*a < val {a *= 2} // then get a closer value by doubling.
    a /= 2;
    
    b = val / a; a = (a+b) /2; // I forgot what this does, it looks like 
    b = val / a; a = (a+b) /2; // a binary search.
    // b = val / a; a = (a+b) /2; // For extra accuracy, which is not needed
    
    return a;
}

pub fn flog2(x: f32) -> f32 {
    const error: f32 = 0.086071014*0.5;
    const mask: u32 = (1 << 23)-1;
    const ratio_recip: f32 = 1.0 / (mask+1) as f32;

    let xi = x.to_bits();

    let exp = (xi >> 23) as f32 - 128.0;
    let fract = ((xi & mask) as f32)*ratio_recip;

    exp + fract + error
}
