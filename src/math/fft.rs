use super::{Cplx, fast};

pub fn butterfly<T: std::marker::Copy>(a: &mut [Cplx<T>], power: usize) {
	for i in 1..a.len()-1 { // first and last element stays in place
		let ni = super::fast::bit_reverse(i, power);
		if i < ni {a.swap(ni, i)}
	}
}

pub fn butterfly_half<T: std::marker::Copy>(a: &mut [Cplx<T>], power: usize) {
	let power = power;
	let l = a.len()/2;
	
	/*for i in 1..l {
	    a[i] = a[i*2];
	}*/
	
	for i in 1..l-1 {
		let ni = super::fast::bit_reverse(i, power);
		if i < ni {a.swap(ni, i)}
	}
}

pub fn twiddle_norm(x: f32) -> Cplx<f32> {
	//const z: f32 = std::f32::consts::PI;
	let y = (x * crate::math::TAU).sin_cos(); Cplx::<f32>::new(y.1, y.0)
	//Cplx::<f32>::new(fast::cos_norm(x), fast::sin_norm(x))
}


pub fn compute_fft_recursive(a: &mut [Cplx<f32>]) {
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

	let twiddle = twiddle_norm(1.0 / l as f32);
	let mut w = Cplx::<f32>::one();
	for i in 0..lh
	{
		let il = i+lh;

		let q = a[il]*w;

		a[il] = a[i] - q;
		a[i ] = a[i] + q;

		w = w * twiddle;
	}
}

pub fn compute_fft_iterative(a: &mut [Cplx<f32>]) {
    let length = a.len();

    for pair in a.chunks_exact_mut(2) {
        let q = pair[1];
		pair[1] = pair[0] - q;
		pair[0] = pair[0] + q;
    }

    let mut window = 4usize;
    let mut root_angle = -0.25;

    while window <= length {

        let root = twiddle_norm(root_angle);

        a.chunks_exact_mut(window).for_each(|chunk| {
            let (left, right) = chunk.split_at_mut(window / 2);

            let mut factor = Cplx::<f32>::one();

            left.iter_mut()
            .zip(right.iter_mut())
            .for_each(|(smpl, smpr)| {
                let q = *smpr * factor;

                *smpr = *smpl - q;
                *smpl = *smpl + q;

                factor = factor * root;
            });

        });

        window *= 2;
        root_angle *= 0.5;
    }
}

// Discards the other half of the fft.
pub fn compute_fft_half(a: &mut [Cplx<f32>])
{
	let lh = a.len()/2;
	compute_fft_iterative(&mut a[..lh]);
}
