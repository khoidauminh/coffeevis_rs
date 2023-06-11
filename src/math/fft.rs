use super::{Cplx, fast};

pub fn butterfly2<T: std::marker::Copy>(a: &mut [Cplx<T>], power: usize) {
	for i in 0..a.len() {
		let ni = super::fast::bit_reverse(i, power);
		if i < ni {a.swap(ni, i)}
	}
}

pub fn twiddle_norm(x: f32) -> Cplx<f32> {
	Cplx::<f32>::new(fast::cos_norm(x), fast::sin_norm(x))
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
	
	let twiddle = twiddle_norm(- 1.0 / l as f32);
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
    
    while window <= length {
        
        let root = twiddle_norm(-1.0 / window as f32);
        
        a.chunks_exact_mut(window).for_each(|chunk| {
            let (left, right) = chunk.split_at_mut(window >> 1);
        
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

        window <<= 1;
    }
}

// Discards the other half of the fft.
pub fn compute_fft_half(a: &mut [Cplx<f32>]) 
{
	let lh = a.len()/2;
	compute_fft_recursive(&mut a[..lh]);
}
