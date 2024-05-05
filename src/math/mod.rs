mod vec2;
mod fft;

pub mod fast;
pub mod rng;
pub mod space;

use std::ops;

pub const PI: f64 = std::f64::consts::PI;
pub const TAU: f64 = PI*2.0;
pub const PIH: f64 = PI*0.5;
pub const TAU_RECIP: f64 = 1.0 / TAU;
pub const ZERO: Cplx = Cplx { x: 0.0, y: 0.0 };

#[derive(Copy, Clone, Debug)]
pub struct Vec2<T: Copy + Clone> {
	pub x: T,
	pub y: T
}

pub type Cplx = Vec2<f64>;

pub trait ToUsize<T> {
    fn new(value: T) -> Self;
}

impl ToUsize<i32> for usize {
	fn new(x: i32) -> usize {
	    if x > 0 {x as usize} else {0}
	
		/*let b = (x < 0) as i32;
		let mask = !(-b);
		
		(x & mask) as usize*/
		
		//x.max(0) as usize
		//const MINUS1: usize = -1i32 as usize;
		//(x as usize) & ((x < 0) as usize).wrapping_add(MINUS1)
	}
}

pub const fn ideal_fft_bound(up_to: usize) -> usize {
	(up_to *3/2).next_power_of_two()*2
}

pub fn fft_stereo(a: &mut [Cplx], up_to: usize, norm: bool) {
	fft::compute_fft_stereo(a, up_to, norm);
}

pub fn fft_stereo_small(a: &mut [Cplx], up_to: usize, normalize: bool) {
	let bound = ideal_fft_bound(up_to);
	let bound = bound.min(a.len());
		
	fft::compute_fft_stereo(&mut a[0..bound], up_to, normalize);
}

pub fn fft_small(a: &mut [Cplx], up_to: usize) {
	let bound = ideal_fft_bound(up_to);
	let bound = bound.min(a.len());
	fft(&mut a[0..bound]);
}

pub fn fft(a: &mut [Cplx]) {
	let l = a.len();
	let power = fast::ilog2(l);

	fft::butterfly(a, power);
	fft::compute_fft(a);
}

