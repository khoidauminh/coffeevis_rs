use crate::data::gen_const::TWIDDLE_MAP;

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

pub fn compute_fft_iterative(a: &mut [Cplx]) {
    for pair in a.chunks_exact_mut(2) {
        let q = pair[1];
        pair[1] = pair[0] - q;
        pair[0] += q;
    }

    for four in a.chunks_exact_mut(4) {
        let mut q = four[2];
        four[2] = four[0] - q;
        four[0] += q;

        q = four[3].times_minus_i();
        four[3] = four[1] - q;
        four[1] += q;
    }

    let length = a.len();

    let mut halfsize = 4usize;
    while halfsize < length {
        let root = &TWIDDLE_MAP[halfsize..];

        let size = halfsize * 2;

        a.chunks_exact_mut(size).for_each(|chunk| {
            let (l, r) = chunk.split_at_mut(halfsize);

            for j in 0..halfsize {
                let z = r[j] * root[j];
                r[j] = l[j] - z;
                l[j] += z;
            }
        });

        halfsize *= 2;
    }
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
        a[i] = Cplx { 
            x: (z1 + z2).l1_norm(),
            y: (z1 - z2).l1_norm(),
        };
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
