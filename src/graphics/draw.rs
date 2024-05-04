use super::{Canvas, P2, blend::{Blend, Mixer}, CommandBuffer};

impl Canvas {

	pub fn set_pixel_xy(&mut self, p: P2, c: u32) {		
		self.command_buffer.plot(p, c, u32::mix);
	}

	pub fn set_pixel(&mut self, i: usize, c: u32) {
		self.command_buffer.plot_index(i, c, u32::mix);
	}

	pub fn set_pixel_by(&mut self, i: usize, c: u32, b: Mixer) {
		self.command_buffer.plot_index(i, c, b);
	}

	pub fn set_pixel_xy_by(&mut self, p: P2, c: u32, b: Mixer) {
		self.command_buffer.plot(p, c, b);
	}

	pub fn draw_rect_xy_by(&mut self, ps: P2, pe: P2, c: u32, b: Mixer) {
		self.command_buffer.rect(ps, pe, c, b);
	}
	
	pub fn draw_rect_xy(&mut self, ps: P2, pe: P2, c: u32) {
		self.command_buffer.rect(ps, pe, c, u32::mix);
	}
	
	pub fn fade(&mut self, al: u8) {
		self.command_buffer.fade(al, self.background);
	}
	
	pub fn fill(&mut self, c: u32) {
		self.command_buffer.fill(c);
	}
	
	pub fn draw_rect_wh(&mut self, p: P2, w: usize, h: usize, c: u32) {
		self.command_buffer.rect_wh(p, w, h, c, u32::mix);
		
	}

	pub fn draw_rect_wh_by(&mut self, p: P2, w: usize, h: usize, c: u32, b: Mixer) {
		self.command_buffer.rect_wh(p, w, h, c, b);
	}

	// Using Bresenham's line algorithm.
	pub fn draw_line(&mut self, ps: P2, pe: P2, c: u32) {
		self.command_buffer.line(ps, pe, c, u32::mix);
	}
}
