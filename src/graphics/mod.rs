use std::sync::{Arc, Mutex};

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

#[derive(Clone)]
enum DrawParam {
    Rect(draw_raw::Rect),
    RectWh(draw_raw::RectWh),
    Line(draw_raw::Line),
    Plot(draw_raw::Plot),
    PlotIdx(draw_raw::PlotIdx),
    Fill(draw_raw::Fill),
    Fade(draw_raw::Fade),
    Circle(draw_raw::Circle),
}

struct DrawCommand<T: Pixel> {
	pub param: DrawParam,
	pub color: T,
	pub blending: Mixer<T>
}

impl<T: Pixel> DrawCommand<T> {
	pub fn new(param: DrawParam, color: T, blending: Mixer<T>) -> Self {
		Self {param, color, blending}
	}
}

macro_rules! make_command {
	
	($c:expr, $b:expr, $name:ident) => {
		DrawCommand::new(DrawParam::$name(draw_raw::$name {}), $c, $b)
	};
	
	($c:expr, $b:expr, $name:ident, $($e:ident),+) => {
		DrawCommand::new(DrawParam::$name(draw_raw::$name { $($e), + }), $c, $b)
	}
}

struct DrawCommandBuffer<T: Pixel> {
	buffer: Vec<DrawCommand<T>>
}

impl<T: Pixel> DrawCommandBuffer<T>  {
	
	pub fn new() -> Self {
		Self { buffer: Vec::new() }
	}
	
    pub fn rect(&mut self, ps: P2, pe: P2, c: T, b: Mixer<T>) {
        self.buffer.push(make_command!(c, b, Rect, ps, pe));
    }

    pub fn rect_wh(&mut self, ps: P2, w: usize, h: usize, c: T, b: Mixer<T>) {
        self.buffer.push(make_command!(c, b, RectWh, ps, w, h));
    }

    pub fn line(&mut self, ps: P2, pe: P2, c: T, b: Mixer<T>) {
        self.buffer.push(make_command!(c, b, Line, ps, pe));
    }

    pub fn plot(&mut self, p: P2, c: T, b: Mixer<T>) {
        self.buffer.push(make_command!(c, b, Plot, p));
    }

    pub fn plot_index(&mut self, i: usize, c: T, b: Mixer<T>) {
        self.buffer.push(make_command!(c, b, PlotIdx, i));
    }

    pub fn fill(&mut self, c: T) {
        // Discards all previous commands since this
        // fill overwrites the entire buffer.
        self.buffer.clear();
        self.buffer.push(make_command!(c, Blend::over, Fill));
    }

    pub fn circle(&mut self, p: P2, r: i32, f: bool, c: T, b: Mixer<T>) {
        self.buffer.push(make_command!(c, b, Circle, p, r, f));
    }

    pub fn fade(&mut self, a: u8, c: T) {
        self.buffer.push(make_command!(c, Blend::over, Fade, a));
    }

    pub fn execute(&mut self, canvas: &mut [T], cwidth: usize, cheight: usize) {
        self.buffer.iter().for_each(|command| {
            
            macro_rules! exec {
				($s:ident) => {
					$s.exec(canvas, cwidth, cheight, command.color, command.blending)
				}
			}
            
            match command.param.clone() {
				DrawParam::Rect(s) 		=> exec!(s),
				DrawParam::RectWh(s) 	=> exec!(s),
				DrawParam::Line(s) 		=> exec!(s),
				DrawParam::Plot(s) 		=> exec!(s),
				DrawParam::PlotIdx(s) 	=> exec!(s),
				DrawParam::Fill(s) 		=> exec!(s),
				DrawParam::Circle(s) 	=> exec!(s),
				DrawParam::Fade(s) 		=> exec!(s)
			};
			
        });

        self.buffer.clear();
    }
}

pub struct PixelBuffer<T: Pixel> {
    pix: Arc<Mutex<Vec<T>>>,
    len: usize,
    mask: usize,
    width: usize,
    height: usize,
    command_buffer: DrawCommandBuffer<T>,

    pub background: T,
}

pub type Image<T> = PixelBuffer<T>;
pub type Canvas = PixelBuffer<u32>;
pub type AlphaMask = PixelBuffer<u8>;

impl<T: Pixel> PixelBuffer<T> {
    pub fn new(w: usize, h: usize, background: T) -> Self {
        let padded = (w * h).next_power_of_two();
        Self {
            pix: Arc::new(Mutex::new(vec![T::from(0u8); padded])),
            command_buffer: DrawCommandBuffer::new(),
            mask: padded - 1,
            len: w * h,
            width: w,
            height: h,
            background,
        }
    }

    pub fn draw_to_self(&mut self) {
        // println!("{:?}", self.command_buffer);

		if let Ok(ref mut canvas) = self.pix.try_lock() {
			self.command_buffer
            .execute(canvas, self.width, self.height);
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
		if let Ok(ref mut canvas) = self.pix.try_lock() {
			canvas.fill(self.background);
		}
    }

    pub fn get_arc(&self) -> Arc<Mutex<Vec<T>>> {
		self.pix.clone()
	}

    pub fn resize(&mut self, w: usize, h: usize) {
        let len = w * h;
        let padded = len.next_power_of_two();

		if let Ok(ref mut canvas) = self.pix.try_lock() {
			canvas.resize(padded, T::from(0u8));

			self.mask = padded - 1;
			self.width = w;
			self.height = h;
			self.len = len;
		}
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
        self.pix.lock().unwrap()[iw]
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
        
        let Ok(ref mut canvas) = self.pix.try_lock() else {
			return
		};
        
        let dst_width = width.unwrap_or(self.width * scale);

        let src_rows = canvas.chunks_exact(self.width);
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
        
        let Ok(ref mut canvas) = self.pix.try_lock() else {
			return
		};

        let i = y * self.width;
        canvas[i..i + self.width].fill(COLOR_BLANK);
    }

    pub fn subtract_clear(&mut self, amount: u8) {
        
        let Ok(ref mut canvas) = self.pix.try_lock() else {
			return
		};
        
        canvas.iter_mut().take(self.len).for_each(|pixel| {
            *pixel = pixel.sub_by_alpha(amount);
        });
    }
}
