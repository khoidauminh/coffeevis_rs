pub mod blend;
//pub mod draw;
pub mod draw_raw;

use crate::data::{MAX_WIDTH, foreign::ForeignCommandsCommunicator};
use crate::math::Vec2;
use blend::Mixer;
use draw_raw::*;

const FIELD_START: usize = 64;
const COMMAND_BUFFER_INIT_CAPACITY: usize = MAX_WIDTH as usize;

use std::ops::{self, Deref, DerefMut};

pub(crate) trait Pixel:
    Copy
    + Clone
    + Sized
    + ops::BitAnd<Output = Self>
    + ops::BitOr<Output = Self>
    + ops::Shl<Output = Self>
    + ops::Shr<Output = Self>
    + ops::Add<Output = Self>
    + ops::Sub<Output = Self>
    + ops::Mul<Output = Self>
    + std::fmt::Debug
{
    fn black() -> Self;
    fn white() -> Self;
    fn trans() -> Self;
    fn from(x: u32) -> Self;

    fn over(self, other: Self) -> Self;
    fn mix(self, other: Self) -> Self;
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;

    fn grayb(self) -> u8;

    fn premultiply(self) -> Self;

    fn copy_alpha(self, other: Self) -> Self;

    fn mul_alpha(self, a: u8) -> Self;

    fn blend(self, other: Self) -> Self;

    fn sub_by_alpha(self, other: u8) -> Self;

    fn set_alpha(self, alpha: u8) -> Self;

    fn or(self, other: Self) -> Self;
    fn fade(self, alpha: u8) -> Self;
    fn decompose(self) -> [u8; 4];
    fn compose(array: [u8; 4]) -> Self;
}

#[derive(Debug)]
pub struct DrawCommand<T> {
    pub func: DrawFunction<T>,
    pub param: DrawParam,
    pub color: T,
    pub blending: Mixer<T>,
}

#[derive(Debug)]
pub struct DrawCommandBuffer<T>(Vec<DrawCommand<T>>);

pub struct PixelBuffer<T> {
    out_buffer: Vec<T>,
    buffer: Vec<T>,
    width: usize,
    height: usize,
    field: usize,
    background: T,
    command: DrawCommandBuffer<T>,
    foreign_commands_communicator: Option<ForeignCommandsCommunicator>,
    is_running_foreign: bool,
}

pub(crate) type Canvas = PixelBuffer<u32>;
pub(crate) type P2 = crate::math::Vec2<i32>;

macro_rules! make_draw_func {
    ($fn_name:ident, $func:ident, $param_struct:ident $(, $param:ident: $type:ty)*) => {
        pub fn $fn_name(&mut self, $($param: $type), *, c: T, b: Mixer<T>) {
           	self.0.push(DrawCommand {
          		func: $func,
          		param: DrawParam::$param_struct{ $($param), * },
          		color: c,
          		blending: b
           	});
        }
	}
}

impl<T: Pixel> DrawCommandBuffer<T> {
    pub fn new() -> Self {
        Self(Vec::with_capacity(COMMAND_BUFFER_INIT_CAPACITY))
    }

    pub fn from(vec: Vec<DrawCommand<T>>) -> Self {
        Self(vec)
    }

    make_draw_func!(rect, draw_rect_xy_by, Rect, ps: P2, pe: P2);
    make_draw_func!(rect_wh, draw_rect_wh_by, RectWh, ps: P2, w: usize, h: usize);
    make_draw_func!(line, draw_line_by, Line, ps: P2, pe: P2);
    make_draw_func!(plot, set_pixel_xy_by, Plot, p: P2);
    make_draw_func!(plot_index, set_pixel_by, PlotIdx, i: usize);
    make_draw_func!(circle, draw_cirle_by, Circle, p: P2, r: i32, f: bool);

    pub fn fill(&mut self, c: T) {
        // Discards all previous commands since this
        // fill overwrites the entire buffer.
        self.0.clear();
        self.0.push(DrawCommand {
            func: fill,
            param: DrawParam::Fill {},
            color: c,
            blending: Pixel::over,
        });
    }

    pub fn fade(&mut self, a: u8) {
        self.0.push(DrawCommand {
            func: fade,
            param: DrawParam::Fade { a },
            color: T::trans(),
            blending: Pixel::over,
        });
    }

