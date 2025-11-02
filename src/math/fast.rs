pub fn wrap(x: f32) -> f32 {
    x - x.round()
}

pub fn sin_norm(x: f32) -> f32 {
    let x = x * std::f32::consts::TAU;
    x.sin()
}

pub fn cos_norm(x: f32) -> f32 {
    let x = x * std::f32::consts::TAU;
    x.cos()
}

pub fn bit_reverse(n: usize, power: u32) -> usize {
    n.reverse_bits()
        .wrapping_shr(usize::BITS.wrapping_sub(power))
}

pub fn cubed_sqrt(x: f32) -> f32 {
    x * x.sqrt()
}

pub fn unit_exp2_0(x: f32) -> f32 {
    x * (0.3431 * x + 0.6568)
}

pub const fn ilog2(x: usize) -> u32 {
    usize::BITS.wrapping_sub(x.wrapping_shr(1).leading_zeros())
}
