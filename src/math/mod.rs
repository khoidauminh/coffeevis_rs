// You will begin to see uses of approximated math functions instead of the std's.
// The purpose for this is to make the visualizer as fast as possible, accuracy is redundant.

// Many functions in this module are unusued.

use crate::constants::{FFT_SIZE, POWER, pi2, pif32, pih, minus_pi2};

const size_of_usize: usize = std::mem::size_of::<usize>()*8;

type Cplx = (f32, f32);

pub fn fft(a: &mut [Cplx]) {
    let l = a.len();
    let lh = l/2;

    if l == 1 {
        return ();
    }

    let lh = l/2;

    let mut eve = vec![(0f32, 0f32); lh];
    let mut odd = vec![(0f32, 0f32); lh];

    for i in 0..lh {
        eve[i] = a[2*i];
        odd[i] = a[2*i+1];
    }

	fft(&mut eve);
	fft(&mut odd);

    //~ if l > 2 {
        //~ transpose(a, lh);
        //~ fft(&mut a[..lh]);
        //~ fft(&mut a[lh..]);
    //~ }

	let minus_pi2lf = minus_pi2 / l as f32;

    for i in 0..lh {
        let q = euler(odd[i], minus_pi2lf *i as f32);
        let u = eve[i];
        a[i] = complex_add(u, q);
        a[i+lh] = complex_sub(u, q);
    }
}

fn transpose(a: &mut [Cplx], c: usize) {
    let l = a.len();
    let ll = l-1;
    let lh = l/2;
    let mut visited: Vec<bool> = vec![false; l];
    for cycle in 1..lh {
        if visited[cycle] {continue}

        let mut i = cycle;
        while {
            i = if i == ll {ll} else {(c*i)%ll};

            a.swap(cycle, i);
            visited[i] = true;

            i != cycle
        } {}
    }
}

pub fn fft_inplace(a: &mut [Cplx], l: usize) {
    let lh = l >> 1;

    let lg2 = log2i(l);

    for i in 0..l {
        let ni = (i.reverse_bits() >> (size_of_usize-lg2)); // bit reverse operation
        // implementation aquired from: https://stackoverflow.com/q/932079
        if i < ni {
			a.swap(i, ni);
		}
    }

    let mut m = 2;
    while m < l {
        let mh = m/2;
        let minus_pi2lf = minus_pi2 / m as f32;
        let mut k = 0;

        let wm = euler((1.0, 0.0), minus_pi2lf);

        for k in (0..lh).step_by(m) { // the other half doesn't need to be calculated.
			let mut w = (1.0, 0.0);
            for i in k..(mh+k) {
                //let q = euler(a[i+mh], minus_pi2lf *(i-k) as f32);
                let q = complex_mul(w, a[i+mh]);
                let u = a[i];
                a[i] = complex_add(u, q);
                a[i+mh] = complex_sub(u, q);

                w = complex_mul(w, wm);
            }
        }
        m *= 2;
    }
}

pub fn angle_wrap(rawx: f32) -> f32 {
    const pi2_recip: f32 = 1.0 / pi2;
    //(rawx + pif32)%pi2 - pif32
    rawx - pi2*((rawx+pif32)*pi2_recip) as i16 as f32
    //(rawx+pif32).rem_euclid(pi2)-pif32

}

const B: f32 = 4.0/std::f32::consts::PI;
const C: f32 = -4.0/(std::f32::consts::PI*std::f32::consts::PI);

// implemented from https://stackoverflow.com/a/28050328
pub fn fast_sin(rawx: f32) -> f32 {
   // let y = rawx.powi(2).copysign(rawx)*(B + C);
   // y.powi(2).copysign(y)*(0.775 + 0.225)

    let y = rawx*(B + C*rawx.abs());
    y*(0.775 + 0.225*y.abs())
}

pub fn fast_sin_wrap(rawx: f32) -> f32 {
    fast_sin(angle_wrap(rawx))
}

pub fn fast_cos(rawx: f32) -> f32 {
    fast_sin(rawx + pih)
}

pub fn fast_cos_wrap(rawx: f32) -> f32 {
    fast_sin_wrap(rawx + pih)
}

pub fn euler(n: Cplx, a: f32) -> Cplx {
    complex_mul(n, (fast_cos(a), fast_sin(a)))
}

pub fn euler_wrap(n: Cplx, a: f32) -> Cplx {
    complex_mul(n, (fast_cos_wrap(a), fast_sin_wrap(a)))
}

const BIAS: u32 = 127 << 23;
pub fn fast_fsqrt(x: f32) -> f32 {
    let mut xi = x.to_bits() & 0x7F_FF_FF_FF; // discarding the sign, allowing x to be negative
    xi = (xi+BIAS) >> 1;
    //return unsafe{std::mem::transmute::<u32, f32>(xi)};
    f32::from_bits(xi)
}

