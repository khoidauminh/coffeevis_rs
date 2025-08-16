pub type Argb = u32;

pub type Mixer = fn(Argb, Argb) -> Argb;

use super::Pixel;

pub fn grayb(r: u8, g: u8, b: u8) -> u8 {
    ((r as u16 + g as u16 + 2 * b as u16) / 4) as u8
}

pub fn u8_mul(a: u8, b: u8) -> u8 {
    (a as u16 * b as u16).to_be_bytes()[0]
}

pub fn argb_fade(this: Argb, other: u8) -> Argb {
    let [aa, r, g, b] = this.decompose();
    Argb::compose([u8_mul(aa, other), r, g, b])
}

pub fn composite_u32(c1: Argb, c2: Argb) -> Argb {
    let [a1, r1, g1, b1] = c1.decompose();
    let [a2, r2, g2, b2] = c2.decompose();

    let (a, a3) = {
        let a1 = a1 as u16;
        let a2 = a2 as u16;

        let a3 = (a1 * (255 - a2)) / 256;

        (a2 + a3, a3)
    };

    if a == 0 {
        return Argb::compose([0, 0, 0, 0]);
    }

    let composite_channel = |c1: u8, c2: u8| -> u8 {
        let c1 = c1 as u16;
        let c2 = c2 as u16;
        let a2 = a2 as u16;
        let a = a as u16;

        ((c2 * a2 + c1 * a3) / a) as u8
    };

    Argb::compose([
        a as u8,
        composite_channel(r1, r2),
        composite_channel(g1, g2),
        composite_channel(b1, b2),
    ])
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

    fn blend(self, other: Argb) -> Argb {
        self.over(other)
    }

    fn copy_alpha(self, other: Argb) -> Argb {
        (self & 0x00_FF_FF_FF) | (other & 0xFF_00_00_00)
    }

    fn grayb(self) -> u8 {
        let [_, r, g, b] = self.decompose();
        grayb(r, g, b)
    }

    fn premultiply(self) -> Argb {
        let [a, ar, ag, ab] = self.decompose();
        Argb::from_be_bytes([a, u8_mul(ar, a), u8_mul(ag, a), u8_mul(ab, a)])
    }

    fn mul_alpha(self, a: u8) -> Argb {
        let [aa, ar, ag, ab] = self.decompose();
        Argb::from_be_bytes([u8_mul(aa, a), ar, ag, ab])
    }

    fn set_alpha(self, alpha: u8) -> Argb {
        (self & 0x00_FF_FF_FF) | (alpha as Argb) << 24
    }

    fn or(self, other: Argb) -> Argb {
        self | other
    }

    fn fade(self, alpha: u8) -> Argb {
        argb_fade(self, alpha)
    }

    fn decompose(self) -> [u8; 4] {
        self.to_be_bytes()
    }

    fn compose(array: [u8; 4]) -> Argb {
        Argb::from_be_bytes(array)
    }

    fn over(self, other: Argb) -> Argb {
        other
    }

    fn mix(self, other: Argb) -> Argb {
        composite_u32(self, other)
    }

    fn add(self, other: Argb) -> Argb {
        let [aa, ar, ag, ab] = self.decompose();
        let [ba, br, bg, bb] = other.decompose();
        Argb::from_be_bytes([
            aa,
            ar.saturating_add(u8_mul(br, ba)),
            ag.saturating_add(u8_mul(bg, ba)),
            ab.saturating_add(u8_mul(bb, ba)),
        ])
    }

    fn sub(self, other: Argb) -> Argb {
        let [aa, ar, ag, ab] = self.decompose();
        let [ba, br, bg, bb] = other.decompose();
        Argb::from_be_bytes([
            aa,
            ar.saturating_sub(u8_mul(br, ba)),
            ag.saturating_sub(u8_mul(bg, ba)),
            ab.saturating_sub(u8_mul(bb, ba)),
        ])
    }

    fn sub_by_alpha(self, other: u8) -> Argb {
        let [aa, ar, ag, ab] = self.decompose();
        Argb::from_be_bytes([
            aa,
            ar.saturating_sub(other),
            ag.saturating_sub(other),
            ab.saturating_sub(other),
        ])
    }
}
