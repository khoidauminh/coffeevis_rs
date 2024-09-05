pub fn wrap(x: f64) -> f64 {
    x - x.round()
}

pub fn sin_norm(x: f64) -> f64 {
    let x = x * std::f64::consts::TAU;
    x.sin()
}

pub fn cos_norm(x: f64) -> f64 {
    let x = x * std::f64::consts::TAU;
    x.cos()
}

pub fn bit_reverse(n: usize, power: usize) -> usize {
    n.reverse_bits()
        .wrapping_shr(usize::BITS.wrapping_sub(power as u32))
}

pub fn cubed_sqrt(x: f64) -> f64 {
    x * x.sqrt()
}

pub fn unit_exp2_0(x: f64) -> f64 {
    x * (0.3431 * x + 0.6568)
}

// Used in rage 0..=1
pub fn unit_exp2(x: f64) -> f64 {
    unit_exp2_0(x) + 1.0
}

pub fn isqrt(val: usize) -> usize {
    (val as f64).sqrt() as usize
}

pub const fn ilog2(x: usize) -> usize {
    usize::BITS.wrapping_sub(x.wrapping_shr(1).leading_zeros()) as usize
}
