use crate::{
	math::Cplx,
	data::DEFAULT_ROTATE_SIZE,
	graphics::{Canvas, P2, blend::Blend}
};

const COPY_SIZE: usize = DEFAULT_ROTATE_SIZE;
const WT_POWER: usize = crate::data::POWER;
const WT_SIZE: usize = 1 << WT_POWER;

struct WaveletTable {
    table: [Cplx; WT_SIZE]
}

impl WaveletTable {
    pub fn init(inp: &mut crate::audio::SampleArr) -> Self {
        let mut cloned = [Cplx::zero(); WT_SIZE];
        for i in 0..WT_SIZE {
            cloned[i] = inp[i >> 2];
        }
        inp.rotate_left(WT_SIZE/8);
        hwt(&mut cloned);
        Self {
            table: cloned
        }
    }
    
    pub fn get(&self, i: usize) -> Option<Cplx> {
        if i >= WT_SIZE {
            return None;
        }
        Some(self.table[i])
    }
    
    pub fn translate_coord(
        x: usize, y: usize, 
        w: usize, h: usize
    ) -> usize {
        let depth = WT_POWER * y / h;
        let depth_width = 1 << depth;
        let index = depth_width * x / w;
        
        // let index_next = index+1;
        // let index_next = if index_next == depth_width {index} else {index_next};
        
        depth_width + index
    }
    
    pub fn draw(&self, canvas: &mut Canvas) {
        let ch = canvas.height();
        let cw = canvas.width();
        for canvas_y in 0..ch {
            let canvas_y_rev = ch - canvas_y - 1;
            let depth = WT_POWER * canvas_y_rev / ch;
            let depth_width = 1 << depth;
            
            let offset = depth_width;
            
            for canvas_x in 0..cw {
                let index = depth_width * canvas_x / cw;
                
                if index >= WT_SIZE {return}
                
                let val = (self.table[offset + index].l1_norm()*128.0) as u8;
                let color = u32::compose([val, val, val, val]);
                
                canvas.set_pixel_xy(P2::new(canvas_x as i32, canvas_y as i32), color)
            }
        }
    }
}

pub fn draw_wavelet(
	prog: &mut crate::data::Program, 
	stream: &mut crate::audio::SampleArr
) {

	let mut w = [Cplx::zero(); WT_SIZE];
	let l = stream.len();
	
	w
	.iter_mut()
	.enumerate()
	.for_each(|(i, smp)| {
		let copy_size = l >> 2;
		let start = 0;
		
		let inew 	= copy_size as f64 * i as f64 / WT_SIZE as f64;
		let ifloor 	= start + inew as usize;
		let iceil  	= start + inew.ceil() as usize;
		
		let t = inew.fract();
		*smp = crate::math::interpolate::linearfc(stream[ifloor], stream[iceil], t);
	});
	
	//w.copy_from_slice(stream.iter().take(WT_SIZE).map(|x| *x).collect::<Vec<_>>().as_slice());
	
	//w.copy_from_slice(&stream[..WT_SIZE.min(stream.len())]);
	hwt(&mut w);
	
	let _wl = WT_SIZE;
	let _pl = prog.pix.sizel();
	let (pw, ph) = (prog.pix.width(), prog.pix.height());
	
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
		
		let yt = WT_SIZE as f64 * (ph - y -1) as f64 / ph as f64;
		
		for x in 0..pw 
		{
			let xt = WT_SIZE as f64 * x as f64 / pw as f64;
			
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
			prog.pix.set_pixel(pi, u32::from_be_bytes([b, r, g, b]));
		}
	}
	
	stream.rotate_left(DEFAULT_ROTATE_SIZE);
}

pub fn draw_wavelet_(
	prog: &mut crate::data::Program, 
	stream: &mut crate::audio::SampleArr
) {
    let table = WaveletTable::init(stream);
    table.draw(&mut prog.pix);
}

fn hwt(a: &mut [Cplx; WT_SIZE]) {
	let mut aux = [Cplx::zero(); WT_SIZE];
	let mut l = WT_SIZE/2;
	
	while l > 0 {
		for i in 0..l {
			let i2 = i*2;
			let i21 = i2+1;
			aux[i]   = (a[i2] + a[i21]).scale(0.5);
			aux[i+l] = a[i2] - a[i21];
		}
		a[..l*2].copy_from_slice(&aux[..l*2]);
		l /= 2;
	}
}

fn hwt_pong(a: &mut [Cplx; WT_SIZE]) {
    let mut aux = [Cplx::zero(); WT_SIZE];
	let mut l = WT_SIZE/2;
	let mut pong = true;
	
	while l > 0 {
	    if pong {
		    for i in 0..l {
			    let i2 = i*2;
			    let i21 = i2+1;
			    aux[i]   = (a[i2] + a[i21]).scale(0.5);
			    aux[i+l] = a[i2] - a[i21];
		    }
		} else {
		    for i in 0..l {
			    let i2 = i*2;
			    let i21 = i2+1;
			    a[i]   = (aux[i2] + aux[i21]).scale(0.5);
			    a[i+l] = aux[i2] - aux[i21];
		    }
		}
		pong ^= true;
		// a[..l*2].copy_from_slice(&aux[..l*2]);
		l /= 2;
	}
	
	if pong {
	    a[..l*2].copy_from_slice(&aux[..l*2]);
	}
}
/*
fn hwt_recursive(a: &mut [Cplx]) 
{
	let l = a.len();
	let aux = vec![Cplx::zero(); l];
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


fn convole(a: &[Cplx], b: &[Cplx], mult: usize, shift: usize) -> Cplx
{
	let mut sum = Cplx::zero();
	let _lb = b.len();
	
	for i in 0..a.len() 
	{
		if let (Some(x), Some(y)) = (a.get(i), b.get(mult*shift - i)) 
		{
				sum = sum + (*x)*(*y);
		}
	}
	
	sum
}

const HAAR_WAVELET_H: &[Cplx] = &[
	Cplx {x: -std::f64::consts::FRAC_1_SQRT_2, y: 0.0},
	Cplx {x:  std::f64::consts::FRAC_1_SQRT_2, y: 0.0}
];

const HAAR_WAVELET_G: &[Cplx] = &[
	Cplx {x: std::f64::consts::FRAC_1_SQRT_2, y: 0.0},
	Cplx {x: std::f64::consts::FRAC_1_SQRT_2, y: 0.0}
];

fn cos_bell_wavelet(x: f64, scale: f64, shift: f64) -> Cplx
{
	let x = scale*(x + shift);
	crate::math::cos_sin(x).scale((-x.powi(2)).exp())
}

// p ranges in 0..w.len()
fn wavelet_xy_interpolated(w: &[Cplx], p: Cplx, pow: usize /*, itpl: fn(f64, f64, f64)->f64*/) -> Cplx 
{	
	let pf = pow as f64;
	let l = w.len();
	let lf = l as f64;
	let ll = l-1;
	
	let idx = |x: f64, h: f64| -> f64
	{
		let iy = ((1<<(h as u32))-1) as f64;
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
