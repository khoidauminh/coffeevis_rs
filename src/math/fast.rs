// Now, before you make a judgement that these extreme optimizations 
// are a waste of time, this my personal audio visualizer, in which 
// accuracy is the least concern. The visualizer operates at low 
// resolution, meaning that finer details can be omitted. The program
// is intented to be as minimal in cpu usage as it can.
// 
// I get to go crazy with optimizations as long as they spit out 
// more performance.

#[inline]
fn to_bits(x: f64) -> u64 { x.to_bits() }

#[inline]
fn from_bits(x: u64) -> f64 { f64::from_bits(x) }

pub fn wrap(x: f64) -> f64 {
	x - x.round()
}

pub fn radian_wrap(x: f64) -> f64 {
	x % std::f64::consts::PI
}

#[inline]
pub fn abs(x: f64) -> f64 {
	from_bits(to_bits(x) & 0x7FFF_FFFF_FFFF_FFFF)
}

#[inline]
pub fn copysign(x: f64, sign: u64) -> f64 {
	from_bits(to_bits(x) | sign)
}

#[inline]
pub fn sign(x: f64) -> u64 {
	to_bits(x) & 0x8000_0000_0000_0000
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
	n.reverse_bits() >> usize::BITS.saturating_sub(power as u32) as usize
}
/*
pub fn twiddle_norm(x: f64) -> Cplx {
	let xabs = x.abs();

	let cos_one8th = 1.0 - (4.329568*x).powi(2);
	let sin_one8th = fsqrt(1.0 - cos_one8th.powi(2)).copysign(x);

	if xabs > 0.375 {
		return Cplx::new(-cos_one8th, -sin_one8th)
	}

	if xabs > 0.125 {
		return Cplx::new(-sin_one8th, cos_one8th)
	}

	//let cos = (x*TAU).cos();
	//Cplx::new(cos, fsqrt(1.0-cos.powi(2).copysign(x)))



	Cplx::new(cos_one8th, sin_one8th)
}*/

pub fn cubed_sqrt(x: f64) -> f64 {
	x * x.sqrt()
}

pub fn unit_exp2_0(x: f64) -> f64 {
    x*(0.3431*x + 0.6568)
}

// Used in rage 0..=1
pub fn unit_exp2(x: f64) -> f64 {
    unit_exp2_0(x) + 1.0
}

pub fn isqrt(val: usize) -> usize {
    /*if val < 2 {return val}

    let mut a = 2;
    let mut b;

    let mut z = a*a;
    while z < val {a = z; z *= a} // Blow up the low bound by tetration,
    
    while a*a < val {a *= 2} // then get a closer value by doubling.
    a /= 2;
    
    b = val / a; a = (a+b) /2; // I forgot what this does, it looks like 
    b = val / a; a = (a+b) /2; // a binary search.
    // b = val / a; a = (a+b) /2; // For extra accuracy, which is not needed
    
    return a;*/
    
    (val as f64).sqrt() as usize
}
