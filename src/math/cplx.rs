use std::ops::*;
use super::Cplx;

impl<T> Add for Cplx<T> 
where T: Add<Output = T> + std::marker::Copy
{
	type Output = Cplx<T>;
	fn add(self, other: Cplx<T>) -> Cplx<T> 
	{
		Cplx::<T> { x: self.x + other.x, y: self.y + other.y } 
	}
}

impl<T> Sub for Cplx<T>
where T: Sub<Output = T> + std::marker::Copy
{
	type Output = Cplx<T>;
	fn sub(self, other: Cplx<T>) -> Cplx<T> 
	{
		Cplx::<T> { x: self.x - other.x, y: self.y - other.y } 
	}
}

impl<T> Mul for Cplx<T>
where T:  Add<Output = T> + Sub<Output = T> + Mul<Output = T> + std::marker::Copy
{
	type Output = Cplx<T>;
	fn mul(self, other: Cplx<T>) -> Cplx<T> 
	{
		Cplx::<T> 
		{
			x: self.x*other.x - self.y*other.y,
			y: self.x*other.y + self.y*other.x
		}
	}
}
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
where T: std::default::Default + std::marker::Copy + Mul<Output = T> + Add<Output = T>
{
	pub fn new(x: T, y: T) -> Cplx<T> 
	{
		Cplx::<T> { x: x, y: y }
	} 
	
	pub fn one() -> Cplx<f32> 
	{
		Cplx::<f32> { x: 1.0, y: 0.0 }
	}
	
	pub fn scale(&self, a: T) -> Cplx<T> 
	{
		Cplx::<T> { x: self.x * a, y: self.y*a }
	}
	
	pub fn merge(&self) -> T {
		self.x + self.y
	}
}

impl Cplx<f32> 
{	
	pub fn mag(&self) -> f32 
	{
		super::fast::fsqrt(self.x.powi(2) + self.y.powi(2))
	}
	
	pub fn l1_norm(&self) -> f32
	{
		self.x.abs() + self.y.abs()
	}
	
	pub fn abs(&self) -> Cplx<f32> 
	{
		Cplx::<f32> {x: self.x.abs(), y: self.y.abs()}
	}
	
	pub const fn zero() -> Cplx<f32>
	{
		Cplx::<f32> { x: 0.0, y: 0.0 }
	}
}

impl Cplx<i32> 
{
	pub fn flatten(&self, s: Cplx<i32>) -> i32 
	{
		self.x + self.y*s.x
	}
	
	pub const fn zero() -> Cplx<i32>
	{
		Cplx::<i32> { x: 0, y: 0 }
	}
}