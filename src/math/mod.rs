mod cplx;
pub mod fast;
mod fft;
pub mod rng;

use std::ops;

pub const PI: f32 = std::f32::consts::PI;
pub const TAU: f32 = PI*2.0;
pub const PIH: f32 = PI*0.5;
pub const TAU_RECIP: f32 = 1.0 / TAU;

#[derive(Copy, Clone)]
pub struct Cplx<T: Copy + Clone> {
	pub x: T,
	pub y: T
}

pub trait ToUsize<T> {
    fn new(value: T) -> Self;
}

impl ToUsize<i32> for usize {
	fn new(x: i32) -> usize {
		if x > 0 {return x as usize}
		0
		
		//x.max(0) as usize
		//const MINUS1: usize = -1i32 as usize;
		//(x as usize) & ((x < 0) as usize).wrapping_add(MINUS1)
	}
}

pub fn fft(a: &mut [Cplx<f32>]) {
	let l = a.len();
	let power = log2i::<usize>(l);

	fft::butterfly(a, power);
	fft::compute_fft_iterative(a);
}

pub fn fft_half(a: &mut [Cplx<f32>]) {
	let l = a.len();
	let power = log2i::<usize>(l);

	fft::butterfly_half(a, power);
	fft::compute_fft_half(a);
}

pub const fn log2i<T>(n: usize) -> usize {
	usize::BITS.saturating_sub(n.leading_zeros()).wrapping_sub(1) as usize
}

pub fn increment<T>(a: T, limit: T) -> T 
where T: ops::Add<Output = T> + std::cmp::PartialOrd + From<u8>
{
    let b = a + T::from(1);
    if b < limit {return b}
    T::from(0)
}

pub fn decrement<T>(a: T, limit: T) -> T
    where T: From<usize> + ops::Sub<Output = T> + std::cmp::PartialEq
{
    if a == T::from(0) { return limit-1.into() }
    a - 1.into()
}

pub fn squish(x: f32, scale: f32, limit: f32) -> f32 {
	// (-192.0*x.max(0.0).recip()).exp()
	// const scale: f32 = 621.0;
	(-scale*(x.abs()+scale).recip()+1.0)*limit*x.signum()
}

pub fn inverse_factorial(i: usize) -> usize {
	let mut o = 1;
	let mut j = 1;

	if i < 2 {return 1}

	while j < i {
		o += 1;
		j *= o;
	}

	o
}
/*
pub fn derivative<T>(a: &mut [Cplx<T>], amount: f32)
where T: std::marker::Copy + ops::Sub<Output = T>
{
	for i in 1..a.len()
	{
		a[i-1] = a[i] - a[i-1].scale(amount)
	}
}*/

pub fn integrate_inplace(a: &mut [Cplx<f32>], factor: usize, norm: bool)
{
    if factor < 2 { return; }

    let mut sum = Cplx::<f32>::zero();
    let mut table = vec![Cplx::<f32>::zero(); factor];
    let mut fi = 0;
    let mut si = 0;

    let l = a.len();

    let first_iter = factor.min(l);
    while si < first_iter {
        table[fi] = a[si];

        sum = sum + a[si];
        a[si] = sum;

        fi += 1;
        si += 1;
    }

    fi = 0;

    let bound = l.saturating_sub(factor);
    while si < bound {
        sum = sum - table[fi];
        sum = sum + a[si];
        table[fi] = a[si];

        a[si] = sum;

        fi = increment(fi, factor);
        si += 1;
    }

    while si < l {
        sum = sum - table[fi];
        a[si] = sum;

        si += 1;
        fi = increment(fi, factor);
    }

    if norm {
        let div = 1.0 / factor as f32;
        // a.iter_mut().for_each(|x| *x = *x * Cplx::new((factor as f32).recip(), 0.0));
        let mut i = 0;
        while i < first_iter {a[i] = a[i].scale(1.0 / i as f32); i+=1};
        while i < bound {a[i] = a[i].scale(div); i+=1};
        while i < l {a[i] = a[i].scale(1.0 / (l-i) as f32); i+=1};
    }
}

pub fn normalize_max_cplx(a: &mut [Cplx<f32>], limit: f32, threshold: f32, prev_max: f32, smooth_factor: f32) -> f32 {
    let mut max = limit;
    for i in a.iter() {
        max = max.max(i.x.abs());
        max = max.max(i.y.abs());
    }

    // if max >= threshold {return}

    let max = f32::min(max, threshold);

    let max = interpolate::subtractive_fall(prev_max, max, limit, smooth_factor);

    let amp = threshold / max;

    for i in a.iter_mut() {
        *i = i.scale(amp);
    }

    max
}

pub fn normalize_max_f32(a: &mut [f32], limit: f32, threshold: f32, prev_max: f32, smooth_factor: f32) -> f32
{
    let mut max = limit;
    for i in a.iter() {max = max.max(*i)}

    // if max >= threshold {return}

    let max = f32::min(max, threshold);

    let max = interpolate::subtractive_fall(prev_max, max, limit, smooth_factor);

    let amp = max.recip();

    for i in a.iter_mut() {*i *= amp}

    max
}

