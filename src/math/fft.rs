use std::vec;

use crate::math::fast::{bit_reverse, ilog2};

use super::Cplx;

const MAX_POWER: usize = 13;
const MAX_SIZE: usize = 1 << MAX_POWER;

pub struct Fft {
    twiddles: Vec<Cplx>,
    butterfly_swap_list: Vec<(usize, usize)>,
    stereo: Option<usize>,
    normalize: bool,
}

impl Fft {
    pub fn new(n: usize) -> Self {
        assert_eq!(n.count_ones(), 1, "Length must be a power of 2");
        assert!(n <= MAX_SIZE, "Length is greater than {}", MAX_SIZE);

        let power = ilog2(n);

        Self {
            butterfly_swap_list: (1..n - 1)
                .map(|i| (i, bit_reverse(i, power)))
                .filter(|(i, ni)| i < ni)
                .collect(),

            twiddles: {
                let mut out = vec![Cplx::default(); n];

                let mut k = 1;

                while k < out.len() {
                    let angle = -std::f32::consts::PI / k as f32;

                    for (j, ele) in out[k..].iter_mut().enumerate() {
                        *ele = Cplx::euler(j as f32 * angle);
                    }

                    k *= 2;
                }

                out
            },

            stereo: None,
            normalize: false,
        }
    }

    pub fn with_stereo(self, up_to: usize) -> Self {
        Self {
            stereo: Some(up_to),
            ..self
        }
    }

    pub fn normalize(self) -> Self {
        Self {
            normalize: true,
            ..self
        }
    }

    pub fn exec(&self, vector: &mut [Cplx]) {
        assert_eq!(self.twiddles.len(), vector.len(), "Length mismatch");

        for (i, ni) in &self.butterfly_swap_list {
            vector.swap(*i, *ni);
        }

        self.compute_fft_iterative(vector);

        if let Some(up_to) = self.stereo {
            Self::decouple_stereo(vector, up_to);
        }

        if self.normalize {
            Self::do_normalize(vector, self.stereo);
        }
    }

    fn compute_fft_iterative(&self, a: &mut [Cplx]) {
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

        while halfsize < length {
            let root = &self.twiddles[halfsize..];

            let size = halfsize * 2;

            a.chunks_exact_mut(size).for_each(|chunk| {
                let (l, r) = chunk.split_at_mut(halfsize);

                for i in 0..halfsize {
                    r[i] *= root[i];
                }

                for i in 0..halfsize {
                    let z = r[i];
                    r[i] = l[i] - z;
                    l[i] += z;
                }
            });

            halfsize *= 2;
        }
    }

    // Avoids having to evaluate a 2nd FFT.
    //
    // This leverages the the linear and symmetric
    // property of the FFT.
    fn decouple_stereo(a: &mut [Cplx], up_to: usize) {
        let l = a.len();
        let bound = up_to.min(l / 2);

        for i in 1..bound {
            let z1 = a[i];
            let z2 = a[l - i].conj();
            a[i] = Cplx((z1 + z2).l1_norm(), (z1 - z2).l1_norm())
        }
    }

    fn do_normalize(a: &mut [Cplx], up_to: Option<usize>) {
        let l = a.len();
        let norm = 1.0 / l as f32;

        let bound = up_to.unwrap_or(a.len()).min(a.len());

        for smp in a.iter_mut().take(bound) {
            *smp *= norm;
        }
    }
}