pub fn fft_half(a: &mut [Cplx]) {
	let l = a.len();
	let power = l.ilog2() as usize;

	fft::butterfly_half(a, power);
	fft::compute_fft_half(a);
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

pub fn squish(x: f64, scale: f64, limit: f64) -> f64 {
	// (-192.0*x.max(0.0).recip()).exp()
	// const scale: f64 = 621.0;
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

pub fn upscale(a: &mut [Cplx], small_bound: usize, up_bound: usize) {
	let l = a.len();
	let bound = l.min(up_bound);

	for i in (0..bound).rev() {
		
		let small_i = i * small_bound / up_bound;
		
		a[i] = a[small_i];
	}
}

pub fn upscale_linear(a: &mut [Cplx], small_bound: usize, up_bound: usize) {
	let ratio = small_bound as f64 / up_bound as f64;
	let l = a.len();
	let bound = l.min(up_bound);
	
	for i in (0..bound).rev() {
		
		let small_i_f = i as f64 * ratio;

		let small_i = small_i_f as usize;
		let small_i_next = small_i + 1;
		
		let t = small_i_f.fract();
		
		a[i] = interpolate::linearfc(a[small_i], a[small_i_next], t);
	}
}

/*
pub fn derivative<T>(a: &mut [Vec2<T>], amount: f64)
where T: std::marker::Copy + ops::Sub<Output = T>
{
	for i in 1..a.len()
	{
		a[i-1] = a[i] - a[i-1].scale(amount)
	}
}*/

pub fn integrate_inplace(a: &mut [Cplx], factor: usize, norm: bool)
{
    if factor < 2 { return; }

    let mut sum = Cplx::zero();
    let mut table = vec![Cplx::zero(); factor];
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
        let div = 1.0 / factor as f64;
        // a.iter_mut().for_each(|x| *x = *x * Cplx::new((factor as f64).recip(), 0.0));
        let mut i = 0;
        while i < first_iter {a[i] = a[i].scale(1.0 / i as f64); i+=1};
        while i < bound {a[i] = a[i].scale(div); i+=1};
        while i < l {a[i] = a[i].scale(1.0 / (l-i) as f64); i+=1};
    }
}

pub fn normalize_max_cplx(a: &mut [Cplx], limit: f64, threshold: f64, prev_max: f64, smooth_factor: f64) -> f64 {
    let mut max = limit;
    for i in a.iter() {
        max = max.max(i.x.abs());
        max = max.max(i.y.abs());
    }

    // if max >= threshold {return}

    let max = f64::min(max, threshold);

    let max = interpolate::subtractive_fall(prev_max, max, limit, smooth_factor);

    let amp = threshold / max;

    for i in a.iter_mut() {
        *i = i.scale(amp);
    }

    max
}

pub fn normalize_max_f64(a: &mut [f64], limit: f64, threshold: f64, prev_max: f64, smooth_factor: f64) -> f64
{
    let mut max = limit;
    for i in a.iter() {max = max.max(*i)}

    // if max >= threshold {return}

    let max = f64::min(max, threshold);

    let max = interpolate::subtractive_fall(prev_max, max, limit, smooth_factor);

    let amp = max.recip();

    for i in a.iter_mut() {*i *= amp}

    max
}

pub fn normalize_average(a: &mut [Cplx], limit: f64, prev_ave: f64, smooth_factor: f64) -> f64 {
	let mut ave = limit;
	for i in a.iter() {
		ave += i.x.abs();
		ave += i.y.abs();
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
pub fn cos_sin(x: f64) -> Cplx {
    if cfg!(any(feature = "wtf", feature = "approx_trig")) { 
		
		use fast::{sin_norm, cos_norm, wrap};
		let x2 = wrap(x);
		Cplx::new(cos_norm(x2), sin_norm(x2))
		
	} else {
		
		let x = x*std::f64::consts::TAU;
		let y = x.sin_cos();
		Cplx::new(y.1, y.0)
	
	}
}

pub mod interpolate {
	use super::Cplx;
	pub fn linearfc(a: Cplx, b: Cplx, t: f64) -> Cplx {
		a + (b-a).scale(t)
	}

	pub fn linearf(a: f64, b: f64, t: f64) -> f64 {
		a + (b-a)*t
	}

	pub fn cosf(a: f64, b: f64, t: f64) -> f64 {
		a + (b-a)*(0.5-0.5*super::fast::cos_norm(t*0.5))
	}

	pub fn smooth_step(a: f64, b: f64, t: f64) -> f64 {
	    // a + (b-a)*(t*t*(3.0-2.0*t))
		let t = t - 0.5;
		let t = t * (2.0 - 2.0*t.abs()) + 0.5;
		
		a + (b-a)*t
	}

	pub fn nearest<T>(a: T, b: T, t: f64) -> T {
		if t < 0.5 {return a}
		b
	}

	// perbyte = 1/256 (equivalent to percent = 1/100)
	pub const fn lineari(a: i32, b: i32, perbyte: i32) -> i32 {
		a + (((b-a)*perbyte) >> 8)
	}

	pub fn subtractive_fall(prev: f64, now: f64, min: f64, amount: f64) -> f64 {
        //(now - (prev/now.max(min))*amount).max(min)
        if now > prev {return now}
        let new = prev - amount;
        if new < min {return min}
        if new < now {return now}
        new
	}
	
	pub fn multiplicative_fall(prev: f64, now: f64, min: f64, factor: f64) -> f64 {
        if now > prev {return now}
        let new = prev * (1.0 - factor);
        if new < min {return min}
        if new < now {return now}
        new
	}


	pub fn sqrt(a: f64, b: f64, factor: f64) -> f64 {
		let offset = b-a;
		a + offset*factor.sqrt()
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

pub fn fft_scale_up(i: usize, bound: usize) -> f64 {
	const PAD_LOW:  usize = 2;
	const PAD_HIGH: usize = 4;
	((((i + PAD_LOW) * (bound+PAD_HIGH - i)) >> 7) +1) as f64
}
/*
pub mod blackmannuttall {
	use crate::FFT_SIZE;
	use super::Cplx;

	const MASK: usize = FFT_SIZE-1;

	static mut ARRAY_: [f64; FFT_SIZE] = [0.0; FFT_SIZE];
	static INIT: std::sync::Once = std::sync::Once::new();

	pub fn get(i: usize, N: usize) -> f64 {
		/*let mut array = ARRAY_.write().unwrap();

		if array.1 {

		array.0[i * FFT_SIZE / N]

		} else {

			for i in 0..FFT_SIZE {
				array.0[i] = window(i, FFT_SIZE);
			}
			array.1 = true;
			array.0[i * FFT_SIZE / N]
		}*/

		unsafe {
			INIT.call_once(|| {
				for i in 0..FFT_SIZE {
					ARRAY_[i] = window(i, FFT_SIZE);
				}
			});

			ARRAY_[i * FFT_SIZE / N]
		}
	}

	pub fn perform_window(a: &mut [Cplx]) {
		let N = a.len();
		let N_2 = N / 2;
		for i in 0..N_2 {
			let factor = get(i, N);
			let i_rev = N - i - 1;
			a[i] = a[i] * factor;
			a[i_rev] = a[i_rev] * factor;
		}
	}

	fn window(i: usize, N: usize) -> f64 {
		let a0: f64 = 0.3635819;
		let a1: f64 = -0.4891775;
		let a2: f64 = 0.1365995;
		let a3: f64 = -0.0106411;

		let N = N.wrapping_sub(1);
		let x = std::f64::consts::PI * i as f64 / (N as f64);
		a0 + a1* (2.0 * x).cos() + a2 * (4.0 * x).cos() + a3 * (6.0 * x).cos()
	}
}
*/