pub fn normalize_average(a: &mut [Cplx<f32>], limit: f32, prev_ave: f32, smooth_factor: f32) -> f32 {
	let mut ave = limit;
	for i in a.iter() {
		ave = (ave + i.x.abs());
		ave = (ave + i.y.abs());
	}

	let ave = interpolate::subtractive_fall(prev_ave, ave, limit, smooth_factor);

	let amp = ave.recip();

    for i in a.iter_mut() {
        *i = i.scale(amp);
    }

    ave
}

/*
pub fn remove_offset<T>(a: &mut [T])
where T: ops::Sub<Output = T> + ops::Add<Output = T>
{
    let l = a.len();
    for i in 1..l {
        let il = i-1;
        a[il] = a[i] - a[il];
    }

    for i in 1..l {
        let il = i-1;
        a[i] = a[i] + a[il];
    }
}
*/
pub fn cos_sin(x: f32) -> Cplx<f32> {
    if cfg!(any(feature = "wtf", feature = "approx_trig")) { 
		
		use fast::{sin_norm, cos_norm, wrap};
		let x2 = wrap(x);
		Cplx::new(cos_norm(x2), sin_norm(x2))
		
	} else {
		
		let x = x*std::f32::consts::TAU;
		let y = x.sin_cos();
		Cplx::new(y.1, y.0)
	
	}
}

pub mod interpolate {
	use super::Cplx;
	pub fn linearfc(a: Cplx<f32>, b: Cplx<f32>, t: f32) -> Cplx<f32> {
		a + (b-a).scale(t)
	}

	pub fn linearf(a: f32, b: f32, t: f32) -> f32 {
		a + (b-a)*t
	}

	pub fn cosf(a: f32, b: f32, t: f32) -> f32 {
		a + (b-a)*(0.5-0.5*super::fast::cos_norm(t*0.5))
	}

	pub fn bezierf(a: f32, b: f32, t: f32) -> f32 {
	    a + (b-a)*(t*t*(3.0-2.0*t))
	}

	pub fn nearest<T>(a: T, b: T, t: f32) -> T {
		if t < 0.5 {return a}
		return b
	}

	// perbyte = 1/256 (equivalent to percent = 1/100)
	pub const fn lineari(a: i32, b: i32, perbyte: i32) -> i32 {
		a + (((b-a)*perbyte) >> 8)
	}

	pub fn subtractive_fall(prev: f32, now: f32, min: f32, amount: f32) -> f32 {
        //(now - (prev/now.max(min))*amount).max(min)
        if now > prev {return now}
        let new = prev - amount;
        if new < min {return min}
        if new < now {return now}
        new
	}
	
	pub fn subtractive_fall_hold(
		prev: f32, 
		now: f32, 
		min: f32, 
		amount: f32,
		hold: usize,
		hold_index: &mut usize
	) -> f32 {
		if now > prev {
			*hold_index = 0;
			return now
		}
		
		if *hold_index < hold {
			*hold_index += 1;
			return prev
		} else {
			*hold_index = 0;
			let new = prev - amount;
			if new < min {return min}
			if new < now {return now}
			new
		}
	}

	pub fn multiplicative_fall(prev: f32, now: f32, min: f32, factor: f32) -> f32 {
        if now > prev {return now}
        let new = prev * (1.0 - factor);
        if new < min {return min}
        if new < now {return now}
        new
	}

	pub fn gravitational_fall(prev: f32, now: f32, min: f32, max: &mut f32, acc: f32) -> f32 {
        if now > prev {
            *max = now;
            return now;
        }

        if now == *max {
            return now - acc
        }

        let new = now - (*max - now)*0.5;

        if new < min {return min}
        if new < now {return now}

        new
	}

	pub fn sqrt(a: f32, b: f32, factor: f32) -> f32 {
		let offset = b-a;
		a + (0.1*offset + 1.0).sqrt() - 1.0
	}
	
	pub fn envelope(
		prev: f32, 
		now: f32, 
		limit: f32, 
		attack: f32, 
		release: f32, 
		hold: usize, 
		hold_index: &mut usize
	) -> f32 {
		
		let out = if prev < now {
			
			*hold_index = 0;
			
			/*let new = prev + attack;
			new.min(now)*/
			
			now
			
		} else if prev >= now {
			
			if *hold_index < hold {
				*hold_index += 1;
				prev
			} else {
				
				*hold_index = 0;
				
				let new = prev - release;
				new.max(now)
			}
			
		} else {now};
		
		out.max(limit)
	}
}
/*
pub fn fps_slowdown(no_sample: u8) -> u8 {
    use crate::data::SILENCE_LIMIT;
    if no_sample <
}*/

pub fn highpass_inplace<T>(a: &mut [T])
where T: std::ops::Sub<Output = T> + Copy {
	for i in 1..a.len() {a[i-1] = a[i] - a[i-1]}
}

pub fn fft_scale_up(i: usize, bound: usize) -> f32 {
	const PAD_LOW:  usize = 2;
	const PAD_HIGH: usize = 4;
	((((i + PAD_LOW) * (bound+PAD_HIGH - i)) >> 7) +1) as f32
}
