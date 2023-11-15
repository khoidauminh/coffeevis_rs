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

pub struct Canvas {
	pix: Vec<u32>,
	//~ pix: crate::mem::StackVec<u32, 20>,
	len: usize,
	mask: usize,
	width: usize,
	height: usize,
	
	pub background: u32,
}

pub type Image = Canvas;

pub fn grayb(r: u8, g: u8, b: u8) -> u8
{
	((r as u16 + g as u16 + 2*b as u16) / 4) as u8
}

impl Canvas {
	pub fn from_buffer(vec: Vec<u32>, w: usize, h: usize, background: u32) -> Self {
		let padded = vec.len().next_power_of_two();
		let mut newvec = vec![0u32; padded];
		newvec[0..vec.len()].copy_from_slice(&vec);
		
		Self {
			pix: newvec,
			len: w*h,
			mask: padded -1,
			width: w,
			height: h,
			
			background: background,
		}
	}

	pub fn new(w: usize, h: usize, background: u32) -> Self {
		let padded = (w*h).next_power_of_two();
		Self {
			pix: vec![0u32; padded],
			mask: padded-1,
			len: w*h,
			width: w,
			height: h,
			background: background,
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

	pub fn clear(&mut self) {	
		self.fill(self.background);
	}
	
	pub fn clear_row(&mut self, y: usize) {	
		// if y >= self.height {return}
		
		let i = y*self.width;
		self.pix[i..i+self.width].fill(COLOR_BLANK);
	}
	
	pub fn subtract_clear(&mut self, amount: u8) {
		self.pix.iter_mut().take(self.len).for_each(|pixel| {
			*pixel = pixel.sub_by_alpha(amount);
		});
	}

	pub fn as_slice<'a>(&'a self) -> &'a [u32] {
		&self.pix[0..self.len]
	}

	pub fn as_mut_slice<'a>(&'a mut self) -> &'a mut [u32] {
		&mut self.pix[0..self.len]
	}

	pub fn resize(&mut self, w: usize, h: usize) {
		let len = w*h;
		let padded = len.next_power_of_two();
		
		self.pix.resize(padded, 0u32);
		
		self.mask = padded-1;
		self.width = w;
		self.height = h;
		self.len = len;
	}

	pub fn update(&mut self) {
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
	
	pub fn get_idx_wrap(&self, p: P2) -> usize {
		self.wrap(self.get_idx_fast(p))
	}

	pub fn pixel(&self, i: usize) -> u32 {
		let iw = self.wrap(i);
		*unsafe{self.pix.get_unchecked(iw)}
	}
	
	fn wrap(&self, i: usize) -> usize {
		i & self.mask 
	}

	pub fn pixel_mut<'a>(&'a mut self, i: usize) -> &'a mut u32 {
		let iw = self.wrap(i);
		unsafe{self.pix.get_unchecked_mut(iw)}
	}

	pub fn pixel_xy(&self, p: P2) -> u32 {
		*unsafe{self.pix.get_unchecked(self.get_idx_wrap(p))}
	}

	pub fn pixel_xy_mut<'a>(&'a mut self, p: P2) -> &'a mut u32 {
		let i = self.get_idx_wrap(p);
		unsafe{self.pix.get_unchecked_mut(i)}
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
