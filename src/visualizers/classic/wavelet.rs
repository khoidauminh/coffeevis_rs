use crate::{
	math::Cplx,
	data::{FFT_POWER, ROTATE_SIZE, SAMPLE_SIZE},
};

const COPY_SIZE: usize = ROTATE_SIZE;

pub const draw_wavelet: crate::VisFunc = |prog, stream|
{
	let mut w = [Cplx::<f32>::zero(); WT_SIZE];
	let l = stream.len();
	
	w
	.iter_mut()
	.enumerate()
	.for_each(|(i, smp)| {
		let copy_size = l >> 2;
		let start = 0;
		
		let inew 	= copy_size as f32 * i as f32 / WT_SIZE as f32;
		let ifloor 	= start + inew as usize;
		let iceil  	= start + inew.ceil() as usize;
		
		let t = inew.fract();
		*smp = crate::math::interpolate::linearfc(stream[ifloor], stream[iceil], t);
	});
	
	//w.copy_from_slice(stream.iter().take(WT_SIZE).map(|x| *x).collect::<Vec<_>>().as_slice());
	
	//w.copy_from_slice(&stream[..WT_SIZE.min(stream.len())]);
	hwt(&mut w);
	
	let wl = WT_SIZE;
	let pl = prog.pix.sizel();
	let (pw, ph) = (prog.pix.width, prog.pix.height);
	
	let power = WT_SIZE.ilog2() as usize;
	/*
	prog.pix.
	as_mut_slice()
	.iter_mut()
	.enumerate()
	.for_each(|(idx, pixel)| 
	{
		let y = idx / pw;
		let x = idx % pw;
		
		let iy = power * (ph - y - 1) / ph;
		let id = 1 << iy;
		let ix = id * x / pw;
		
		let r = crate::math::squish(w[x * wl / pw].x.abs(), 0.5, 255.9) as u8;
		let b = crate::math::squish(w[x * wl / pw].y.abs(), 0.5, 255.9) as u8;
		let g = ((r as u16 + b as u16) / 2) as u8;
		*pixel = u32::from_be_bytes([0xFF, r, g, b]);
	});*/
	
	
	for y in 0..ph
	{
		/*let layer = power * (ph - y -1) / ph;
		let floor = (1 << layer)-1;
		let ceil  = floor*2;*/
		
		let yt = WT_SIZE as f32 * (ph - y -1) as f32 / ph as f32;
		
		for x in 0..pw 
		{
			let xt = WT_SIZE as f32 * x as f32 / pw as f32;
			
			// let ix = (ceil - floor) * x / pw;
			
			// let i = floor + ix;
			
			let smp = wavelet_xy_interpolated(
						&mut w, 
						Cplx::new(xt, yt), 
						power, 
					);
			
			let r = crate::math::squish(smp.x, 0.25, 255.9) as u8;
			let b = crate::math::squish(smp.y, 0.25, 255.9) as u8;
			let g = ((r as u16 + b as u16) / 2) as u8;
			
			let pi = y*pw + x;
			*prog.pix.pixel_mut(pi) = u32::from_be_bytes([0xFF, r, g, b]);
		}
	}
	
	stream.rotate_left(ROTATE_SIZE);
};

const WT_SIZE: usize = 1 << (crate::data::POWER+2);

/*
fn wt(a: &mut [Cplx<f32>; FFT_SIZE])
{
	let mut o = [Cplx::<f32>::zero(); FFT_SIZE]; FFT_POWER];
	
	o[0].copy_from_slice(&stream[..FFT_SIZE]);
	
	for depth in 1..FFT_POWER 
	{
		let lh = FFT_SIZE >> depth;
		for i in 0..lh 
		{
			let i2 = i*2;
			let depthl = depth-1;
			o[depth][i   ] = o[depthl][i2];
			o[depth][i+lh] = o[depthl][i2];
	}
}
*/

