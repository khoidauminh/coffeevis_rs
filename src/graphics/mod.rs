pub mod blend;
pub mod draw;
pub mod draw_raw;

use blend::{Blend, Mixer};
pub const COLOR_BLANK: u32 = 0x00_00_00_00;
pub const COLOR_BLACK: u32 = 0xFF_00_00_00;
pub const COLOR_WHITE: u32 = 0xFF_FF_FF_FF;

pub trait Pixel: Copy + Clone + Blend + From<u32> + TryFrom<u32> + From<u8> + TryFrom<u8> {}

impl Pixel for u32 {}

pub(crate) type P2 = crate::math::Vec2<i32>;

use draw_raw::{DrawFunction, DrawParam};

pub struct DrawCommand<T: Pixel> {
    pub func: DrawFunction<T>,
    pub param: DrawParam,
    pub color: T,
    pub blending: Mixer<T>,
}

impl<T: Pixel> DrawCommand<T> {
    pub fn new(func: DrawFunction<T>, param: DrawParam, color: T, blending: Mixer<T>) -> Self {
        Self {
            func,
            param,
            color,
            blending,
        }
    }
}

macro_rules! make_command {
	($c:expr, $b:expr, $func:ident, $name:ident) => {
		DrawCommand::new($func, DrawParam::$name{}, $c, $b)
	};

	($c:expr, $b:expr, $func:ident, $name:ident, $($e:ident),+) => {
		DrawCommand::new($func, DrawParam::$name{ $($e), + }, $c, $b)
	}
}

pub struct DrawCommandBuffer<T: Pixel> {
    buffer: Vec<DrawCommand<T>>,
}

use draw_raw::*;

impl<T: Pixel> DrawCommandBuffer<T> {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn rect(&mut self, ps: P2, pe: P2, c: T, b: Mixer<T>) {
        self.buffer
            .push(make_command!(c, b, draw_rect_xy_by, Rect, ps, pe));
    }

    pub fn rect_wh(&mut self, ps: P2, w: usize, h: usize, c: T, b: Mixer<T>) {
        self.buffer
            .push(make_command!(c, b, draw_rect_wh_by, RectWh, ps, w, h));
    }

    pub fn line(&mut self, ps: P2, pe: P2, c: T, b: Mixer<T>) {
        self.buffer
            .push(make_command!(c, b, draw_line_by, Line, ps, pe));
    }

    pub fn plot(&mut self, p: P2, c: T, b: Mixer<T>) {
        self.buffer
            .push(make_command!(c, b, set_pixel_xy_by, Plot, p));
    }

    pub fn plot_index(&mut self, i: usize, c: T, b: Mixer<T>) {
        self.buffer
            .push(make_command!(c, b, set_pixel_by, PlotIdx, i));
    }

    pub fn fill(&mut self, c: T) {
        // Discards all previous commands since this
        // fill overwrites the entire buffer.
        self.buffer.clear();
        self.buffer.push(make_command!(c, Blend::over, fill, Fill));
    }

    pub fn circle(&mut self, p: P2, r: i32, f: bool, c: T, b: Mixer<T>) {
        self.buffer
            .push(make_command!(c, b, draw_cirle_by, Circle, p, r, f));
    }

    pub fn fade(&mut self, a: u8, c: T) {
        self.buffer
            .push(make_command!(c, Blend::over, fade, Fade, a));
    }

    pub fn execute(&mut self, canvas: &mut [T], cwidth: usize, cheight: usize) {
        self.buffer.iter().for_each(|command| {
            (command.func)(
                canvas,
                cwidth,
                cheight,
                command.color,
                command.blending,
                command.param,
            );
        });
        self.buffer.clear();
    }
}

pub struct PixelBuffer<T: Pixel> {
    buffer: Vec<T>,
    len: usize,
    mask: usize,
    width: usize,
    height: usize,
    command: DrawCommandBuffer<T>,
    pub background: T,
}

pub type Image<T> = PixelBuffer<T>;
pub type Canvas = PixelBuffer<u32>;
pub type AlphaMask = PixelBuffer<u8>;

impl<T: Pixel> PixelBuffer<T> {
    pub fn new(w: usize, h: usize, background: T) -> Self {
        let padded = crate::math::larger_or_equal_pw2(w * h);
        Self {
            buffer: vec![T::from(0u8); padded],
            command: DrawCommandBuffer::new(),
            mask: padded - 1,
            len: w * h,
            width: w,
            height: h,
            background,
        }
    }

    pub fn draw_to_self(&mut self) {
        self.command
            .execute(&mut self.buffer, self.width, self.height);
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
        self.buffer.fill(self.background);
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        let len = w * h;
        let padded = crate::math::larger_or_equal_pw2(len);

        self.buffer.resize(padded, T::from(0u8));

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
        let x = p.x as u32;
        let y = p.y as u32;

        y.wrapping_mul(self.width as u32).wrapping_add(x) as usize
    }

    pub fn get_idx_wrap(&self, p: P2) -> usize {
        self.wrap(self.get_idx_fast(p))
    }

    pub fn pixel(&self, i: usize) -> T {
        let iw = self.wrap(i);
        self.buffer[iw]
    }

    fn wrap(&self, i: usize) -> usize {
        i & self.mask
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

        let src_rows = self.buffer.chunks_exact(self.width);
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
        let i = y * self.width;
        self.buffer[i..i + self.width].fill(COLOR_BLANK);
    }

    pub fn subtract_clear(&mut self, amount: u8) {
        self.buffer.iter_mut().take(self.len).for_each(|pixel| {
            *pixel = pixel.sub_by_alpha(amount);
        });
    }
}
