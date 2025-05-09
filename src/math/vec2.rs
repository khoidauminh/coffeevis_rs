use std::ops::*;

use super::Vec2;

impl<T> Neg for Vec2<T>
where
    T: Neg<Output = T> + Copy,
{
    type Output = Vec2<T>;
    fn neg(self) -> Vec2<T> {
        Vec2::<T> {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> Add for Vec2<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Vec2<T>;
    fn add(self, other: Vec2<T>) -> Vec2<T> {
        Vec2::<T> {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T> AddAssign for Vec2<T>
where
    T: Add<Output = T> + Copy,
{
    fn add_assign(&mut self, other: Vec2<T>) {
        self.x = self.x + other.x;
        self.y = self.y + other.y;
    }
}

impl<T> Sub for Vec2<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vec2<T>;
    fn sub(self, other: Vec2<T>) -> Vec2<T> {
        Vec2::<T> {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T> SubAssign for Vec2<T>
where
    T: Sub<Output = T> + Copy,
{
    fn sub_assign(&mut self, other: Vec2<T>) {
        *self = *self - other;
    }
}

impl Mul<Vec2<u32>> for Vec2<u32> {
    type Output = Vec2<u32>;
    fn mul(self, other: Vec2<u32>) -> Vec2<u32> {
        Vec2::<u32> {
            x: self.x * other.x - self.y * other.y,
            y: self.x * other.y + self.y * other.x,
        }
    }
}

impl Mul<Vec2<f32>> for Vec2<f32> {
    type Output = Vec2<f32>;
    fn mul(self, other: Vec2<f32>) -> Vec2<f32> {
        Vec2::<f32> {
            x: self.x.mul_add(other.x, -self.y * other.y),
            y: self.x.mul_add(other.y, self.y * other.x),
        }
    }
}

impl MulAssign for Vec2<f32> {
    fn mul_assign(&mut self, other: Vec2<f32>) {
        *self = *self * other;
    }
}

impl MulAssign for Vec2<u32> {
    fn mul_assign(&mut self, other: Vec2<u32>) {
        *self = *self * other;
    }
}

impl<T> Mul<T> for Vec2<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec2<T>;

    fn mul(self, other: T) -> Vec2<T> {
        Vec2::<T> {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl<T> MulAssign<T> for Vec2<T>
where
    T: Mul<Output = T> + Copy,
{
    fn mul_assign(&mut self, f: T) {
        self.x = self.x * f;
        self.y = self.y * f;
    }
}

impl Mul<Vec2<f32>> for f32 {
    type Output = Vec2<f32>;

    fn mul(self, other: Vec2<f32>) -> Vec2<f32> {
        Vec2::<f32> {
            x: self * other.y,
            y: self * other.x,
        }
    }
}

impl<T> Vec2<T>
where
    T: Copy + Clone,
{
    pub fn as_slice(&self) -> [T; 2] {
        [self.x, self.y]
    }

    pub fn as_tuple(&self) -> (T, T) {
        (self.x, self.y)
    }
}

impl<T> Vec2<T>
where
    T: Default + Copy + Mul<Output = T> + Add<Output = T> + Neg<Output = T>,
{
    pub fn one() -> Vec2<f32> {
        Vec2::<f32> { x: 1.0, y: 0.0 }
    }

    pub fn i() -> Vec2<f32> {
        Vec2::<f32> { x: 0.0, y: 1.0 }
    }

    pub fn times_i(self) -> Vec2<T> {
        Vec2::<T> {
            x: -self.y,
            y: self.x,
        }
    }

    pub fn times_minus_i(self) -> Vec2<T> {
        Vec2::<T> {
            x: self.y,
            y: -self.x,
        }
    }

    pub fn conj(self) -> Vec2<T> {
        Vec2::<T> {
            x: self.x,
            y: -self.y,
        }
    }

    pub fn scale(&self, a: T) -> Vec2<T> {
        Vec2::<T> {
            x: self.x * a,
            y: self.y * a,
        }
    }

    pub fn mid(&self) -> T {
        self.x + self.y
    }
}

impl From<Vec2<f32>> for f32 {
    fn from(val: Vec2<f32>) -> Self {
        val.max()
    }
}

impl Vec2<f32> {
    pub fn new(x: f32, y: f32) -> Vec2<f32> {
        Vec2::<f32> { x, y }
    }

    pub const fn zero() -> Vec2<f32> {
        Vec2::<f32> { x: 0.0, y: 0.0 }
    }

    pub fn times_twiddle_8th(self) -> Vec2<f32> {
        let scale = 0.707_106_77;
        Vec2::<f32> {
            x: (self.x + self.y) * scale,
            y: (-self.y + self.y) * scale,
        }
    }

    pub fn times_twiddle_3_8th(self) -> Vec2<f32> {
        let scale = -0.707_106_77;
        Vec2::<f32> {
            x: (self.x - self.y) * scale,
            y: (self.x + self.y) * scale,
        }
    }

    pub fn mag(self) -> f32 {
        f32::hypot(self.x, self.y)
    }

    pub fn l1_norm(self) -> f32 {
        self.x.abs() + self.y.abs()
    }

    pub fn abs(self) -> Vec2<f32> {
        Vec2::<f32> {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }

    pub fn max(self) -> f32 {
        f32::max(self.x.abs(), self.y.abs())
    }

    pub fn to_p2(self) -> Vec2<i32> {
        Vec2::<i32> {
            x: self.x as i32,
            y: self.y as i32,
        }
    }

    pub fn center(self) -> Vec2<f32> {
        Vec2::<f32> {
            x: self.x / 2.0,
            y: self.y / 2.0,
        }
    }

    pub fn normalize(self) -> Vec2<f32> {
        let r = self.mag().recip();
        Vec2::<f32> {
            x: self.x * r,
            y: self.y * r,
        }
    }
}

impl Vec2<i32> {
    pub fn new<A, B>(x: A, y: B) -> Vec2<i32>
    where
        i32: TryFrom<A> + TryFrom<B>,
    {
        Vec2::<i32> {
            x: i32::try_from(x).unwrap_or(0),
            y: i32::try_from(y).unwrap_or(0),
        }
    }

    pub fn flatten(&self, s: Vec2<i32>) -> i32 {
        self.x + self.y * s.x
    }

    pub fn to_cplx(self) -> Vec2<f32> {
        Vec2::<f32> {
            x: self.x as f32,
            y: self.y as f32,
        }
    }

    pub fn center(self) -> Vec2<i32> {
        Vec2::<i32> {
            x: self.x / 2,
            y: self.y / 2,
        }
    }
}
