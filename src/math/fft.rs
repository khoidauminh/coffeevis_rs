// use crate::data::gen_const::TWIDDLE_MAP;

use std::cell::LazyCell;

use super::Cplx;

pub fn butterfly<T>(a: &mut [T], power: u32) {
    // first and last element stays in place
    for i in 1..a.len() - 1 {
        let ni = super::fast::bit_reverse(i, power);
        if i < ni {
            a.swap(ni, i)
        }
    }
}

thread_local! {
    static TWIDDLE_MAP: LazyCell<[Cplx; 1 << 13]> = LazyCell::new(||{
        let mut out = [Cplx::default(); 1 << 13];

        let mut k = 1;
        let mut i = 1;
        while k < out.len() {
            let angle = -std::f32::consts::PI / k as f32;

            for j in 0..k {
                let (y, x) = (j as f32 * angle).sin_cos();
                out[i] = Cplx(x, y);
                i += 1;
            }

            k *= 2;
        }

        out
    });
}

pub fn compute_fft_iterative(a: &mut [Cplx]) {
    let mut chunk4s = a.chunks_exact_mut(4);
    while let Some([a0, a1, a2, a3]) = chunk4s.next() {
        let a0pa1 = *a0 + *a1;
        let a0ma1 = *a0 - *a1;

        let a2pa3 = *a2 + *a3;
        let a2ma3 = *a2 - *a3;
        let a2ma3j = a2ma3.times_minus_i();

        *a0 = a0pa1 + a2pa3;
        *a1 = a0ma1 + a2ma3j;
        *a2 = a0pa1 - a2pa3;
        *a3 = a0ma1 - a2ma3j;
    }

    let length = a.len();

    let mut halfsize = 4usize;

    TWIDDLE_MAP.with(|twiddlemap| {
        while halfsize < length {
            let root = &twiddlemap[halfsize..];

            let size = halfsize * 2;

            a.chunks_exact_mut(size).for_each(|chunk| {
                let (l, r) = chunk.split_at_mut(halfsize);

                r.iter_mut().zip(root).for_each(|(x, &w)| *x *= w);

                l.iter_mut().zip(r.iter_mut()).for_each(|(l, r)| {
                    let z = *r;
                    *r = *l - z;
                    *l += z;
                });
            });

            halfsize *= 2;
        }
    });
}

// Avoids having to evaluate a 2nd FFT.
//
// This leverages the the linear and symmetric
// property of the FFT.
pub fn compute_fft_stereo(a: &mut [Cplx], up_to: usize, normalize: super::Normalize) {
    super::fft(a);

    let l = a.len();
    let bound = up_to.min(l / 2);

    for i in 1..bound {
        let z1 = a[i];
        let z2 = a[l - i].conj();
        a[i] = Cplx((z1 + z2).l1_norm(), (z1 - z2).l1_norm())
    }

    if normalize == super::Normalize::Yes {
        let norm = 1.0 / l as f32;
        for smp in a.iter_mut().take(bound) {
            *smp *= norm;
        }
    }
}

pub fn compute_fft(a: &mut [Cplx]) {
    compute_fft_iterative(a);
}