    pub fn execute(&mut self, canvas: &mut [T], cwidth: usize, cheight: usize, clear: bool) {
        self.0.iter().for_each(|command| {
            (command.func)(
                canvas,
                cwidth,
                cheight,
                command.color,
                command.blending,
                command.param.clone(),
            );
        });

        if clear {
            self.0.clear();
        }
    }
}

impl<T: Pixel> Deref for PixelBuffer<T> {
    type Target = DrawCommandBuffer<T>;
    fn deref(&self) -> &Self::Target {
        &self.command
    }
}

impl<T: Pixel> DerefMut for PixelBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.command
    }
}

impl<T: Pixel> PixelBuffer<T> {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            out_buffer: Vec::new(),
            buffer: vec![T::trans(); w * h],
            command: DrawCommandBuffer::new(),
            width: w,
            height: h,
            field: FIELD_START,
            background: T::from(0xFF_24_24_24),
            foreign_commands_communicator: None,
            is_running_foreign: false,
        }
    }

    pub fn init_commands_communicator(&mut self) {
        self.foreign_commands_communicator = ForeignCommandsCommunicator::new();
        self.is_running_foreign = true;
    }

    pub fn is_foreign(&self) -> bool {
        self.foreign_commands_communicator.is_some() && self.is_running_foreign
    }

    pub fn toggle_running_foreign(&mut self) {
        self.is_running_foreign ^= true;
    }

    pub fn as_slice(&self) -> &[T] {
        self.buffer.as_slice()
    }

    pub fn set_background(&mut self, bg: T) {
        self.background = bg;
    }

    pub fn draw_to_self(&mut self) {
        if self.is_running_foreign {
            if let Some(c) = self.foreign_commands_communicator.as_mut() {
                if let Ok(v) = c.read_commands() {
                    // dbg!(&v);
                    self.command = v;
                    self.clear();
                    self.command
                        .execute(&mut self.buffer, self.width, self.height, false);
                }
                return;
            }
        }

        self.command
            .execute(&mut self.buffer, self.width, self.height, true);
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

    pub fn sizeu(&self) -> Vec2<usize> {
        Vec2::<usize> {
            x: self.width as usize,
            y: self.height as usize,
        }
    }

    pub fn sizel(&self) -> usize {
        self.buffer.len()
    }

    pub fn clear(&mut self) {
        self.buffer.fill(T::trans());
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        let len = w * h;
        if len > self.buffer.len() {
            self.buffer.resize(len, T::from(0));
        }
        self.width = w;
        self.height = h;
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

    pub fn pixel(&self, i: usize) -> T {
        self.buffer[i]
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
    pub fn scale_to(
        &mut self,
        scale: usize,
        dest: &mut [T],
        width: Option<usize>,
        mixer: Option<Mixer<T>>,
        crt: bool,
    ) {
        let mixer = mixer.unwrap_or(T::mix);

        let dst_width = width.unwrap_or(self.width * scale);

        let new_len = dest.len() + FIELD_START * dst_width;

        if self.out_buffer.len() < new_len {
            self.out_buffer.resize(new_len, T::trans());
        }

        if !crt {
            // Shift the lines of the out buffer down
            // to create the illusion of movement.
            //
            // We simulate shifting by sliding the starting
            // point of the buffer backward. When we reach the
            // start of the buffer, we finally do the actual shift.

            self.field = self.field.wrapping_sub(1);

            if self.field > FIELD_START {
                let shift_start = self.height * scale * dst_width;
                let offset = dst_width * FIELD_START;

                for (_, i) in (0..shift_start)
                    .step_by(dst_width)
                    .enumerate()
                    .filter(|&(i, _)| i % scale != 0)
                    .rev()
                {
                    let j = i + dst_width;
                    let z = i + offset;
                    self.out_buffer.copy_within(i..j, z);
                }

                self.field = FIELD_START;
            }
        }

        let index_start = self.field * dst_width;

        if let Some(out_buffer) = self
            .out_buffer
            .get_mut(index_start..index_start + dest.len())
        {
            self.buffer
                .chunks_exact(self.width)
                .zip(out_buffer.chunks_exact_mut(dst_width).step_by(scale))
                .flat_map(|(src_row, dst_row)| src_row.iter().zip(dst_row.chunks_exact_mut(scale)))
                .for_each(|(src_pixel, dst_chunk)| {
                    dst_chunk.fill(mixer(self.background, *src_pixel))
                });

            dest.copy_from_slice(out_buffer);
        }
    }
}
