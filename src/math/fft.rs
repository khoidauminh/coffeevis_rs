use super::Cplx;

pub fn butterfly<T>(a: &mut [T], power: usize) {
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
        pair[0] = pair[0] + q;
    }

    for four in a.chunks_exact_mut(4) {
        let mut q = four[2];
        four[2] = four[0] - q;
        four[0] = four[0] + q;

        q = four[3].times_minus_i();
        four[3] = four[1] - q;
        four[1] = four[1] + q;
    }

    const TWIDDLE_FACTORS: [Cplx; 16] = [
        Cplx { x: 1.0, y: 0.0 },
        Cplx { x: -1.0, y: -0.0 },
        Cplx { x: 0.0, y: -1.0 },
        Cplx {
            x: 0.7071067811865476,
            y: -0.7071067811865475,
        },
        Cplx {
            x: 0.9238795325112867,
            y: -0.3826834323650898,
        },
        Cplx {
            x: 0.9807852804032304,
            y: -0.19509032201612825,
        },
        Cplx {
            x: 0.9951847266721969,
            y: -0.0980171403295606,
        },
        Cplx {
            x: 0.9987954562051724,
            y: -0.049067674327418015,
        },
        Cplx {
            x: 0.9996988186962042,
            y: -0.024541228522912288,
        },
        Cplx {
            x: 0.9999247018391445,
            y: -0.012271538285719925,
        },
        Cplx {
            x: 0.9999811752826011,
            y: -0.006135884649154475,
        },
        Cplx {
            x: 0.9999952938095762,
            y: -0.003067956762965976,
        },
        Cplx {
            x: 0.9999988234517019,
            y: -0.0015339801862847655,
        },
        Cplx {
            x: 0.9999997058628822,
            y: -0.0007669903187427045,
        },
        Cplx {
            x: 0.9999999264657179,
            y: -0.00038349518757139556,
        },
        Cplx {
            x: 0.9999999816164293,
            y: -0.0001917475973107033,
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
            left[0] = left[0] + q;

            let mut factor = root;

            left.iter_mut()
                .zip(right.iter_mut())
                .skip(1)
                .for_each(|(smpl, smpr)| {
                    let q = *smpr * factor;

                    *smpr = *smpl - q;
                    *smpl = *smpl + q;

                    factor = factor * root;
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
pub fn compute_fft_stereo(a: &mut [Cplx], up_to: usize, normalize: bool) {
    super::fft(a);

    let l = a.len();
    let bound = up_to.min(l);

    for i in 1..bound {
        let bin1 = a[i];
        let bin2 = a[l - i].conj();

        a[i] = Cplx::new((bin1 + bin2).l1_norm(), (bin1 - bin2).l1_norm());
    }

    if normalize {
        let norm = 1.0 / l as f32;
        for smp in a.iter_mut().take(bound) {
            *smp = *smp * norm;
        }
    }
}

pub fn compute_fft(a: &mut [Cplx]) {
    compute_fft_iterative(a);
}
