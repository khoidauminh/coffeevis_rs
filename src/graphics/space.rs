/*struct Quaternion<T: Clone + Copy + std::marker::Copy + PartialOrd + PartialEq> {
    r: T, i: T, j: T, k: T
}

impl Quaternion {
    
}*/

trait QuaternionBounds: 
    Clone + 
    Copy + 
    std::marker::Copy + 
    PartialOrd + 
    PartialEq +
    std::ops::Add +
    std::ops::Mul + 
    std::ops::Sub 
{}

trait Quaternion<T> 
{
    fn mag(&self) -> T;
    fn dot(&self, other: Self) -> T;
    fn add(&self, other: Self) -> T;
}
/*
impl<T> Quaternion<T> for Vec<T 
where T: QuaternionBounds
{
    fn mag(&self) -> T 
    {
        self.product().powf(
    }
}
*/
impl Quaternion<f32> for &[f32] 
{
    fn mag(&self) -> f32 {
        self.iter().fold(0.0, |acc, x| acc + x*x).sqrt()
    }

    fn dot(&self, other: &[f32]) -> f32 {
        self.iter().zip(other.iter())
        .fold(0.0, |acc, (x, y)| acc + x*y)
    }
}


impl Quaternion<f32> for [f32] {}