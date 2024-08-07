use super::{blend::Mixer, DrawCommandBuffer, Pixel, PixelBuffer, P2};

impl<T: Pixel> PixelBuffer<T> {
    pub fn set_pixel_xy(&mut self, p: P2, c: T) {
        self.command_buffer.plot(p, c, T::mix);
    }

    pub fn set_pixel(&mut self, i: usize, c: T) {
        self.command_buffer.plot_index(i, c, T::mix);
    }

    pub fn set_pixel_by(&mut self, i: usize, c: T, b: Mixer<T>) {
        self.command_buffer.plot_index(i, c, b);
    }

    pub fn set_pixel_xy_by(&mut self, p: P2, c: T, b: Mixer<T>) {
        self.command_buffer.plot(p, c, b);
    }

    pub fn draw_rect_xy_by(&mut self, ps: P2, pe: P2, c: T, b: Mixer<T>) {
        self.command_buffer.rect(ps, pe, c, b);
    }

    pub fn draw_rect_xy(&mut self, ps: P2, pe: P2, c: T) {
        self.command_buffer.rect(ps, pe, c, T::mix);
    }

    pub fn fade(&mut self, al: u8) {
        self.command_buffer.fade(al, self.background);
    }

    pub fn fill(&mut self, c: T) {
        self.command_buffer.fill(c);
    }

    pub fn draw_rect_wh(&mut self, p: P2, w: usize, h: usize, c: T) {
        self.command_buffer.rect_wh(p, w, h, c, T::mix);
    }

    pub fn draw_rect_wh_by(&mut self, p: P2, w: usize, h: usize, c: T, b: Mixer<T>) {
        self.command_buffer.rect_wh(p, w, h, c, b);
    }

    // Using Bresenham's line algorithm.
    pub fn draw_line(&mut self, ps: P2, pe: P2, c: T) {
        self.command_buffer.line(ps, pe, c, T::mix);
    }

    pub fn draw_line_by(&mut self, ps: P2, pe: P2, c: T, b: Mixer<T>) {
        self.command_buffer.line(ps, pe, c, b);
    }
}
