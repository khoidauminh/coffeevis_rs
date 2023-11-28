use super::{Cplx, Vec2, fast};

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
	if cfg!(any(feature = "wtf", feature = "approx_trig")) { 
		let sin = fast::sin_norm(x);
		let cos = fast::cos_norm(x);
		Cplx::new(cos, sin)
	} else {
		let x = x*std::f64::consts::TAU;
		let y = x.sin_cos();
		Cplx::new(y.1, y.0)
	}
}

pub fn twiddle(x: f64) -> Cplx {
	if cfg!(any(feature = "wtf", feature = "approx_trig")) { 
		let x = x * super::TAU_RECIP;
		let sin = fast::sin_norm(x);
		let cos = fast::cos_norm(x);
		Cplx::new(cos, sin)
	} else {
		let y = x.sin_cos();
		Cplx::new(y.1, y.0)
	}
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
    //~ let mut root_angle = FIST_ROOT_ANGLE;
	let mut depth = 4;

    while window <= length {

        //~ let root = twiddle(root_angle);
		let root = super::fast::twiddle(depth);

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
pub fn compute_fft_stereo_small(a: &mut [Cplx], up_to: usize, scale_factor: u8, normalize: bool) {
	let up_to = up_to * scale_factor as usize;
	
	let bound = up_to.next_power_of_two();
	let power = bound.ilog2() as usize + 1;
	
	let l = a.len();
	
	let norm = if normalize { 0.666 / bound as f64 } else { 1.0 };
	
	if bound >= l / 2 {
		super::fft(a);
	} else {
		butterfly(&mut a[0..bound], power);
		butterfly(&mut a[l-bound..l], power);
		
		compute_fft(&mut a[0..bound]);
		compute_fft(&mut a[l-bound..l]);
	}
	
	let mut extract = |i: usize, rev_i: usize| {
		
		let fft_1 = a[i];
		let fft_2 = a[rev_i].conj();
		
		let x = (fft_1 + fft_2).l1_norm();
		let y = (fft_1 - fft_2).l1_norm();
		
		// let scalef = math::fft_scale_up(i, RANGE)* NORMALIZE;
		a[i] = Cplx::new(x, y) * norm;
	};
	
	extract(0, l-1);
	
	for i in 1..up_to {
		
		let rev_i = l-i;
		
		extract(i, rev_i);
	}
}

pub fn compute_fft_stereo(a: &mut [Cplx], up_to: usize, normalize: bool) {
	let l = a.len();
	let _power = l.ilog2() as usize;
	
	let norm = if normalize { 1.0 / l as f64 } else { 1.0 };
	
	super::fft(a);
	
	let mut extract = |i: usize, rev_i: usize| {
		
		let fft_1 = a[i];
		let fft_2 = a[rev_i].conj();
		
		let x = (fft_1 + fft_2).l1_norm();
		let y = (fft_1 - fft_2).l1_norm();
		
		// let scalef = math::fft_scale_up(i, RANGE)* NORMALIZE;
		a[i] = Cplx::new(x, y) * norm;
	};
	
	extract(0, l-1);
	
	for i in 1..(up_to+1).min(l-1) {
		
		let rev_i = l-i;
		
		extract(i, rev_i);
	}
}

pub fn compute_fft(a: &mut [Cplx]) {
    compute_fft_iterative(a);
}

// Discards the other half of the fft.
pub fn compute_fft_half(a: &mut [Cplx]) {
	let lh = a.len()/2;
	compute_fft(&mut a[..lh]);
}
