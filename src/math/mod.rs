pub mod fast;
mod fft;
pub mod rng;
mod vec2;

use std::ops;

pub use std::f64::consts::FRAC_PI_2 as PIH;
pub use std::f64::consts::TAU;

pub const TAU_RECIP: f64 = 1.0 / TAU;
pub const ZERO: Cplx = Cplx { x: 0.0, y: 0.0 };

#[derive(Copy, Clone, Debug, Default)]
pub struct Vec2<T: Copy + Clone> {
    pub x: T,
    pub y: T,
}

pub type Cplx = Vec2<f64>;

pub trait ToUsize<T> {
    fn new(value: T) -> Self;
}

pub const fn ideal_fft_bound(up_to: usize) -> usize {
    (up_to * 3 / 2).next_power_of_two() * 2
}

pub fn larger_or_equal_pw2(x: usize) -> usize {
    if x.is_power_of_two() {
        x
    } else {
        x.next_power_of_two()
    }
}

pub fn fft_stereo(a: &mut [Cplx], up_to: usize, norm: bool) {
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

pub fn squish(x: f64, scale: f64, limit: f64) -> f64 {
    (-scale * (x.abs() + scale).recip() + 1.0) * limit * x.signum()
}

pub fn integrate_inplace(a: &mut [Cplx], factor: usize, norm: bool) {
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

        sum = sum + a[si];
        a[si] = sum;

        fi += 1;
        si += 1;
    }

    fi = 0;

    let bound = l.saturating_sub(factor);
    while si < bound {
        sum = sum - table[fi];
        sum = sum + a[si];
        table[fi] = a[si];

        a[si] = sum;

        fi = increment(fi, factor);
        si += 1;
    }

    while si < l {
        sum = sum - table[fi];
        a[si] = sum;

        si += 1;
        fi = increment(fi, factor);
    }

    if norm {
        let div = 1.0 / factor as f64;
        let mut i = 0;
        while i < first_iter {
            a[i] = a[i].scale(1.0 / i as f64);
            i += 1
        }
        while i < bound {
            a[i] = a[i].scale(div);
            i += 1
        }
        while i < l {
            a[i] = a[i].scale(1.0 / (l - i) as f64);
            i += 1
        }
    }
}

pub fn cos_sin(x: f64) -> Cplx {
    let x = x * std::f64::consts::TAU;
    let y = x.sin_cos();
    Cplx::new(y.1, y.0)
}

pub mod interpolate {
    use super::Cplx;
    pub fn linearfc(a: Cplx, b: Cplx, t: f64) -> Cplx {
        a + (b - a).scale(t)
    }

    pub fn linearf(a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }

    pub fn cosf(a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * (0.5 - 0.5 * super::fast::cos_norm(t * 0.5))
    }

    pub fn smooth_step(a: f64, b: f64, t: f64) -> f64 {
        // a + (b-a)*(t*t*(3.0-2.0*t))
        let t = t - 0.5;
        let t = t * (2.0 - 2.0 * t.abs()) + 0.5;

        a + (b - a) * t
    }

    pub fn nearest<T>(a: T, b: T, t: f64) -> T {
        if t < 0.5 {
            return a;
        }
        b
    }

    pub fn subtractive_fall(prev: f64, now: f64, min: f64, amount: f64) -> f64 {
        //(now - (prev/now.max(min))*amount).max(min)
        if now > prev {
            return now;
        }
        let new = prev - amount;
        if new < min {
            return min;
        }
        if new < now {
            return now;
        }
        new
    }

    pub fn multiplicative_fall(prev: f64, now: f64, min: f64, factor: f64) -> f64 {
        if now > prev {
            return now;
        }
        let new = prev * (1.0 - factor);
        if new < min {
            return min;
        }
        if new < now {
            return now;
        }
        new
    }

    pub fn step(mut a: f64, b: f64, ladder_step: f64) -> f64 {
        if a < b {
            a += ladder_step;
            a.min(b)
        } else {
            a -= ladder_step;
            a.max(b)
        }
    }
}
