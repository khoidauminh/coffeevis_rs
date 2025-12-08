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

pub fn strict_log4(mut x: usize) -> usize {
    let mut o = 1;
    while x > 4 {
        o += 1;
        x /= 4;
    }

    if x != 4 {
        panic!("Not a power of 4");
    }

    o
}

pub fn butterfly_radix4<T>(a: &mut [T], power: u32) {
    for i in 1..a.len() - 1 {
        let ni = {
            let mut o = 0;
            let mut x = i;
            for _ in 1..power {
                o *= 4;
                o += x & 0b11;
                x /= 4;
            }

            o
        };

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
                out[i] = Cplx::euler(j as f32 * angle);
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

    let mut size = 16usize;

    TWIDDLE_MAP.with(|twiddle_map| {
        while size <= length {
            let quarter = size / 4;

            a.chunks_exact_mut(size).for_each(|chunk| {
                let (x0, x1, x2, x3) = {
                    let (h0, h1) = chunk.split_at_mut(size / 2);
                    let (x0, x1) = h0.split_at_mut(quarter);
                    let (x2, x3) = h1.split_at_mut(quarter);

                    (x0, x1, x2, x3)
                };

                let roots1 = &twiddle_map[size / 4..];
                let roots2 = &twiddle_map[size / 2..];

                let iter = x0
                    .iter_mut()
                    .zip(x1.iter_mut())
                    .zip(x2.iter_mut())
                    .zip(x3.iter_mut())
                    .enumerate();

                for (i, (((x0, x1), x2), x3)) in iter {
                    let a0 = *x0;
                    let a1 = *x1;
                    let a2 = *x2;
                    let a3 = *x3;

                    let a1w1 = a1 * roots1[i];
                    let a3w1 = a3 * roots1[i];

                    let a0p1w1 = a0 + a1w1;
                    let a2p3w1w2 = (a2 + a3w1) * roots2[i];

                    let a0ma1w1 = a0 - a1w1;
                    let a2ma3w1jw2 = (a2 - a3w1).times_minus_i() * roots2[i];

                    *x0 = a0p1w1 + a2p3w1w2;
                    *x2 = a0p1w1 - a2p3w1w2;
                    *x1 = a0ma1w1 + a2ma3w1jw2;
                    *x3 = a0ma1w1 - a2ma3w1jw2;
                }
            });

            size *= 4;
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
