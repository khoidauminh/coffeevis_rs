use super::*;

// Now, before you make a judgement that these extreme optimizations 
// are a waste of time, this my personal audio visualizer, in which 
// accuracy is the least concern. The visualizer operates at low 
// resolution, meaning that finer details can be omitted. The program
// is intented to be as minimal in cpu usage as it can.
// 
// I get to go crazy with optimizations as long as they spit out 
// more performance.

#[inline]
fn to_bits(x: f32) -> u32 { x.to_bits() }

#[inline]
fn from_bits(x: u32) -> f32 { f32::from_bits(x) }

pub fn wrap(x: f32) -> f32 {
	x - x.round()
}

pub fn radian_wrap(x: f32) -> f32 {
	x % std::f32::consts::PI
}

#[inline]
pub fn abs(x: f32) -> f32 {
	from_bits(to_bits(x) & 0x7FFF_FFFF)
}

#[inline]
pub fn copysign(x: f32, sign: u32) -> f32 {
	from_bits(to_bits(x) | sign)
}

#[inline]
pub fn sign(x: f32) -> u32 {
	to_bits(x) & 0x80000000
}

// Turns out none of the attempts at outperforming 
// Rust's sin, cos worked. 
// Oh well.
//
// Std sin, cos are currently the default functions 
// Install with `--features wtf` or `--features approx_trig`
// to force using these functions.
mod wtf {
	use super::{abs, copysign, to_bits, sign};

	/// Agressively optimized sin
	pub fn sin_norm(x: f32) -> f32 {
		
		let xabs = abs(x);

		const linear_coef: f32 = 6.06;
		const quadra_coef: f32 = 18.36;
		
		if xabs < 0.0865 {
			return linear_coef*x;
		}

		let sign = sign(x);

		if xabs < 0.4135 {
			let x2 = xabs - 0.25;
			let y = 1.0 - quadra_coef*x2*x2;
			return copysign(y, sign);
		}

		let y = linear_coef*(0.5 - xabs);
		return copysign(y, sign);
	}

	/// Agressively optimized cos
	pub fn cos_norm(x: f32) -> f32 {
		/*let x = 0.25 - abs(x);
		x*(7.3 - 13.1125*abs(x))*/
		
		let x = abs(x);
		
		const linear_coef: f32 = 6.034;
		const quadra_coef: f32 = 18.676;
		
		const bound1: f32 = 0.167;
		const bound2: f32 = 0.3371;
		
		if x < bound1 {
			return 1.0 - quadra_coef*x*x;
		}
		
		if x < bound2 {
			return linear_coef*(0.25 - x);
		}
	
		let x2 = 0.5 - x;
		let y = quadra_coef*x2*x2 - 1.0;
		return y;
	}
}

mod fast_trig {
	use super::abs;
	/// Credits to:
	/// http://web.archive.org/web/20141220225551/http://forum.devmaster.net/t/fast-and-accurate-sine-cosine/9648
	/// This is a similar implementation that uses less floating point operations
	/// while retaining similar amount of accuracy.
	pub fn sin_norm(x: f32) -> f32 {
		let y = x*(0.5 - abs(x));
		y*(12.47 + 56.48*abs(y))
	}

	
	pub fn cos_norm(x: f32) -> f32 {
		sin_norm (0.25 - abs(x))
	}

	/// Reimplementation of fastapprox::faster::cos	
	pub fn other_cos_norm(x: f32) -> f32 {
		let x = 0.25 - abs(x);
		x*(6.191 - 35.34*x*x)
	}
}

mod std_trig {
	pub fn sin_norm(x: f32) -> f32 {
		(x*std::f32::consts::TAU).sin()
	}
	
	pub fn cos_norm(x: f32) -> f32 {
		(x*std::f32::consts::TAU).cos()
	}
}

pub fn sin_norm(x: f32) -> f32 {
	if cfg!(feature = "wtf") {
		wtf::sin_norm(x)
	} else if cfg!(feature = "aprrox_trig") {
		fast_trig::sin_norm(x)
	} else {
		std_trig::sin_norm(x)
	}
}

pub fn cos_norm(x: f32) -> f32 {
	if cfg!(feature = "wtf") {
		wtf::cos_norm(x)
	} else if cfg!(feature = "aprrox_trig") {
		fast_trig::cos_norm(x)
	} else {
		std_trig::cos_norm(x)
	}
}

pub fn bit_reverse(n: usize, power: usize) -> usize {
	n.reverse_bits() >> usize::BITS.saturating_sub(power as u32) as usize
}
/*
pub fn twiddle_norm(x: f32) -> Cplx<f32> {
	let xabs = x.abs();

	let cos_one8th = 1.0 - (4.329568*x).powi(2);
	let sin_one8th = fsqrt(1.0 - cos_one8th.powi(2)).copysign(x);

	if xabs > 0.375 {
		return Cplx::<f32>::new(-cos_one8th, -sin_one8th)
	}

	if xabs > 0.125 {
		return Cplx::<f32>::new(-sin_one8th, cos_one8th)
	}

	//let cos = (x*TAU).cos();
	//Cplx::new(cos, fsqrt(1.0-cos.powi(2).copysign(x)))



	Cplx::new(cos_one8th, sin_one8th)
}*/

pub fn cubed_sqrt(x: f32) -> f32 {
	x * x.sqrt()
}

pub fn unit_exp2_0(x: f32) -> f32 {
    x*(0.3431*x + 0.6568)
}

// Used in rage 0..=1
pub fn unit_exp2(x: f32) -> f32 {
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
    
    (val as f32).sqrt() as usize
}

pub fn flog2(x: f32) -> f32 {
    const error: f32 = 0.086071014*0.5;
    const mask: u32 = (1 << 23)-1;
    const ratio_recip: f32 = 1.0 / (mask+1) as f32;

    let xi = to_bits(x);

    let exp = (xi >> 23) as f32 - 128.0;
    let fract = ((xi & mask) as f32)*ratio_recip;

    exp + fract + error
}
