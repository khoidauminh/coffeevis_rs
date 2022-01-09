use crate::constants::{FFT_SIZE, POWER, pi2, pif32, pih};

const minus_pi2 : f32 = -6.283185307179586f32;

pub fn cooley_turky_fft_recursive(a : Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    let l = a.len();

    if l == 1 {
        return a;
    }

    let lf = (l as f32).recip();
    let lh = l >> 1;

    let feven;
    let fodd;

    {
        let mut even = vec![(0f32, 0f32); lh];
        let mut odd = vec![(0f32, 0f32); lh];

        for i in 0..lh {
            even[i] = a[2*i];
            odd[i] = a[2*i+1];
        }

        feven = cooley_turky_fft_recursive(even);
        fodd = cooley_turky_fft_recursive(odd);
    }

	let minus_pi2lf = minus_pi2 *lf;
    let mut sum = vec![(0f32, 0f32); l];
    for i in 0..lh {
        let q = euler(fodd[i], minus_pi2lf *i as f32);
        sum[i] = complex_add(feven[i], q);
        sum[i+lh] = complex_sub(feven[i], q);
    }

    return sum;
}

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

pub fn lowpass(data : &mut Vec<(f32, f32)>, a : f32) {
    data[0].0 = a*data[0].0;

    for i in 1..data.len() {
        data[i].0 = data[i-1].0 + a*(data[i].0-data[i-1].0);
    }
}
// These aren't needed at the moment 
pub fn lowpass_array(data : &mut [(f32, f32)], a : f32) {
    data[0] = complex_mul((a, 0.0), data[0]);

    for i in 1..data.len() {
        data[i] = complex_add(data[i-1], complex_mul((a, 0.0), complex_sub(data[i], data[i-1])));
    }
}

pub fn lowpass_bi_array(data : &mut [(f32, f32)], a : f32) {
    let l = data.len()-1;

    data[l] = complex_mul((a, 0.0), data[l]);
    data[0] = complex_mul((a, 0.0), data[0]);

    for i in 1..=l {
    	let ri = l-i;
    	data[i] = complex_add(data[i-1], complex_mul((a, 0.0), complex_sub(data[i], data[i-1])));
        data[ri] = complex_add(data[ri+1], complex_mul((a, 0.0), complex_sub(data[ri], data[ri+1])));
    }
}
//

pub fn hanning(data : &mut Vec<(f32, f32)>) {
    let l = data.len();
    let lf = ((l-1) as f32).recip();

    for i in 0..l {
        data[i] = complex_mul(data[i], ((3.141592 * i as f32 * lf).sin().powi(2), 0.0f32));
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

pub fn triangular(data : &mut Vec<(f32, f32)>) {
    let l = data.len();
    let lf = l as f32;
    let lfh = lf*0.5;

    for i in 0..l {
        data[i] = complex_mul(data[i], (1.0 -((i as f32 -lfh)/lfh).abs(), 0f32));
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
