use super::{Canvas, P2, blend::{Blend, Mixer}, CommandBuffer};

impl Canvas {

	pub fn set_pixel_xy(&mut self, p: P2, c: u32) {
		//~ draw_raw::set_pixel_xy(
			//~ &mut self.pix, self.width, self.height,
			//~ p, c
		//~ );
		
		self.command_buffer.plot(p, c, u32::mix);
	}

	pub fn set_pixel(&mut self, i: usize, c: u32) {
		//~ draw_raw::set_pixel(
			//~ &mut self.pix,
			//~ i, c
		//~ );
		
		self.command_buffer.plot_index(i, c, u32::mix);
	}

	pub fn set_pixel_by(&mut self, i: usize, c: u32, b: Mixer) {
		//~ draw_raw::set_pixel_by(
			//~ &mut self.pix,
			//~ i, c, b
		//~ );
		
		self.command_buffer.plot_index(i, c, b);
	}

	pub fn set_pixel_xy_by(&mut self, p: P2, c: u32, b: Mixer) {
		//~ draw_raw::set_pixel_xy_by(
			//~ &mut self.pix, self.width, self.height,
			//~ p, c, b
		//~ );
		
		self.command_buffer.plot(p, c, b);
	}

	pub fn draw_rect_xy_by(&mut self, ps: P2, pe: P2, c: u32, b: Mixer) {
		// let xbound = self.width;
		// let ybound = self.height.wrapping_sub(1);

		//~ draw_raw::draw_rect_xy_by(
			//~ &mut self.pix, self.width, self.height, 
			//~ ps, pe, c, b
		//~ );
		self.command_buffer.rect(ps, pe, c, b);
	}
	
	pub fn draw_rect_xy(&mut self, ps: P2, pe: P2, c: u32) {
		//~ self.draw_rect_xy_by(ps, pe, c, u32::mix);
		self.command_buffer.rect(ps, pe, c, u32::mix);
	}
	
	pub fn fade(&mut self, al: u8) {
		//~ draw_raw::fade(&mut self.pix, al, self.background);
		self.command_buffer.fade(al, self.background);
	}
	
	pub fn fill(&mut self, c: u32) {
		//~ self.pix[0..self.len].fill(c);
		self.command_buffer.fill(c);
	}
	
	pub fn draw_rect_wh(&mut self, p: P2, w: usize, h: usize, c: u32) {
		//~ draw_raw::draw_rect_wh_by(
			//~ &mut self.pix, self.width, self.height, 
			//~ p, w, h, c, u32::mix
		//~ );
		self.command_buffer.rect_wh(p, w, h, c, u32::mix);
		
	}

	pub fn draw_rect_wh_by(&mut self, p: P2, w: usize, h: usize, c: u32, b: Mixer) {
		//~ draw_raw::draw_rect_wh_by(
			//~ &mut self.pix, self.width, self.height, 
			//~ p, w, h, c, b
		//~ );
		self.command_buffer.rect_wh(p, w, h, c, b);
	}

	// Using Bresenham's line algorithm.
	pub fn draw_line(&mut self, ps: P2, pe: P2, c: u32) {
		//~ draw_raw::draw_line(
			//~ &mut self.pix, self.width, self.height,
			//~ ps, pe, c
		//~ );
		self.command_buffer.line(ps, pe, c, u32::mix);
	}
}
