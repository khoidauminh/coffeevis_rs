use std::ops::*;

#[derive(Copy, Clone)]
struct Vec3<T>(T, T, T);

impl<T> Vec3<T>
where
    T: Copy + Add<Output = T> + Neg<Output = T> + Mul<Output = T>,
{
    pub fn new(x: T, y: T, z: T) -> Self {
        Self(x, y, z)
    }

    pub fn translate(&self, b: Self) -> Self {
        Self(self.0 + b.0, self.1 + b.1, self.2 + b.2)
    }

    pub fn negate(&self) -> Self {
        Self(-self.0, -self.1, -self.2)
    }

    pub fn offset(&self, b: Self) -> Self {
        self.translate(b.negate())
    }
}
