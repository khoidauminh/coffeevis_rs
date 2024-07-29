use std::sync::Arc;

pub mod blend;
pub mod draw;
pub mod draw_raw;
// pub mod space;

use blend::{Blend, Mixer};
pub const COLOR_BLANK: u32 = 0x00_00_00_00;
pub const COLOR_BLACK: u32 = 0xFF_00_00_00;
pub const COLOR_WHITE: u32 = 0xFF_FF_FF_FF;

pub trait Pixel: Copy + Clone + Blend + From<u32> + TryFrom<u32> + From<u8> + TryFrom<u8> {}

impl Pixel for u32 {}

const SIZE_DEFAULT: (usize, usize) = (50, 50);

pub(crate) type P2 = crate::math::Vec2<i32>;

#[derive(Debug, Clone)]
enum DrawCommand<T: Pixel> {
    Rect(P2, P2, T, Mixer<T>),
    RectWh(P2, usize, usize, T, Mixer<T>),
    Line(P2, P2, T, Mixer<T>),
    Plot(P2, T, Mixer<T>),
    PlotIdx(usize, T, Mixer<T>),
    Fill(T),
    Fade(u8, T),
    Merge(Arc<[T]>),
}

trait DrawCommandBuffer<T: Pixel> {
    fn rect(&mut self, ps: P2, pe: P2, c: T, b: Mixer<T>);
    fn rect_wh(&mut self, ps: P2, w: usize, h: usize, c: T, b: Mixer<T>);
    fn line(&mut self, ps: P2, pe: P2, c: T, b: Mixer<T>);
    fn plot(&mut self, p: P2, c: T, b: Mixer<T>);
    fn plot_index(&mut self, i: usize, c: T, b: Mixer<T>);
    fn fill(&mut self, c: T);
    fn fade(&mut self, al: u8, background: T);
    fn execute(&mut self, canvas: &mut [T], cwidth: usize, cheight: usize);
    fn merge(&mut self, canvas: Arc<[T]>);
}

impl<T: Pixel> DrawCommandBuffer<T> for Vec<DrawCommand<T>> {
    fn rect(&mut self, ps: P2, pe: P2, c: T, b: Mixer<T>) {
        self.push(DrawCommand::Rect(ps, pe, c, b));
    }

    fn rect_wh(&mut self, ps: P2, w: usize, h: usize, c: T, b: Mixer<T>) {
        self.push(DrawCommand::RectWh(ps, w, h, c, b));
    }

    fn line(&mut self, ps: P2, pe: P2, c: T, b: Mixer<T>) {
        self.push(DrawCommand::Line(ps, pe, c, b));
    }

    fn plot(&mut self, p: P2, c: T, b: Mixer<T>) {
        self.push(DrawCommand::Plot(p, c, b));
    }

    fn plot_index(&mut self, i: usize, c: T, b: Mixer<T>) {
        self.push(DrawCommand::PlotIdx(i, c, b));
    }

    fn fill(&mut self, c: T) {
        // Discards all previous commands since this
        // fill overwrites the entire buffer.
        self.clear();
        self.push(DrawCommand::Fill(c));
    }

    fn fade(&mut self, al: u8, background: T) {
        self.push(DrawCommand::Fade(al, background));
    }

    fn execute(&mut self, canvas: &mut [T], cwidth: usize, cheight: usize) {
        use DrawCommand as C;

        self.iter().for_each(|command| {
            match command.clone() {
                C::Rect(ps, pe, c, b) => {
                    draw_raw::draw_rect_xy_by(canvas, cwidth, cheight, ps, pe, c, b)
                }

                C::RectWh(ps, w, h, c, b) => {
                    draw_raw::draw_rect_wh_by(canvas, cwidth, cheight, ps, w, h, c, b)
                }

                C::Line(ps, pe, c, b) => {
                    draw_raw::draw_line_by(canvas, cwidth, cheight, ps, pe, c, b)
                }

                C::Plot(p, c, b) => draw_raw::set_pixel_xy_by(canvas, cwidth, cheight, p, c, b),

                C::PlotIdx(i, c, b) => draw_raw::set_pixel_by(canvas, i, c, b),

                C::Fill(c) => draw_raw::fill(canvas, c),

                C::Fade(al, background) => draw_raw::fade(canvas, al, background),

                C::Merge(canvas_other) => draw_raw::merge(canvas, canvas_other),
                // _ => {}
            }
        });

        self.clear();
    }

