pub mod blend;

pub mod draw;

use blend::Mixer;

use std::ops;

use crate::data::DEFAULT_BG_COLOR;

#[derive(Clone, Copy, PartialEq)]
pub enum RenderEffect {
    None,
    Crt,
    Interlaced,
}

impl Default for RenderEffect {
    #[allow(unreachable_code)]
    fn default() -> Self {
        #[cfg(feature = "fast")]
        return RenderEffect::None;

        RenderEffect::Interlaced
    }
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

    fn over(self, other: Self) -> Self;
    fn mix(self, other: Self) -> Self;
    fn add(self, other: Self) -> Self;

    fn set_alpha(self, alpha: u8) -> Self;

    fn or(self, other: Self) -> Self;

    fn decompose(self) -> [u8; 4];
    fn compose(array: [u8; 4]) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct P2(pub i32, pub i32);

impl P2 {
    pub fn center(self) -> P2 {
        P2(self.0 / 2, self.1 / 2)
    }

    pub fn to_cplx(self) -> crate::math::Cplx {
        crate::math::Cplx(self.0 as f32, self.1 as f32)
    }

    pub fn scale(self, scale: u8) -> P2 {
        P2(self.0 * scale as i32, self.1 * scale as i32)
    }

    pub fn field(self, field: u8) -> P2 {
        P2(self.0, self.1 + field as i32)
    }
}

impl std::ops::Rem for P2 {
    type Output = P2;

    fn rem(self, rhs: P2) -> Self {
        Self(self.0.rem_euclid(rhs.0), self.1.rem_euclid(rhs.1))
    }
}

impl std::ops::RemAssign for P2 {
    fn rem_assign(&mut self, rhs: P2) {
        self.0 = self.0.rem_euclid(rhs.0);
        self.1 = self.1.rem_euclid(rhs.1);
    }
}

pub struct Painter<'a> {
    buffer: &'a mut [Argb],

    width: usize,
    height: usize,

    color: Argb,
    mixer: Mixer,

    scale: u8,

    field: u8,
    fill: bool,

    background: Argb,
}

impl<'a> Painter<'a> {
    pub fn from(buffer: &'a mut [Argb], width: usize, height: usize, scale: u8, field: u8, fill: bool) -> Self {
        Self {
            buffer,

            width,
            height,

            color: Argb::white(),
            mixer: u32::over,

            scale,

            field,
            fill,

            background: DEFAULT_BG_COLOR,
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

    pub fn logical_width(&self) -> usize {
        self.width / self.scale as usize
    }

    pub fn logical_height(&self) -> usize {
        self.height / self.scale as usize
    }

    pub fn logical_size(&self) -> P2 {
        P2(self.logical_width() as i32, self.logical_height() as i32)
    }

    pub fn logical_sizeu(&self) -> (usize, usize) {
        (self.logical_width(), self.logical_height())
    }

    pub fn sizel(&self) -> usize {
        self.buffer.len()
    }

    pub fn clear(&mut self) {
        self.buffer.fill(self.background);
    }   
    
    pub fn pixel(&self, i: usize) -> Argb {        
        self.buffer[i]
    }
}