fn hwt(a: &mut [Cplx<f32>; WT_SIZE]) 
{
	let mut aux = [Cplx::<f32>::zero(); WT_SIZE];
	let mut l = WT_SIZE/2;
	
	while l > 0
	{
		for i in 0..l 
		{
			let i2 = i*2;
			let i21 = i2+1;
			aux[i]   = (a[i2] + a[i21]).scale(0.5);
			aux[i+l] = a[i2] - a[i21];
		}
		a[..l*2].copy_from_slice(&aux[..l*2]);
		l /= 2;
	}
}
/*
fn hwt_recursive(a: &mut [Cplx<f32>]) 
{
	let l = a.len();
	let aux = vec![Cplx::<f32>::zero(); l];
	for i in 0..l/2
	{
		let i2 = i*2;
		let i21 = i2+1;
		aux[i]   = (a[i2] + a[i21]).scale(0.5);
		aux[i+l] = a[i2] - a[i21];
	}
	a.copy_from_slice(&split);
	hwt_recursive(a[..l/2]);
}
*/
fn wt(a: &mut [Cplx<f32>]) 
{
	let mut aux: Vec<Cplx<f32>> = a.to_vec();
	let mut l = a.len() /2;
	while l > 0 
	{
		for i in 0..l 
		{
			aux[i  ] = convole(a, HAAR_WAVELET_G, 2, i);
			aux[i+l] = convole(a, HAAR_WAVELET_H, 2, i);
		}
		
		a.copy_from_slice(&aux);
		l /= 2;
	}
}

fn convole(a: &[Cplx<f32>], b: &[Cplx<f32>], mult: usize, shift: usize) -> Cplx<f32>
{
	let mut sum = Cplx::<f32>::zero();
	let lb = b.len();
	
	for i in 0..a.len() 
	{
		if let (Some(x), Some(y)) = (a.get(i), b.get(mult*shift - i)) 
		{
				sum = sum + (*x)*(*y);
		}
	}
	
	sum
}

const HAAR_WAVELET_H: &[Cplx<f32>] = &[
	Cplx::<f32> {x: -0.707107, y: 0.0},
	Cplx::<f32> {x: 0.707107, y: 0.0}
];

const HAAR_WAVELET_G: &[Cplx<f32>] = &[
	Cplx::<f32> {x: 0.707107, y: 0.0},
	Cplx::<f32> {x: 0.707107, y: 0.0}
];

fn cos_bell_wavelet(x: f32, scale: f32, shift: f32) -> Cplx<f32>
{
	let x = scale*(x + shift);
	crate::math::cos_sin(x).scale((-x.powi(2)).exp())
}

// p ranges in 0..w.len()
fn wavelet_xy_interpolated(w: &[Cplx<f32>], p: Cplx<f32>, pow: usize /*, itpl: fn(f32, f32, f32)->f32*/) -> Cplx<f32> 
{	
	let pf = pow as f32;
	let l = w.len();
	let lf = l as f32;
	let ll = l-1;
	
	let idx = |x: f32, h: f32| -> f32
	{
		let iy = ((1<<(h as u32))-1) as f32;
		let ix = iy *x /lf;
		iy +ix
	};
	/*
	let idxi = |x: usize, y: usize| -> usize
	{
		let iy = (1<<(pow *y /l)) -1;
		let ix = iy *x /l;
		iy +ix
	};*/
	
	let y = p.y * pf / lf;

	let i0 = idx(p.x, y.floor());
	let i2 = idx(p.x, y.ceil());
	
	let i1 = i0.ceil();
	let i3 = i2.ceil();

	
	if i0 >= lf {return w.last().unwrap().abs()}
	
	/*if i2 >= lf {
		let c1 = linearfc(w[i0 as usize].abs(), w[i1 as usize].abs(), i0.fract()); 
		return linearfc(c1, w[ll], y.fract())
	}*/
	
	/*
	
	let i0 = ii as usize;
	
	// return w[idxi(p.x as usize, p.y as usize)];
	
	let i1 = ii.ceil() as usize;
	let i2 = (ii*2.0) as usize;
	let i3 = (ii*2.0).ceil() as usize;
	
	let dx = ii.fract();
	let dy = (pf *p.y /lf).fract();
	
	*/
	use crate::math::interpolate::linearfc;
	
	let c1 = linearfc(w[i0 as usize].abs(), w[i1 as usize].abs(), i0.fract()); 
	let c2 = linearfc(w[(i2 as usize).min(ll)].abs(), w[(i3 as usize).min(ll)].abs(), i2.fract());
	
	
	linearfc(c1, c2, y.fract())
}