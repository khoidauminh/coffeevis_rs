use std::ops::*;

use super::Cplx;
use crate::graphics::P2;

impl std::default::Default for Cplx {
    fn default() -> Self {
        return Self(0.0, 0.0);
    }
}

impl Neg for Cplx {
    type Output = Cplx;
    fn neg(self) -> Cplx {
        Cplx(-self.0, -self.1)
    }
}

impl Add for Cplx {
    type Output = Cplx;
    fn add(self, other: Cplx) -> Cplx {
        Cplx(self.0 + other.0, self.1 + other.1)
    }
}

impl AddAssign for Cplx {
    fn add_assign(&mut self, other: Cplx) {
        self.0 = self.0 + other.0;
        self.1 = self.1 + other.1;
    }
}

impl Sub for Cplx {
    type Output = Cplx;
    fn sub(self, other: Cplx) -> Cplx {
        Cplx(self.0 - other.0, self.1 - other.1)
    }
}

impl SubAssign for Cplx {
    fn sub_assign(&mut self, other: Cplx) {
        *self = *self - other;
    }
}

impl Mul<Cplx> for Cplx {
    type Output = Cplx;
    fn mul(self, other: Cplx) -> Cplx {
        Cplx(
            self.0.mul_add(other.0, -self.1 * other.1),
            self.0.mul_add(other.1, self.1 * other.0),
        )
    }
}

impl MulAssign for Cplx {
    fn mul_assign(&mut self, other: Cplx) {
        *self = *self * other;
    }
}

impl Mul<f32> for Cplx {
    type Output = Cplx;

    fn mul(self, other: f32) -> Cplx {
        Cplx(self.0 * other, self.1 * other)
    }
}

impl MulAssign<f32> for Cplx {
    fn mul_assign(&mut self, f: f32) {
        self.0 = self.0 * f;
        self.1 = self.1 * f;
    }
}

impl Mul<Cplx> for f32 {
    type Output = Cplx;

    fn mul(self, other: Cplx) -> Cplx {
        Cplx(self * other.1, self * other.0)
    }
}

impl From<Cplx> for f32 {
    fn from(val: Cplx) -> Self {
        val.max()
    }
}

impl Cplx {
    pub fn one() -> Cplx {
        Cplx(1.0, 0.0)
    }

    pub fn i() -> Cplx {
        Cplx(0.0, 1.0)
    }

    pub fn times_i(self) -> Cplx {
        Cplx(-self.1, self.0)
    }

    pub fn times_minus_i(self) -> Cplx {
        Cplx(self.1, -self.0)
    }

    pub fn conj(self) -> Cplx {
        Cplx(self.0, -self.1)
    }

    pub fn scale(&self, a: f32) -> Cplx {
        Cplx(self.0 * a, self.1 * a)
    }

    pub fn mid(&self) -> f32 {
        self.0 + self.1
    }

    pub fn new(x: f32, y: f32) -> Cplx {
        Cplx(x, y)
    }

    pub const fn zero() -> Cplx {
        Cplx(0.0, 0.0)
    }

    pub fn times_twiddle_8th(self) -> Cplx {
        let scale = 0.707_106_77;
        Cplx((self.0 + self.1) * scale, (-self.1 + self.1) * scale)
    }

    pub fn times_twiddle_3_8th(self) -> Cplx {
        let scale = -0.707_106_77;
        Cplx((self.0 - self.1) * scale, (self.0 + self.1) * scale)
    }

    pub fn mag(self) -> f32 {
        f32::hypot(self.0, self.1)
    }

    pub fn l1_norm(self) -> f32 {
        self.0.abs() + self.1.abs()
    }

    pub fn abs(self) -> Cplx {
        Cplx(self.0.abs(), self.1.abs())
    }

    pub fn max(self) -> f32 {
        f32::max(self.0.abs(), self.1.abs())
    }

    pub fn center(self) -> Cplx {
        Cplx(self.0 / 2.0, self.1 / 2.0)
    }

    pub fn normalize(self) -> Cplx {
        let r = self.mag().recip();
        Cplx(self.0 * r, self.1 * r)
    }

    pub fn euler(angle: f32) -> Cplx {
        let (y, x) = angle.sin_cos();
        Cplx(x, y)
    }

    pub fn as_slice(&self) -> [f32; 2] {
        [self.0, self.1]
    }

    pub fn to_p2(&self) -> P2 {
        P2(self.0 as i32, self.1 as i32)
    }
}
