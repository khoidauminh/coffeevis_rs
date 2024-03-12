use std::ops::*;

use super::fast;
use super::Vec2;

impl<T> Neg for Vec2<T> 
where T: Neg<Output = T> + Copy {
	type Output = Vec2<T>;
	fn neg(self) -> Vec2<T> {
		Vec2::<T> {x: -self.x, y: -self.y}
	}
}

impl<T> Add for Vec2<T>
where T: Add<Output = T> + Copy {
	type Output = Vec2<T>;
	fn add(self, other: Vec2<T>) -> Vec2<T> {
		Vec2::<T> { x: self.x + other.x, y: self.y + other.y }
	}
}

impl<T> Sub for Vec2<T>
where T: Sub<Output = T> + Copy {
	type Output = Vec2<T>;
	fn sub(self, other: Vec2<T>) -> Vec2<T> {
		Vec2::<T> { x: self.x - other.x, y: self.y - other.y }
	}
}

impl<T> Mul<Vec2<T>> for Vec2<T>
where T:  Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Copy {
	type Output = Vec2<T>;
	fn mul(self, other: Vec2<T>) -> Vec2<T> {
		/*Vec2::<T> {
			x: self.x*other.x - self.y*other.y,
			y: self.x*other.y + self.y*other.x
		}*/
		
		let k1 = other.x * (self.x + self.y);
		let k2 = self.x  * (other.y - other.x);
		let k3 = self.y  * (other.x + other.y);
		
		Vec2::<T> {
			x: k1 - k3,
			y: k1 + k2
		}
	}
}

impl<T> Mul<T> for Vec2<T> 
where T: Mul<Output = T> + Copy
{
    type Output = Vec2<T>;
    
    fn mul(self, other: T) -> Vec2<T> {
        Vec2::<T> {
            x: self.x * other,
            y: self.y * other
        }
    }
}

impl Mul<Vec2<f64>> for f64 {
    type Output = Vec2<f64>;
    
    fn mul(self, other: Vec2<f64>) -> Vec2<f64> {
        Vec2::<f64> {
            x: self * other.y,
            y: self * other.x
        }
    }
}

impl<T> Vec2<T>
where T: 
	std::default::Default + 
	Copy + 
	Mul<Output = T> + 
	Add<Output = T> +
	Neg<Output = T> +
{		
	pub fn one() -> Vec2<f64> {
		Vec2::<f64> { x: 1.0, y: 0.0 }
	}

	pub fn i() -> Vec2<f64> {
		Vec2::<f64> {x: 0.0, y: 1.0}
	}

	pub fn times_i(self) -> Vec2<T> {
		Vec2::<T> {x: -self.y, y: self.x}
	}
	
	pub fn times_minus_i(self) -> Vec2<T> {
	    Vec2::<T> {x: self.y, y: -self.x}
	}
		
	pub fn conj(self) -> Vec2<T> {
		Vec2::<T> {x: self.x, y: -self.y}
	}

	pub fn scale(&self, a: T) -> Vec2<T> {
		Vec2::<T> { x: self.x * a, y: self.y*a }
	}

	pub fn mid(&self) -> T {
		self.x + self.y
	}
}

impl From<Vec2<f64>> for f64 {
    fn from(val: Vec2<f64>) -> Self {
        val.max()
    }
}

impl Vec2<f64> {
	pub fn new(x: f64, y: f64) -> Vec2<f64> {
		Vec2::<f64> {
			x, y
		}
	}
	
	pub const fn zero() -> Vec2<f64> {
		Vec2::<f64> {
			x: 0.0,
			y: 0.0
		}
	}
	
	pub fn times_twiddle_8th(self) -> Vec2<f64> {
		let scale = 0.707_106_781_186_547_6;
		Vec2::<f64> {
			x: (  self.x + self.y) * scale,
			y: (- self.y + self.y) * scale
		}
	}
	
	pub fn times_twiddle_3_8th(self) -> Vec2<f64> {
		let scale = -0.707_106_781_186_547_6;
		Vec2::<f64> {
			x: (self.x - self.y) * scale,
			y: (self.x + self.y) * scale,
		}
	}

	pub fn mag(&self) -> f64 {
		(self.x.powi(2) + self.y.powi(2)).sqrt()
	}

	pub fn l1_norm(&self) -> f64 {
		fast::abs(self.x) + fast::abs(self.y)
	}

	pub fn abs(&self) -> Vec2<f64> {
		Vec2::<f64> {x: fast::abs(self.x), y: fast::abs(self.y)}
	}
	
	pub fn max(&self) -> f64 {
		f64::max(
			fast::abs(self.x),
			fast::abs(self.y)
		)
	}
	
	pub fn to_p2(&self) -> Vec2<i32> {
		Vec2::<i32> { x: self.x as i32, y: self.y as i32 }
	}
}


impl Vec2<i32> {
	pub fn new<A, B>(x: A, y: B) -> Vec2<i32> 
	where 
		i32: TryFrom<A> + TryFrom<B>
	{
		Vec2::<i32> {
			x: i32::try_from(x).unwrap_or(0),
			y: i32::try_from(y).unwrap_or(0)
		}
	}
	
	pub fn flatten(&self, s: Vec2<i32>) -> i32 {
		self.x + self.y*s.x
	}
	
	pub fn to_cplx(&self) -> Vec2<f64> {
		Vec2::<f64> {x: self.x as f64, y: self.y as f64}
	}
}
