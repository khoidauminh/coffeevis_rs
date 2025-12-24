use super::Argb;

pub type Mixer = fn(Argb, Argb) -> Argb;

use super::Pixel;

pub fn grayb(r: u8, g: u8, b: u8) -> u8 {
    ((r as u16 + g as u16 + 2 * b as u16) / 4) as u8
}

pub fn u8_mul(a: u8, b: u8) -> u8 {
    (a as u16 * b as u16).to_be_bytes()[0]
}

// pub fn argb_fade(this: Argb, a: u8) -> Argb {
//     let [_, r, g, b] = this.decompose();
//     Argb::compose([0x0, u8_mul(r, a), u8_mul(g, a), u8_mul(b, a)])
// }

#[cfg(not(feature = "fast"))]
// Coffeevis no longer supports true compositing
// in order to achieve more performance.
// This color mixing blends the BG folor to FG color
// based on FG's alpha value.
// This explains the interpolation in the name.
pub fn argb32_interpolate(c1: Argb, c2: Argb) -> Argb {
    let [_, r1, g1, b1] = c1.decompose();
    let [a2, r2, g2, b2] = c2.decompose();

    let composite_channel = |c1, c2| {
        let c1 = c1 as u16;
        let c2 = c2 as u16;
        let a2 = a2 as u16;

        let c3 = c1 * (256 - a2) + c2 * a2;
        let c3 = c3 + 255 * (c1 < c2) as u16;

        let [o, _] = c3.to_be_bytes();

        o
    };

    Argb::compose([
        0x0,
        composite_channel(r1, r2),
        composite_channel(g1, g2),
        composite_channel(b1, b2),
    ])
}

impl Pixel for Argb {
    fn black() -> Argb {
        0x00_00_00_00
    }

    fn white() -> Argb {
        0xFF_FF_FF_FF
    }

    fn set_alpha(self, alpha: u8) -> Argb {
        (self & 0x00_FF_FF_FF) | (alpha as Argb) << 24
    }

    fn or(self, other: Argb) -> Argb {
        self | other
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
        #[cfg(feature = "fast")]
        return self | other;

        #[cfg(not(feature = "fast"))]
        argb32_interpolate(self, other)
    }

    #[allow(unreachable_code)]
    fn add(self, other: Argb) -> Argb {
        #[cfg(feature = "fast")]
        return other;

        let [aa, ar, ag, ab] = self.decompose();
        let [ba, br, bg, bb] = other.decompose();
        Argb::from_be_bytes([
            aa,
            ar.saturating_add(u8_mul(br, ba)),
            ag.saturating_add(u8_mul(bg, ba)),
            ab.saturating_add(u8_mul(bb, ba)),
        ])
    }
}
