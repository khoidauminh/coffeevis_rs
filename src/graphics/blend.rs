pub type Mixer<T> = fn(T, T) -> T;
pub type Argb = u32;

use super::Pixel;

pub fn grayb(r: u8, g: u8, b: u8) -> u8 {
    ((r as u16 + g as u16 + 2 * b as u16) / 4) as u8
}

pub fn u8_mul(a: u8, b: u8) -> u8 {
    (a as u16 * b as u16).to_be_bytes()[0]
}

pub fn u32_fade(this: u32, other: u8) -> u32 {
    let [aa, r, g, b] = this.decompose();
    u32::compose([u8_mul(aa, other), r, g, b])
}

pub fn alpha_mix(a: u8, b: u8) -> u8 {
    let a = a as i16;
    let b = b as i16;

    ((a + (b * (256 - a))) >> 8) as u8
}

pub fn channel_mix(x: u8, y: u8, a: u8) -> u8 {
    let [int, _] = (y as i16)
        .wrapping_sub(x as i16)
        .wrapping_mul(a as i16)
        .to_be_bytes();

    int.wrapping_add(x).wrapping_add((x < y) as u8)
}

pub fn channel_add(x: u8, y: u8, a: u8) -> u8 {
    x.saturating_add(u8_mul(y, a))
}

impl Pixel for Argb {
    fn black() -> Argb {
        0xFF_00_00_00
    }

    fn white() -> Argb {
        0xFF_FF_FF_FF
    }

    fn trans() -> Argb {
        0x0
    }

    fn from(x: u32) -> Argb {
        x
    }

    fn blend(self, other: u32) -> u32 {
        self.over(other)
    }

    fn copy_alpha(self, other: u32) -> u32 {
        (self & 0x00_FF_FF_FF) | (other & 0xFF_00_00_00)
    }

    fn grayb(self) -> u8 {
        let [_, r, g, b] = self.decompose();
        grayb(r, g, b)
    }

    fn premultiply(self) -> u32 {
        let [a, ar, ag, ab] = self.decompose();
        u32::from_be_bytes([a, u8_mul(ar, a), u8_mul(ag, a), u8_mul(ab, a)])
    }

    fn mul_alpha(self, a: u8) -> u32 {
        let [aa, ar, ag, ab] = self.decompose();
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
        let [_aa, ar, ag, ab] = self.decompose();
        let [ba, br, bg, bb] = other.decompose();
        u32::from_be_bytes([
            ba, // alpha_mix(aa, ba),
            channel_mix(ar, br, ba),
            channel_mix(ag, bg, ba),
            channel_mix(ab, bb, ba),
        ])
    }
    fn add(self, other: u32) -> u32 {
        let [aa, ar, ag, ab] = self.decompose();
        let [ba, br, bg, bb] = other.decompose();
        u32::from_be_bytes([
            aa,
            ar.saturating_add(u8_mul(br, ba)),
            ag.saturating_add(u8_mul(bg, ba)),
            ab.saturating_add(u8_mul(bb, ba)),
        ])
    }

    fn sub(self, other: u32) -> u32 {
        let [aa, ar, ag, ab] = self.decompose();
        let [ba, br, bg, bb] = other.decompose();
        u32::from_be_bytes([
            aa,
            ar.saturating_sub(u8_mul(br, ba)),
            ag.saturating_sub(u8_mul(bg, ba)),
            ab.saturating_sub(u8_mul(bb, ba)),
        ])
    }

    fn sub_by_alpha(self, other: u8) -> u32 {
        let [aa, ar, ag, ab] = self.decompose();
        u32::from_be_bytes([
            aa,
            ar.saturating_sub(other),
            ag.saturating_sub(other),
            ab.saturating_sub(other),
        ])
    }
}

impl Pixel for u8 {
    fn black() -> Self {
        0
    }

    fn white() -> Self {
        0xFF
    }

    fn trans() -> Self {
        0
    }

    fn from(x: u32) -> Self {
        x as u8
    }

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

    fn fade(self, alpha: u8) -> Self {
        u8_mul(self, alpha)
    }

    fn decompose(self) -> [u8; 4] {
        unimplemented!()
    }

    fn compose(array: [u8; 4]) -> Self {
        grayb(array[1], array[2], array[3])
    }
}
