use crate::constants::{FFT_SIZE, POWER, pi2, pif32, pih};

const minus_pi2 : f32 = -6.283185307179586f32;

pub fn fft_array_inplace(a : &mut [(f32, f32)]) {
    let l = a.len();

    if l == 1 {
        return ();
    }

    let lh = l/2;

    fft_split_odd_even(a);

	fft_array_inplace(&mut a[..lh]);
	fft_array_inplace(&mut a[lh..]);

	let minus_pi2lf = minus_pi2 / l as f32;

    for i in 0..lh {
        let q = euler(a[i+lh], minus_pi2lf *i as f32);
        let u = a[i];
        a[i] = complex_add(u, q);
        a[i+lh] = complex_sub(u, q);
    }
}

fn fft_split_odd_even(a: &mut [(f32, f32)]) {

	let l = a.len();
	if l < 4 { return; }
    let lh = l/2;

	for i in 1..lh {
		a.swap(i, 2*i);
	}

	fft_split_odd_even(&mut a[lh..]);
    fft_split_odd_even(&mut a[lh..l-lh/2]);
}

// DCT attempt
//~ fn fast_dct(a: &mut [(f32, f32)], k: usize) {
    //~ let l = a.len();

    //~ if l == 1 {
        //~ return ();
    //~ } 

    //~ let lh = l/2;

    //~ fft_split_odd_even(a);

	//~ fast_dct(&mut a[..lh]);
	//~ fast_dct(&mut a[lh..]);

	//~ let minus_pi2lf = minus_pi2 / l as f32;

    //~ for i in 0..lh {
        //~ let q = euler(a[i+lh], std::f32::consts::PI / l as f32 * (i as f32 + 0.5)* k as f32));
        //~ let u = a[i];
        //~ a[i] = complex_add(u, q);
        //~ a[i+lh] = complex_sub(u, q);
    //~ }
//~ }

//~ pub fn image_fast_dct(a: &mut [(f32, f32)], width: usize) {
    
//~ }



// Approximation of sine
pub fn fast_sin(rawx : f32) -> f32 {
    const B : f32 = 4.0/std::f32::consts::PI;
    const C : f32 = -4.0/(std::f32::consts::PI*std::f32::consts::PI);
    
    let x = (rawx + pif32)%pi2 - pif32;

    let y = x*(B + C*x.abs());

    y //*(0.775 + 0.225*y.abs())*0.05
}

// implemented from https://stackoverflow.com/a/28050328
pub fn fast_cos(rawx : f32) -> f32 {
    fast_sin(rawx + pih)
}

// const wrapper : f32 = 2048.0/crate::constants::pi2;
// pub fn cos_from_table(rawx : f32) -> f32 {
//     let x = ((rawx.abs()*wrapper) as usize) & 2047;
//     unsafe { crate::constants::cos_table[x] }
// }

// pub fn sin_from_table(x : f32) -> f32 {
//     cos_from_table(x - 1.570796327)
// }

//~ pub fn fast_sqrt(x : f32) -> f32 {
    //~ let mut i = u32::from_ne(x);
    //~ const bias: u32 = 127 << 3; 
    //~ i = (i + bias) >> 1;
    //~ Ieee74::from_bits(i)
//~ }

pub fn fast_isqrt(x : usize) -> usize {
    if (x < 2) {
        return x;
    }
    
    let small_cand = fast_isqrt(x >> 2) << 1;
    let large_cand = small_cand + 1;
    if large_cand.pow(2) > x {
        return small_cand;
    } 
    return large_cand
}

pub fn euler(n : (f32, f32), a : f32) -> (f32, f32) {
    //complex_mul(n, (a.cos(), a.sin()))
    //complex_mul(n, (cos_from_table(a), sin_from_table(a)))
    complex_mul(n, (fast_cos(a), fast_sin(a)))
}

fn bit_reverse(mut k : usize, s : usize) -> usize {
    let mut o = 0;
    for i in 0..s {
        o = o | ((k >> i) & 1);
        o = o << 1;
    }
    o >> 1
}

pub fn complex_mul(a : (f32, f32), b : (f32, f32)) -> (f32, f32) {
    return (a.0*b.0 - a.1*b.1, a.0*b.1 + a.1*b.0);
}

