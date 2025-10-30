pub mod fast;
mod fft;
pub mod rng;
mod vec2;

use std::ops;

#[derive(PartialEq)]
pub enum Normalize {
    Yes,
    No,
}

#[derive(Copy, Clone, Debug)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

pub type Cplx = Vec2<f32>;

pub trait ToUsize<T> {
    fn new(value: T) -> Self;
}

pub const fn ideal_fft_bound(up_to: usize) -> usize {
    (up_to * 3 / 2).next_power_of_two() * 2
}

pub fn fft_stereo(a: &mut [Cplx], up_to: usize, norm: Normalize) {
    fft::compute_fft_stereo(a, up_to, norm);
}

pub fn fft(a: &mut [Cplx]) {
    let l = a.len();
    let power = fast::ilog2(l);

    fft::butterfly(a, power);
    fft::compute_fft(a);
}

pub fn increment<T>(a: T, limit: T) -> T
where
    T: ops::Add<Output = T> + std::cmp::PartialEq + From<u8>,
{
    let b = a + 1.into();
    if b == limit {
        return 0.into();
    }
    b
}

pub fn decrement<T>(a: T, limit: T) -> T
where
    T: ops::Sub<Output = T> + std::cmp::PartialEq + From<u8>,
{
    if a == T::from(0) {
        return limit - 1.into();
    }
    a - 1.into()
}

pub fn squish(x: f32, scale: f32, limit: f32) -> f32 {
    (-scale * (x.abs() + scale).recip() + 1.0) * limit * x.signum()
}

pub fn integrate_inplace(a: &mut [Cplx], factor: usize, norm: Normalize) {
    if factor < 2 {
        return;
    }

    let mut sum = Cplx::zero();
    let mut table = vec![Cplx::zero(); factor];
    let mut fi = 0;
    let mut si = 0;

    let l = a.len();

    let first_iter = factor.min(l);
    while si < first_iter {
        table[fi] = a[si];

        sum += a[si];
        a[si] = sum;

        fi += 1;
        si += 1;
    }

    fi = 0;

    let bound = l.saturating_sub(factor);
    while si < bound {
        sum -= table[fi];
        sum += a[si];
        table[fi] = a[si];

        a[si] = sum;

        fi = increment(fi, factor);
        si += 1;
    }

    while si < l {
        sum -= table[fi];
        a[si] = sum;

        si += 1;
        fi = increment(fi, factor);
    }

    if norm == Normalize::Yes {
        let div = 1.0 / factor as f32;
        let mut i = 0;
        while i < first_iter {
            a[i] = a[i].scale(1.0 / i as f32);
            i += 1
        }
        while i < bound {
            a[i] = a[i].scale(div);
            i += 1
        }
        while i < l {
            a[i] = a[i].scale(1.0 / (l - i) as f32);
            i += 1
        }
    }
}

pub fn cos_sin(x: f32) -> Cplx {
    let x = x * std::f32::consts::TAU;
    let y = x.sin_cos();
    Cplx::new(y.1, y.0)
}

pub mod interpolate {
    use super::Cplx;
    pub fn linearfc(a: Cplx, b: Cplx, t: f32) -> Cplx {
        a + (b - a).scale(t)
    }

    pub fn linearf(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    pub fn cosf(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * (0.5 - 0.5 * super::fast::cos_norm(t * 0.5))
    }

    pub fn smooth_step(a: f32, b: f32, t: f32) -> f32 {
        let t = t - 0.5;
        let t = t * (2.0 - 2.0 * t.abs()) + 0.5;
        a + (b - a) * t
    }

    pub fn nearest<T>(a: T, b: T, t: f32) -> T {
        if t < 0.5 { a } else { b }
    }

    pub fn linear_decay(prev: f32, now: f32, amount: f32) -> f32 {
        now.max(prev - amount)
    }

    pub fn decay(prev: f32, now: f32, factor: f32) -> f32 {
        now.max(prev * factor)
    }

    pub fn step(mut a: f32, b: f32, ladder_step: f32) -> f32 {
        if a < b {
            a += ladder_step;
            a.min(b)
        } else {
            a -= ladder_step;
            a.max(b)
        }
    }
}
