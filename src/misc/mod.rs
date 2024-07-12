#![allow(dead_code)]

pub mod stackvec;

#[cfg(test)]
mod tests {
    use crate::math::fast;
    use std::ops::Range;
    use std::time::Instant;

    const range: Range<i32> = -10_000_000..10_000_000;
    const modulo: i32 = 211;
    const modulo_recip: f32 = 1. / modulo as f32;

    #[test]
    fn trig_sin() {
        let mut a = 0.0;
        let mut j = 1.0;
        let now = Instant::now();
        for _i in range {
            j *= 0.5;
            a = fast::sin_norm(j);
        }
        println!("{} {}", a, now.elapsed().as_micros());
    }

    #[test]
    fn trig_cos() {
        let mut a = 0.0;
        let mut j = 1.0;
        let now = Instant::now();
        for _i in range {
            j *= 0.5;
            a = fast::cos_norm(j);
        }
        println!("{} {}", a, now.elapsed().as_micros());
    }
}

fn randomizer(i: i32, range: i32) -> i32 {
    use std::sync::atomic::AtomicI32;
    use std::sync::atomic::Ordering::Relaxed;

    static RAND: AtomicI32 = AtomicI32::new(0);

    let a = i.wrapping_mul(7919).wrapping_shl(7);
    let b = a.wrapping_mul(1333).wrapping_shl(2);
    let c = i;

    let d = a ^ b | c;

    let mut r = RAND.load(Relaxed);

    r = r.wrapping_add(d);

    RAND.store(r, Relaxed);

    r % range
}
