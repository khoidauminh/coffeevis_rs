pub mod blend;
//pub mod draw;
pub mod draw;

use crate::data::MAX_WIDTH;
use crate::math::Vec2;
use blend::Mixer;

const FIELD_START: usize = 64;

use std::ops;

#[derive(Clone, Copy, PartialEq)]
pub enum RenderEffect {
    None,
    Crt,
    Interlaced,
}

pub type Argb = u32;

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

    fn alpha(self) -> u8;

    fn or(self, other: Self) -> Self;
    fn fade(self, alpha: u8) -> Self;
    fn decompose(self) -> [u8; 4];
    fn compose(array: [u8; 4]) -> Self;
}

pub struct PixelBuffer {
    out_buffer: Vec<Argb>,
    buffer: Vec<Argb>,
    width: usize,
    height: usize,

    color: Argb,
    mixer: Mixer,

    field: usize,
    background: Argb,
    is_running_foreign: bool,
}

pub(crate) type P2 = crate::math::Vec2<i32>;

impl PixelBuffer {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            out_buffer: Vec::new(),
            buffer: vec![Argb::trans(); w * h],
            width: w,
            height: h,

            color: Argb::white(),
            mixer: u32::over,

            field: FIELD_START,
            background: 0xFF_24_24_24,
            is_running_foreign: false,
        }
    }

    pub fn color(&mut self, c: Argb) {
        self.color = c;
    }

    pub fn mixer(&mut self, mixer: Mixer) {
        self.mixer = mixer;
    }

    pub fn mixerd(&mut self) {
        self.mixer = u32::over;
    }

    pub fn mixerm(&mut self) {
        self.mixer = u32::mix;
    }

    pub fn as_slice(&self) -> &[Argb] {
        self.buffer.as_slice()
    }

    pub fn as_mut_slide(&mut self) -> &mut [Argb] {
        self.buffer.as_mut_slice()
    }

    pub fn set_background(&mut self, bg: Argb) {
        self.background = bg;
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
        self.buffer.fill(Argb::trans());
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        let len = w * h;
        if len > self.buffer.len() {
            self.buffer.resize(len, 0);
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
        let x = p.x as Argb;
        let y = p.y as Argb;

        y.wrapping_mul(self.width as Argb).wrapping_add(x) as usize
    }

    pub fn pixel(&self, i: usize) -> Argb {
        self.buffer[i]
    }

    pub fn to_rgba(i: &[Argb], o: &mut [u8]) {
        for (pin, pout) in i.iter().zip(o.chunks_exact_mut(4)) {
            let mut a = pin.decompose();

            a.rotate_left(1);

            pout.copy_from_slice(&a);
        }
    }

    pub fn clear_out_buffer(&mut self) {
        self.out_buffer.clear();
    }

    // On Winit Wayland, resize increments hasn't been implemented,
    // So the width parameter is there to ensure that the horizontal
    // lines are aligned.
    pub fn scale_to(
        &mut self,
        scale: usize,
        dest: &mut [Argb],
        width: Option<usize>,
        effect: RenderEffect,
    ) {
        if self.width == 0 {
            return;
        }

        let dst_width = width.unwrap_or(self.width * scale);

        if effect == RenderEffect::None {
            if self.out_buffer.len() < dest.len() {
                self.out_buffer.resize(dest.len(), self.background);
            }

            self.buffer
                .chunks_exact(self.width) // source lines
                .zip(self.out_buffer.chunks_exact_mut(dst_width * scale)) // with destination lines
                .flat_map(|(src_row, dst_row)| {
                    src_row.iter().cycle().zip(dst_row.chunks_exact_mut(scale))
                })
                .for_each(|(src_pixel, dst_chunk)| {
                    dst_chunk.fill((self.mixer)(self.background, *src_pixel))
                });

            dest.copy_from_slice(&self.out_buffer);

            return;
        }

        let new_len = dest.len() + FIELD_START * dst_width;

        if self.out_buffer.len() < new_len {
            self.out_buffer.resize(new_len, self.background);
        }

        if effect == RenderEffect::Interlaced {
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
                .chunks_exact(self.width) // source lines
                .zip(out_buffer.chunks_exact_mut(dst_width).step_by(scale)) // with destination lines
                .flat_map(|(src_row, dst_row)| src_row.iter().zip(dst_row.chunks_exact_mut(scale)))
                .for_each(|(src_pixel, dst_chunk)| {
                    dst_chunk.fill((self.mixer)(self.background, *src_pixel))
                });

            dest.copy_from_slice(out_buffer);
        }
    }
}
