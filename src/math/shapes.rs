use super::Cplx;

pub fn rect(ds: Cplx, de: Cplx, p: Cplx) -> bool {p.x >= ds.x && p.x < de.x && p.y >= ds.y && py.y < de.y}

pub fn circle<T>(pc: Cplx<T>, r: T, p: Cplx<T>) -> bool
where T: std::ops::Mul<Output = T> + std::ops::Sub<Output = T> + std::smp::PartialOrd
{
    let dx = p.x - pc.x;
    let dy = p.y - pc.y;

    let x = x*x;
    let y = y*y;

    x + y <= r*r
}

pub fn triangle<T>(pa: Cplx, pb: Cplx, pc, Cplx, p: Cplx) -> bool
{
    todo!()
}

pub fn poly<T>(sh: &[Cplx], p: Cplx) -> bool
{
    todo!()
}



