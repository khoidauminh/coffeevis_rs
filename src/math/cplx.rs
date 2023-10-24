use std::ops::*;
use super::fast;
use super::Cplx;

impl<T> Neg for Cplx<T> 
where T: Neg<Output = T> + std::marker::Copy {
	type Output = Cplx<T>;
	fn neg(self) -> Cplx<T> {
		Cplx::<T> {x: -self.x, y: -self.y}
	}
}

impl<T> Add for Cplx<T>
where T: Add<Output = T> + std::marker::Copy {
	type Output = Cplx<T>;
	fn add(self, other: Cplx<T>) -> Cplx<T> {
		Cplx::<T> { x: self.x + other.x, y: self.y + other.y }
	}
}

impl<T> Sub for Cplx<T>
where T: Sub<Output = T> + std::marker::Copy {
	type Output = Cplx<T>;
	fn sub(self, other: Cplx<T>) -> Cplx<T> {
		Cplx::<T> { x: self.x - other.x, y: self.y - other.y }
	}
}

impl<T> Mul<Cplx<T>> for Cplx<T>
where T:  Add<Output = T> + Sub<Output = T> + Mul<Output = T> + std::marker::Copy {
	type Output = Cplx<T>;
	fn mul(self, other: Cplx<T>) -> Cplx<T> {
		Cplx::<T> {
			x: self.x*other.x - self.y*other.y,
			y: self.x*other.y + self.y*other.x
		}
	}
}

impl<T> Mul<T> for Cplx<T> 
where T: Mul<Output = T> + std::marker::Copy
{
    type Output = Cplx<T>;
    
    fn mul(self, other: T) -> Cplx<T> {
        Cplx::<T> {
            x: self.x * other,
            y: self.y * other
        }
    }
}

impl Mul<Cplx<f32>> for f32 {
    type Output = Cplx<f32>;
    
    fn mul(self, other: Cplx<f32>) -> Cplx<f32> {
        Cplx::<f32> {
            x: self * other.y,
            y: self * other.x
        }
    }
}

/*
impl Mul<i32> for Cplx<i32> {
    type Output = Cplx<i32>;
    
    fn mul(self, other: i32) -> Cplx<i32> {
        
    }
}*/

/*
impl<T: 'static> AddAssign for Cplx<T>
where &'static mut Cplx<T>: std::ops::Add<Cplx<T>, Output = &'static mut Cplx<T>>, T: std::marker::Copy
{
	fn add_assign(&mut self, other: Cplx<T>)
	{
		self = self + other;
	}
}

impl<T: 'static> SubAssign for Cplx<T>
where &'static mut Cplx<T>: std::ops::Sub<Cplx<T>, Output = &'static mut Cplx<T>>, T: std::marker::Copy
{
	fn sub_assign(&mut self, other: Cplx<T>)
	{
		self = self - other;
	}
}

impl<T: 'static> MulAssign for Cplx<T>
where &'static mut Cplx<T>: std::ops::Mul<Cplx<T>, Output = &'static mut Cplx<T>>, T: std::marker::Copy
{
	fn mul_assign(&mut self, other: Cplx<T>)
	{
		self = self * other;
	}
}*/

impl<T> Cplx<T>
where T: 
	std::default::Default + 
	std::marker::Copy + 
	Mul<Output = T> + 
	Add<Output = T> +
	Neg<Output = T> +
{
	pub fn new(x: T, y: T) -> Cplx<T> {
		Cplx::<T> { x: x, y: y }
	}

	pub fn one() -> Cplx<f32> {
		Cplx::<f32> { x: 1.0, y: 0.0 }
	}

	pub fn i() -> Cplx<f32> {
		Cplx::<f32> {x: 0.0, y: 1.0}
	}

	pub fn times_i(self) -> Cplx<T> {
		Cplx::<T> {x: -self.y, y: self.x}
	}
	
	pub fn conj(self) -> Cplx<T> {
		Cplx::<T> {x: self.x, y: -self.y}
	}

	pub fn scale(&self, a: T) -> Cplx<T> {
		Cplx::<T> { x: self.x * a, y: self.y*a }
	}

	pub fn mid(&self) -> T {
		self.x + self.y
	}
}

impl Into<f32> for Cplx<f32> {
    fn into(self) -> f32 {
        self.max()
    }
}

impl Cplx<f32> {
	pub fn mag(&self) -> f32 {
		(self.x.powi(2) + self.y.powi(2)).sqrt()
	}

	pub fn l1_norm(&self) -> f32 {
		fast::abs(self.x) + fast::abs(self.y)
	}

	pub fn abs(&self) -> Cplx<f32> {
		Cplx::<f32> {x: fast::abs(self.x), y: fast::abs(self.y)}
	}

	pub const fn zero() -> Cplx<f32> {
		Cplx::<f32> { x: 0.0, y: 0.0 }
	}
	
	pub fn max(&self) -> f32 {
		f32::max(
			fast::abs(self.x),
			fast::abs(self.y)
		)
	}
}

impl Cplx<i32> {
	pub fn flatten(&self, s: Cplx<i32>) -> i32 {
		self.x + self.y*s.x
	}

	pub const fn zero() -> Cplx<i32> {
		Cplx::<i32> { x: 0, y: 0 }
	}
}
