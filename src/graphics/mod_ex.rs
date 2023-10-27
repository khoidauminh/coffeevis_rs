pub mod blend;
pub mod draw;
// pub mod space;

use blend::Blend;
use crate::math::Cplx;

pub const COLOR_BLANK: u32 = 0x00_00_00_00;
pub const COLOR_BLACK: u32 = 0xFF_00_00_00;
pub const COLOR_WHITE: u32 = 0xFF_FF_FF_FF;

const SIZE_DEFAULT: (usize, usize) = (50, 50);

pub type P2 = Cplx<i32>;

const MAX_STACK_RESOLUTION: usize = 400;
const MAX_STACK_SIZE: usize = 
	MAX_STACK_RESOLUTION*
	MAX_STACK_RESOLUTION
;

pub struct Canvas {
	#[cfg(not(feature = "noheap"))]
	pix: Vec<u32>,

	#[cfg(feature = "noheap")]
	pix: [u32; MAX_STACK_SIZE],
	
	len: usize,
	width: usize,
	height: usize,
}

pub type Image = Canvas;

pub fn grayb(r: u8, g: u8, b: u8) -> u8
{
	((r as u16 + g as u16 + 2*b as u16) / 4) as u8
}

impl Canvas {
	pub fn from_buffer(vec: Vec<u32>, w: usize, h: usize) -> Self {
		
		#[cfg(feature = "noheap")] 
		{
			
			let mut buf = [0u32; MAX_STACK_SIZE];
			let copy_size = vec.len().min(MAX_STACK_SIZE);
			buf[0..copy_size].copy_from_slice(&vec[0..copy_size]);
			
			Self {
				pix: buf,
				len: w*h,
				width: w,
				height: h,
			}
			
		} 
		
		#[cfg(not(feature = "noheap"))] 
		{
			
			Self {
				pix: vec,
				len: w*h,
				width: w,
				height: h,
			}
			
		}
	}

	pub fn new(w: usize, h: usize) -> Self {
		
		#[cfg(feature = "noheap")] 
		{
		
			Self {
				pix: [0u32; MAX_STACK_SIZE],
				len: w*h,
				width: w,
				height: h,
			}
			
		}
		
		#[cfg(not(feature = "noheap"))] 
		{
			
			Self {
				pix: vec![0u32; w*h],
				len: w*h,
				width: w,
				height: h,
			}
			
		} 
	}
	
	pub fn width(&self) -> usize {
	    self.width
	}
	
	pub fn height(&self) -> usize {
	    self.height
	}

	pub fn size(&self) -> P2 {
		P2::new(self.width as i32, self.height as i32)
	}

	pub fn sizel(&self) -> usize {
		self.len
	}

	pub fn sizet(&self) -> (i32, i32) {
		(self.width as i32, self.height as i32)
	}

	pub fn fill(&mut self, c: u32) {
		self.pix.fill(c);
	}