pub fn fast_flog2(x: f32) -> f32 {
    let mut xi = x.to_bits() & 0x7F_FF_FF_FF;
    let log2 = (xi >> 23) as f32 - 128.0;

    xi &= !(255 << 23);
    xi += BIAS;

    let xi = f32::from_bits(xi);

    log2 + (-0.34484843*xi+2.02466578)*xi -0.67487759
}

pub fn fast_isqrt(x: usize) -> usize {
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

// n having value of 0 is ignored
pub fn log2i(mut n: usize) -> usize {
	(size_of_usize - n.leading_zeros() as usize -1)
}

fn bit_reverse_(mut k: usize, s: usize) -> usize {
    let mut o = 0;
    for i in 1..s {
        o <<= 1;
        o |= (k & 1);
        k >>= 1;
    }
    o
}

pub fn complex_mul(a: Cplx, b: Cplx) -> Cplx {
    return (a.0*b.0 - a.1*b.1, a.0*b.1 + a.1*b.0);
}

pub fn complex_add(a: Cplx, b: Cplx) -> Cplx {
    return (a.0 + b.0, a.1 + b.1);
}

pub fn complex_sub(a: Cplx, b: Cplx) -> Cplx {
    return (a.0 - b.0, a.1 - b.1);
}

pub fn complex_mag(a: Cplx) -> f32 {
    fast_fsqrt(a.0.powi(2) + a.1.powi(2))
}

// ------------------------------------------------------------------------- //

pub fn dft(inp: &[Cplx], out: &mut [Cplx], freq_bound: f32) {
    let l = inp.len();
    let lb = out.len();

    for bin in 0..lb {
        for samp in 0..l {
            let angle = minus_pi2 * bin as f32 * samp as f32 / l as f32;
            out[bin] = complex_add(
                out[bin],
                euler_wrap(
                    complex_mul(inp[samp], (l.abs_diff(samp) as f32 / l as f32, 0.0)),
                    angle
                )
            );
        }
        out[bin] = (out[bin].0/l as f32, out[bin].1/l as f32);
    }
}

#[inline(always)]
pub fn lowpass(s1: Cplx, s2: Cplx, a: f32) -> (f32,  f32) {
    complex_add(s1, complex_mul((a, 0.0), complex_sub(s2, s1)))
}

pub fn lowpass_array(data: &mut [Cplx], a: f32) {
    data[0] = complex_mul((a, 0.0), data[0]);

    for i in 1..data.len() {
       //data[i] = complex_add(data[i-1], complex_mul((a, 0.0), complex_sub(data[i], data[i-1])));
       data[i] = lowpass(data[i-1], data[i], a);
    }
}

// s1: output[i-1], s2: input[i], s2: input[i-1]
#[inline(always)]
pub fn highpass(s1: Cplx, s2: Cplx, s3: Cplx, a: f32) -> Cplx {
    complex_mul((a, 0.0), complex_add(s1, complex_sub(s2, s3)))
}

pub fn highpass_array(data: &mut [Cplx], a: f32) {
    let mut s1 = data[0];
    let mut s2 = data[0];
    for i in 1..data.len() {
        s1 = data[i];
        //data[i] = complex_mul((a, 0), complex_add(data[li], complex_sub(data[i], data[li])));
        data[i] = highpass(data[i-1], data[i], s2, a);
        s2 = s1;
    }
}

pub fn hanning(data: &mut [Cplx]) {
    let l = data.len();
    let lf = ((l-1) as f32).recip();

    for i in 0..l {
        data[i] = complex_mul(data[i], ((3.141592 * i as f32 * lf).sin().powi(2), 0.0f32));
    }
}

pub fn triangle(data: &mut [Cplx]) {
    let l = data.len();
    for i in 0..l/2 {
        let a = i as f32 / l as f32;
        data[i]     = complex_mul((a, 0.0), data[i]);
        data[l-i-1] = complex_mul((1.0-a, 0.0), data[i]);
    }
}

const a_: [f32; 4] = [0.35875, 0.48829, 0.14128, 0.01168];

pub fn blackman_harris(data: &mut Vec<Cplx>) {
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
	pub fn linearf(a: f32, b: f32, t: f32) -> f32 {
		a + (b-a)*t
	}

	pub fn cosf(a: f32, b: f32, t: f32) -> f32 {
		a + (b-a)*(0.5-0.5*super::fast_cos(t*crate::constants::pif32))
	}

	// perbyte = 1/256 (equivalent to percent = 1/100)
	pub const fn lineari(a: i32, b: i32, perbyte: i32) -> i32 {
		a + (((b-a)*perbyte) >> 8)
	}
}

pub fn get_average_lr(data: &Vec<Cplx>) -> Cplx {
    let l = data.len();
    let mut suml = 0.0;
    let mut sumr = 0.0;
    for i in 0..(l/2) {
        suml += data[i].0.abs();
        sumr += data[i+l/2].0.abs();
    }
    (suml / l as f32, sumr / l as f32)
}

#[inline]
pub fn advance_with_limit(a: usize, limit: usize) -> usize {
	let b = a+1;
	b * (b < limit) as usize
}
