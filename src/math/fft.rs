use super::{Cplx, Vec2};

pub fn butterfly<T: std::marker::Copy>(a: &mut [Vec2<T>], power: usize) {
	for i in 1..a.len()-1 { // first and last element stays in place
		let ni = super::fast::bit_reverse(i, power);
		if i < ni {a.swap(ni, i)}
	}
}

pub fn butterfly_half<T: std::marker::Copy>(a: &mut [Vec2<T>], power: usize) {
	let l = a.len()/2;
	
	/*for i in 1..l {
	    a[i] = a[i*2];
	}*/
	
	for i in 1..l-1 {
		let ni = super::fast::bit_reverse(i, power);
		if i < ni {a.swap(ni, i)}
	}
}

pub fn twiddle_norm(x: f64) -> Cplx {
	let x = x*std::f64::consts::TAU;
	let y = x.sin_cos();
	Cplx::new(y.1, y.0)
}

pub fn twiddle(x: f64) -> Cplx {
	let y = x.sin_cos();
	Cplx::new(y.1, y.0)
}

pub fn compute_fft_recursive(a: &mut [Cplx]) {
	let l = a.len();

	if l == 2
	{
		let q = a[1];
		a[1] = a[0] - q;
		a[0] = a[0] + q;
		return
	}

	let lh = l / 2;
	compute_fft_recursive(&mut a[..lh]);
	compute_fft_recursive(&mut a[lh..]);

	let twiddle = twiddle_norm(1.0 / l as f64);
	let mut w = Cplx::one();
	for i in 0..lh
	{
		let il = i+lh;

		let q = a[il]*w;

		a[il] = a[i] - q;
		a[i ] = a[i] + q;

		w = w * twiddle;
	}
}

pub fn compute_fft_iterative(a: &mut [Cplx]) {
    let length = a.len();

    for pair in a.chunks_exact_mut(2) {
        let q = pair[1];
		pair[1] = pair[0] - q;
		pair[0] = pair[0] + q;
    }
    
    for four in a.chunks_exact_mut(4) {
        let mut 
        q = four[2];
        four[2] = four[0] - q;
        four[0] = four[0] + q;
        
        q = four[3].times_minus_i();
        four[3] = four[1] - q;
        four[1] = four[1] + q;
    }
    
    for eight in a.chunks_exact_mut(8) {
        let mut 
        q = eight[4];
        eight[4] = eight[0] - q;
        eight[0] = eight[0] + q;
        
        q = eight[5].times_twiddle_8th();
        eight[5] = eight[1] - q;
        eight[1] = eight[1] + q;
        
        q = eight[6].times_minus_i();
        eight[6] = eight[2] - q;
        eight[2] = eight[2] + q;
        
        q = eight[7].times_twiddle_3_8th();
        eight[7] = eight[3] - q;
        eight[3] = eight[3] + q;
    }

	//~ const FIST_ROOT_ANGLE: f64 = -0.0625 * std::f64::consts::TAU;

    let mut window = 16usize;
	let mut depth = 4;

	/*use std::sync::RwLock;
	
	// 0, 1, 2, 3, 4
	// 1, 2, 4, 8, 16
	pub static FAST_TWIDDLES: RwLock<[Cplx; 16]> = RwLock::new([Cplx::zero(); 16]);
	static ONCE: std::sync::Once = std::sync::Once::new();
		
	ONCE.call_once(|| {
		let mut twiddles = FAST_TWIDDLES.write().unwrap();
		for i in 0..16 {
			let x = -std::f64::consts::TAU / (1 << i) as f64;
			twiddles[i] = super::fft::twiddle(x);
		}
	});
	let fast_twiddle = FAST_TWIDDLES.read().unwrap();*/
	
	const TWIDDLE_FACTORS: [Cplx; 24] = 
	[
		Cplx { x:  1.0, y:  0.0 },
		Cplx { x: -1.0, y: -0.0 },
		Cplx { x:  0.0, y: -1.0 },
		Cplx { x: 0.7071067811865476, y: -0.7071067811865475 },
		Cplx { x: 0.9238795325112867, y: -0.3826834323650898 },
		Cplx { x: 0.9807852804032304, y: -0.19509032201612825 },
		Cplx { x: 0.9951847266721969, y: -0.0980171403295606 },
		Cplx { x: 0.9987954562051724, y: -0.049067674327418015 },
		Cplx { x: 0.9996988186962042, y: -0.024541228522912288 },
		Cplx { x: 0.9999247018391445, y: -0.012271538285719925 },
		Cplx { x: 0.9999811752826011, y: -0.006135884649154475 },
		Cplx { x: 0.9999952938095762, y: -0.003067956762965976 },
		Cplx { x: 0.9999988234517019, y: -0.0015339801862847655 },
		Cplx { x: 0.9999997058628822, y: -0.0007669903187427045 },
		Cplx { x: 0.9999999264657179, y: -0.00038349518757139556 },
		Cplx { x: 0.9999999816164293, y: -0.0001917475973107033 },
		Cplx { x: 0.9999999954041073, y: -0.00009587379909597734 },
		Cplx { x: 0.9999999988510269, y: -0.00004793689960306688 },
		Cplx { x: 0.9999999997127567, y: -0.00002396844980841822 },
		Cplx { x: 0.9999999999281892, y: -0.000011984224905069705 },
		Cplx { x: 0.9999999999820472, y: -0.0000059921124526424275 },
		Cplx { x: 0.9999999999955118, y: -0.000002996056226334661 },
		Cplx { x: 0.999999999998878,  y: -0.0000014980281131690111 },
		Cplx { x: 0.9999999999997194, y: -0.0000007490140565847157 },
	];


    while window <= length {

        //~ let root = twiddle(root_angle);
		let root = TWIDDLE_FACTORS[depth];

        a.chunks_exact_mut(window).for_each(|chunk| {
            let (left, right) = chunk.split_at_mut(window / 2);

			let q = right[0];
			right[0] = left[0] - q;
			left[0]  = left[0] + q;

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
        //~ root_angle *= 0.5;
    }
}

// Avoids having to evaluate a 2nd FFT.
//
// This leverages the the linear and symetric
// property of the FFT.
pub fn compute_fft_stereo(a: &mut [Cplx], up_to: usize, normalize: bool) {
	let l = a.len();
	let _power = super::fast::ilog2(l) as usize;
	
	let norm =  if normalize { 1.0 / l as f64 } else { 1.0 };
	
	super::fft(a);
	
	let mut extract = |i: usize, rev_i: usize| {
		let fft_1 = a[i];
		let fft_2 = a[rev_i].conj();
		
		let x: f64 = (fft_1 + fft_2).l1_norm();
		let y: f64 = (fft_1 - fft_2).l1_norm();

		a[i] = Cplx::new(x, y) * norm;
	};
	
	extract(0, l-1);
	
	for i in 1..up_to.min(l) {
		let rev_i = l-i;
		extract(i, rev_i);
	}
}

pub fn compute_fft_stereo_small(a: &mut [Cplx], up_to: usize, normalize: bool) {
	let bound = if up_to.is_power_of_two() { up_to * 2 } else { up_to.next_power_of_two()*2 };
	let bound = bound.min(a.len());
	compute_fft_stereo(&mut a[0..bound], up_to, normalize);
}

pub fn compute_fft(a: &mut [Cplx]) {
    compute_fft_iterative(a);
}

// Discards the other half of the fft.
pub fn compute_fft_half(a: &mut [Cplx]) {
	let lh = a.len()/2;
	compute_fft(&mut a[..lh]);
}