pub fn complex_add(a : (f32, f32), b : (f32, f32)) -> (f32, f32) {
    return (a.0 + b.0, a.1 + b.1);
}

pub fn complex_sub(a : (f32, f32), b : (f32, f32)) -> (f32, f32) {
    return (a.0 - b.0, a.1 - b.1);
}

// ------------------------------------------------------------------------- //

fn dft(a : &mut Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    let l = a.len();
    let mut o = vec![(0.0f32, 0.0f32); l];
    for i in 0..l {
        for j in 0..l {
            o[i] = complex_add(o[i], euler(a[j], minus_pi2*i as f32 / l as f32));
        }
    }
    o
}

#[inline]
pub fn lowpass(s1: (f32, f32), s2: (f32, f32), a: f32) -> (f32,  f32) {
    complex_add(s1, complex_mul((a, 0.0), complex_sub(s2, s1)))
}

pub fn lowpass_array(data : &mut [(f32, f32)], a : f32) {
    data[0] = complex_mul((a, 0.0), data[0]);

    for i in 1..data.len() {
       //data[i] = complex_add(data[i-1], complex_mul((a, 0.0), complex_sub(data[i], data[i-1])));
       data[i] = lowpass(data[i-1], data[i], a);
    }
}

#[inline]
pub fn highpass(s1: (f32, f32), s2: (f32, f32), s3: (f32, f32), a: f32) -> (f32, f32) {
    complex_mul((a, 0.0), complex_add(s1, complex_sub(s2, s3)))
}

pub fn highpass_array(data : &mut [(f32, f32)], a : f32) {
    let mut s1 = data[0];
    let mut s2 = data[1];
    for i in 1..data.len() {
        let li = i-1;
        s1 = data[i];
        //data[i] = complex_mul((a, 0), complex_add(data[li], complex_sub(data[i], data[li])));
        data[i] = highpass(data[i-1], data[i], s2, a);
        s2 = s1;
    }
}

pub fn hanning(data : &mut [(f32, f32)]) {
    let l = data.len();
    let lf = ((l-1) as f32).recip();

    for i in 0..l {
        data[i] = complex_mul(data[i], ((3.141592 * i as f32 * lf).sin().powi(2), 0.0f32));
    }
}

pub fn triangle(data: &mut [(f32, f32)]) {
    let l = data.len();
    for i in 0..l/2 {
        let a = i as f32 / l as f32;
        data[i]     = complex_mul((a, 0.0), data[i]);
        data[l-i-1] = complex_mul((1.0-a, 0.0), data[i]);
    }
}

const a_ : [f32; 4] = [0.35875, 0.48829, 0.14128, 0.01168];

pub fn blackman_harris(data : &mut Vec<(f32, f32)>) {
    let l = data.len();
    let lf = (l as f32).recip();

    for i in 0..l {
        data[i] = complex_mul(data[i],
            (a_[0]
            - a_[1]*(3.141592*2.0*i as f32 *lf).cos()
            + a_[2]*(3.141592*4.0*i as f32 *lf).cos()
            - a_[3]*(3.141592*6.0*i as f32 *lf).cos()
            ,
            0f32
        ));
    }
}

pub mod interpolate {
	pub fn linearf(a : f32, b : f32, t : f32) -> f32 {
		a + (b-a)*t
	}

	// perbyte = 1/256 (equivalent to percent = 1/100)
	pub fn lineari(a : i32, b : i32, perbyte : i32) -> i32 {
		a + (((b-a)*perbyte) >> 8)
	}
}

// n having value of 0 is ignored
pub fn log2i(n : usize) -> usize {
	if (n < 2) {
		return 0;
	} else {
		return 1 + log2i(n >> 1);
	}
} 

pub fn get_average_lr(data : &Vec<(f32, f32)>) -> (f32, f32) {
    let l = data.len();
    let mut suml = 0.0;
    let mut sumr = 0.0;
    for i in 0..(l/2) {
        suml += data[i].0.abs();
        sumr += data[i+l/2].0.abs();
    }
    (suml / l as f32, sumr / l as f32)
}
