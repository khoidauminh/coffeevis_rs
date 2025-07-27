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

pub fn twiddle_norm(x: f32) -> Cplx {
    let x = x * std::f32::consts::TAU;
    let y = x.sin_cos();
    Cplx::new(y.1, y.0)
}

pub fn twiddle(x: f32) -> Cplx {
    let y = x.sin_cos();
    Cplx::new(y.1, y.0)
}

pub fn compute_fft_iterative(a: &mut [Cplx]) {
    let length = a.len();

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

    const TWIDDLE_FACTORS: [Cplx; 16] = [
        Cplx { x: 1.0, y: 0.0 },
        Cplx { x: -1.0, y: -0.0 },
        Cplx { x: 0.0, y: -1.0 },
        Cplx {
            x: 0.707_106_77,
            y: -0.707_106_77,
        },
        Cplx {
            x: 0.923_879_5,
            y: -0.382_683_43,
        },
        Cplx {
            x: 0.980_785_25,
            y: -0.195_090_32,
        },
        Cplx {
            x: 0.995_184_7,
            y: -0.098_017_14,
        },
        Cplx {
            x: 0.998_795_45,
            y: -0.049_067_676,
        },
        Cplx {
            x: 0.999_698_8,
            y: -0.024_541_229,
        },
        Cplx {
            x: 0.999_924_7,
            y: -0.012_271_538,
        },
        Cplx {
            x: 0.999_981_16,
            y: -0.006_135_884_7,
        },
        Cplx {
            x: 0.999_995_3,
            y: -0.003_067_956_8,
        },
        Cplx {
            x: 0.999_998_8,
            y: -0.001_533_980_1,
        },
        Cplx {
            x: 0.999_999_7,
            y: -0.000_766_990_3,
        },
        Cplx {
            x: 0.999_999_94,
            y: -0.000_383_495_18,
        },
        Cplx {
            x: 0.9999999816164293,
            y: -0.000_191_747_6,
        },
    ];

    let mut depth = 3;
    let mut window = 8usize;

    while window <= length {
        let root = TWIDDLE_FACTORS[depth];

        a.chunks_exact_mut(window).for_each(|chunk| {
            let (left, right) = chunk.split_at_mut(window / 2);

            let q = right[0];
            right[0] = left[0] - q;
            left[0] += q;

            let mut factor = root;

            left.iter_mut()
                .zip(right.iter_mut())
                .skip(1)
                .for_each(|(smpl, smpr)| {
                    let q = *smpr * factor;

                    *smpr = *smpl - q;
                    *smpl += q;

                    factor *= root;
                });
        });

        window *= 2;
        depth += 1;
    }
}

// Avoids having to evaluate a 2nd FFT.
//
// This leverages the the linear and symetric
// property of the FFT.
pub fn compute_fft_stereo(a: &mut [Cplx], up_to: usize, normalize: super::Normalize) {
    super::fft(a);

    let l = a.len();
    let bound = up_to.min(l);

    for i in 1..bound {
        let bin1 = a[i];
        let bin2 = a[l - i].conj();

        a[i] = Cplx::new((bin1 + bin2).l1_norm(), (bin1 - bin2).l1_norm());
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
