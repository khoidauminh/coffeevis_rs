pub type Mixer<T> = fn(T, T) -> T;

use std::ops;

pub trait Blend:
    Sized
    + ops::BitAnd<Output = Self>
    + ops::BitOr<Output = Self>
    + ops::Shl<Output = Self>
    + ops::Shr<Output = Self>
    + ops::Add<Output = Self>
    + ops::Sub<Output = Self>
    + ops::Mul<Output = Self>
{
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
    // fn mul(&mut self, other: u32) -> u32;
    // fn mul_alpha(&mut self, other: u32) -> u32;
}

pub fn grayb(r: u8, g: u8, b: u8) -> u8 {
    ((r as u16 + g as u16 + 2 * b as u16) / 4) as u8
}

pub fn u8_mul(a: u8, b: u8) -> u8 {
    let [out, _] = (a as u16 * b as u16).to_be_bytes();
    out
}

pub fn u32_fade(this: u32, other: u8) -> u32 {
    let [aa, r, g, b] = this.to_be_bytes();
    let a = u8_mul(aa, other);
    let r = u8_mul(r, other);
    let g = u8_mul(g, other);
    let b = u8_mul(b, other);
    u32::from_be_bytes([a, r, g, b])
}

pub fn alpha_mix(a: u8, b: u8) -> u8 {
    let a = a as i16;
    let b = b as i16;

    ((a + (b * (256 - a))) >> 8) as u8
}

pub fn channel_mix(x: u8, y: u8, a: u8) -> u8 {
    let x = x as i16;
    let y = y as i16;
    let a = a as i16;

    let [int, _] = y.wrapping_sub(x).wrapping_mul(a).to_be_bytes();

    let o = x + int as i16;

    o as u8 + (x < y) as u8
}

pub fn channel_add(x: u8, y: u8, a: u8) -> u8 {
    x.saturating_add(u8_mul(y, a))
}

impl Blend for u32 {
    fn blend(self, other: u32) -> u32 {
        self.over(other)
    }

    fn copy_alpha(self, other: u32) -> u32 {
        (self & 0x00_FF_FF_FF) | (other & 0xFF_00_00_00)
    }

    fn grayb(self) -> u8 {
        let [_, r, g, b] = self.to_be_bytes();
        grayb(r, g, b)
    }

    fn premultiply(self) -> u32 {
        let [a, ar, ag, ab] = self.to_be_bytes();
        u32::from_be_bytes([a, u8_mul(ar, a), u8_mul(ag, a), u8_mul(ab, a)])
    }

    fn mul_alpha(self, a: u8) -> u32 {
        let [aa, ar, ag, ab] = self.to_be_bytes();
        u32::from_be_bytes([u8_mul(aa, a), ar, ag, ab])
    }

    fn set_alpha(self, alpha: u8) -> u32 {
        self & 0x00_FF_FF_FF | (alpha as u32) << 24
    }

    fn or(self, other: u32) -> u32 {
        self | other
    }

    fn fade(self, alpha: u8) -> u32 {
        u32_fade(self, alpha)
    }

    fn decompose(self) -> [u8; 4] {
        self.to_be_bytes()
    }

    fn compose(array: [u8; 4]) -> u32 {
        u32::from_be_bytes(array)
    }

    fn over(self, other: u32) -> u32 {
        other
    }

    fn mix(self, other: u32) -> u32 {
        let [_aa, ar, ag, ab] = self.to_be_bytes();
        let [ba, br, bg, bb] = other.to_be_bytes();
        u32::from_be_bytes([
            ba, // alpha_mix(aa, ba),
            channel_mix(ar, br, ba),
            channel_mix(ag, bg, ba),
            channel_mix(ab, bb, ba),
        ])
    }
    fn add(self, other: u32) -> u32 {
        let [aa, ar, ag, ab] = self.to_be_bytes();
        let [ba, br, bg, bb] = other.to_be_bytes();
        u32::from_be_bytes([
            aa,
            ar.saturating_add(u8_mul(br, ba)),
            ag.saturating_add(u8_mul(bg, ba)),
            ab.saturating_add(u8_mul(bb, ba)),
        ])
    }

    fn sub(self, other: u32) -> u32 {
        let [aa, ar, ag, ab] = self.to_be_bytes();
        let [ba, br, bg, bb] = other.to_be_bytes();
        u32::from_be_bytes([
            aa,
            ar.saturating_sub(u8_mul(br, ba)),
            ag.saturating_sub(u8_mul(bg, ba)),
            ab.saturating_sub(u8_mul(bb, ba)),
        ])
    }

    fn sub_by_alpha(self, other: u8) -> u32 {
        let [aa, ar, ag, ab] = self.to_be_bytes();
        u32::from_be_bytes([
            aa,
            ar.saturating_sub(other),
            ag.saturating_sub(other),
            ab.saturating_sub(other),
        ])
    }
}

impl Blend for u8 {
    fn blend(self, other: Self) -> Self {
        other
    }

    fn over(self, other: Self) -> Self {
        other
    }

    fn mix(self, other: Self) -> Self {
        let a = self as u16;
        let b = other as u16;

        a.wrapping_add(b).wrapping_shr(1) as Self
    }

    fn add(self, other: Self) -> Self {
        self.saturating_add(other)
    }

    fn sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }

    fn grayb(self) -> Self {
        self
    }

    fn premultiply(self) -> Self {
        unimplemented!()
    }

    fn set_alpha(self, _alpha: u8) -> Self {
        unimplemented!()
    }

    fn copy_alpha(self, _other: Self) -> Self {
        unimplemented!()
    }

    fn mul_alpha(self, a: u8) -> Self {
        u8_mul(self, a)
    }

    fn sub_by_alpha(self, _other: u8) -> Self {
        unimplemented!()
    }

    fn or(self, other: Self) -> Self {
        self | other
    }

    fn fade(self, _alpha: u8) -> Self {
        unimplemented!()
    }

    fn decompose(self) -> [u8; 4] {
        unimplemented!()
    }

    fn compose(array: [u8; 4]) -> Self {
        grayb(array[1], array[2], array[3])
    }
}