    fn merge(&mut self, canvas: Arc<[T]>) {
        self.push(DrawCommand::Merge(canvas));
    }
}

pub struct PixelBuffer<T: Pixel> {
    pix: Vec<T>,
    len: usize,
    mask: usize,
    width: usize,
    height: usize,
    command_buffer: Vec<DrawCommand<T>>,

    pub background: T,
}

pub type Image<T> = PixelBuffer<T>;
pub type Canvas = PixelBuffer<u32>;
pub type AlphaMask = PixelBuffer<u8>;

impl<T: Pixel> PixelBuffer<T> {
    pub fn new(w: usize, h: usize, background: T) -> Self {
        let padded = (w * h).next_power_of_two();
        Self {
            pix: vec![T::from(0u8); padded],
            command_buffer: Vec::new(),
            mask: padded - 1,
            len: w * h,
            width: w,
            height: h,
            background,
        }
    }

    pub fn draw_to_self(&mut self) {
        // println!("{:?}", self.command_buffer);

        self.command_buffer
            .execute(&mut self.pix, self.width, self.height);
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
        self.pix.fill(self.background);
    }

    pub fn as_slice(&self) -> &[T] {
        &self.pix[0..self.len]
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.pix[0..self.len]
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        let len = w * h;
        let padded = len.next_power_of_two();

        self.pix.resize(padded, T::from(0u8));

        self.mask = padded - 1;
        self.width = w;
        self.height = h;
        self.len = len;
    }

    pub fn update(&mut self) {
        self.resize(self.width, self.height);
    }

    pub fn is_in_bound(&self, p: P2) -> bool {
        (p.x as usize) < self.width && (p.y as usize) < self.height
    }

    pub fn get_idx_fast(&self, p: P2) -> usize {
        // if p.x < 0 || p.y < 0 {return usize::MAX}
        let x = p.x as usize;
        let y = p.y as usize;

        /*let out_of_bounds = (!(
            x >= self.width
        ) as usize).wrapping_sub(1);*/

        y.wrapping_mul(self.width).wrapping_add(x)
    }

    pub fn get_idx_wrap(&self, p: P2) -> usize {
        self.wrap(self.get_idx_fast(p))
    }

    pub fn pixel(&self, i: usize) -> T {
        let iw = self.wrap(i);
        self.pix[iw]
    }

    fn wrap(&self, i: usize) -> usize {
        i & self.mask
    }

    pub fn pixel_mut(&mut self, i: usize) -> &mut T {
        let iw = self.wrap(i);
        &mut self.pix[iw]
    }

    pub fn pixel_xy(&self, p: P2) -> T {
        self.pix[self.get_idx_wrap(p)]
    }

    pub fn pixel_xy_mut(&mut self, p: P2) -> &mut T {
        let i = self.get_idx_wrap(p);
        &mut self.pix[i]
    }

    pub fn to_rgba(i: &[u32], o: &mut [u8]) {
        for (pin, pout) in i.iter().zip(o.chunks_exact_mut(4)) {
            let mut a = pin.decompose();

            a.rotate_left(1);

            pout.copy_from_slice(&a);
        }
    }

    // On Winit Wayland, resize increments hasn't been implemented,
    // So the width parameter is there to ensure that the horizontal
    // lines are aligned.
    pub fn scale_to(&self, dest: &mut [T], scale: usize, width: Option<usize>) {
        let dst_width = width.unwrap_or(self.width * scale);

        let src_rows = self.pix.chunks_exact(self.width);
        let dst_rows = dest.chunks_exact_mut(dst_width).step_by(scale);

        for (src_row, dst_row) in src_rows.zip(dst_rows) {
            for (src_pixel, dst_chunk) in src_row.iter().zip(dst_row.chunks_exact_mut(scale)) {
                dst_chunk.fill(*src_pixel);
            }
        }

        for block in dest.chunks_exact_mut(dst_width * scale) {
            let (row1, rows) = block.split_at_mut(dst_width);

            for row in rows.chunks_mut(dst_width) {
                row.copy_from_slice(row1);
            }
        }
    }
}

impl Canvas {
    pub fn clear_row(&mut self, y: usize) {
        // if y >= self.height {return}

        let i = y * self.width;
        self.pix[i..i + self.width].fill(COLOR_BLANK);
    }

    pub fn subtract_clear(&mut self, amount: u8) {
        self.pix.iter_mut().take(self.len).for_each(|pixel| {
            *pixel = pixel.sub_by_alpha(amount);
        });
    }
}