	pub fn clear<'a>(&'a mut self) {
		/*use std::thread;

		let quarterlen = self.pix.len() / 4;
		let halflen = self.pix.len() / 2;
		let halfquarterlen = halflen + quarterlen;

		let (mut slice_a, mut slice_b) = self.pix.split_at_mut(halflen);
		
		let thread1 = thread::spawn(move || {
			slice_a.fill(COLOR_BLANK);
		});

		let thread2 = thread::spawn(move || {
			slice_b.fill(COLOR_BLANK);
		});	

		thread1.join().unwrap();
		thread2.join().unwrap();*/
	
		self.fill(COLOR_BLANK);
	}
	
	pub fn clear_row(&mut self, y: usize) {	
		// if y >= self.height {return}
		
		let i = y*self.width;
		self.pix[i..i+self.width].fill(COLOR_BLANK);
	}
	
	pub fn subtract_clear(&mut self, amount: u8) {
		self.pix.iter_mut().for_each(|pixel| {
			*pixel = pixel.sub_by_alpha(amount);
		});
	}

	pub fn as_slice<'a>(&'a self) -> &'a [u32] {
		self.pix.as_slice()
	}

	pub fn as_mut_slice<'a>(&'a mut self) -> &'a mut [u32] {
		self.pix.as_mut_slice()
	}

	pub fn fade(&mut self, al: u8) {
		self.pix.iter_mut().for_each(|smp| *smp = blend::u32_fade(*smp, al));
	}

	pub fn resize(&mut self, w: usize, h: usize) {
		
		#[cfg(feature = "noheap")] 
		{
		
			if w*h > self.len {
				panic!(
					"Maximum custom stack allocation size reached\n\
					Note: This binary has been compiled under feature \"noheap\".
				")
			}
		
		}
		
		#[cfg(not(feature = "noheap"))] 
		{
		
			self.pix.resize(w*h, 0u32);
			
		}	
	
		self.width = w;
		self.height = h;
	}

	pub fn update_size(&mut self) {
		self.resize(self.width, self.height);
	}

	pub fn is_in_bound(&self, p: P2) -> bool {
		(p.x as usize) < self.width &&
		(p.y as usize) < self.height
	}

	pub fn get_idx_fast(&self, p: P2) -> usize {
		// if p.x < 0 || p.y < 0 {return usize::MAX}
		let x = p.x as usize;
		let y = p.y as usize;

		/*let out_of_bounds = (!(
		    x >= self.width
	    ) as usize).wrapping_sub(1);*/

		y
        .wrapping_mul(self.width)
        .wrapping_add(x)
	}

	pub fn pixel(&self, i: usize) -> u32 {
		self.pix[i]
	}

	pub fn pixel_mut<'a>(&'a mut self, i: usize) -> &'a mut u32 {
		&mut self.pix[i]
	}

	pub fn pixel_xy(&self, p: P2) -> u32 {
		self.pix[self.get_idx_fast(p)]
	}

	pub fn pixel_xy_mut<'a>(&'a mut self, p: P2) -> &'a mut u32 {
		let i = self.get_idx_fast(p);
		&mut self.pix[i]
	}

	pub fn set_pixel(&mut self, p: P2, c: u32) {
		if self.is_in_bound(p) {
            let i = self.get_idx_fast(p);		
		    self.pix[i] = c;
        }
	}

	pub fn set_pixel_by(&mut self, p: P2, c: u32, b: fn(u32, u32)->u32) {
		if self.is_in_bound(p) {
		    let i = self.get_idx_fast(p);
            let p = &mut self.pix[i];	
		    *p = b(*p, c);
        }
	}
	
	pub fn plot(&mut self, p: P2, c: u32, b: fn(u32, u32)->u32) {
		if self.is_in_bound(p) {
	    	let i = self.get_idx_fast(p);
		    let p = &mut self.pix[i];
		    *p = b(*p, c);
        }
	}

	pub fn scale_to2(&self, dest: &mut [u32], scale: usize) {
		let winw = self.width*scale;
		let winh = self.height*scale;

		let jump = winw - scale;
        let scale2 = scale.pow(2);
        let jumprow = winw*scale2;

		for obase in (0..self.sizel()).step_by(self.width) {
            let ibase = obase*scale2;
            for ox in 0..self.width {
                let pixel = self.pixel(obase+ox);
                let ix = ox*scale;
                let i = ibase+ix;
                dest[i..i+scale].fill(pixel)
            }

            let copy_range = ibase..ibase+winw;
            let start = ibase+winw;
            let bound = ibase+winw*scale;
            for i in (start..bound).step_by(winw) {
                dest.copy_within(copy_range.clone(), i);
            }
        }
	}
	
	pub fn scale_to1(&self, dest: &mut [u32], scale: usize) {
		let dst_width = self.width * scale;
		
		let src_rows = self.pix.chunks_exact(self.width);
		let dst_rows = dest.chunks_exact_mut(dst_width).step_by(scale);
		
		for (src_row, dst_row) in src_rows.zip(dst_rows) {
			for 
				(src_pixel, dst_chunk) in 
				src_row.iter().zip(dst_row.chunks_exact_mut(scale)) 
			{
				dst_chunk.fill(*src_pixel);
			}
		}
		
		for block in dest.chunks_exact_mut(dst_width*scale) {
			let (row1, rows) = block.split_at_mut(dst_width);
			
			for row in rows.chunks_mut(dst_width) {
				row.copy_from_slice(&row1);
			}
		}
	}
	
	pub fn scale_to(&self, dest: &mut [u32], scale: usize) {
		self.scale_to2(dest, scale);
	}
}
