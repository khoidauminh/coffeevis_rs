struct Rng {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
}

use std::sync::Mutex;

pub fn random_int(bound: u32) -> u32 {
    static VARS: Mutex<[u32; 3]> = Mutex::new([131, 1242, 391]);
    let garbage = std::time::Instant::now();

    let Ok(array) = VARS.try_lock() else {
        return garbage.elapsed().as_nanos() as u32;
    };

    let [mut a, mut b, mut c] = *array;

    a = a.wrapping_mul(b) % 7919;

    b ^= a;

    c = c
        .wrapping_mul(a)
        .wrapping_add(b.wrapping_mul(c))
        .wrapping_add(a)
        .wrapping_add(b);

    a ^= c;

    if let Ok(mut v) = VARS.try_lock() {
        *v = [a, b, c];
    }

    a += garbage.elapsed().as_nanos() as u32;

    a % bound
}

pub fn faster_random_int(seed: usize, i: usize, bound: usize) -> usize {
    let num = seed.wrapping_add(i.wrapping_mul(349323)) as f32 * 0.5707962;

    let mut num = num.to_bits();
    num ^= num.wrapping_shr(32);

    num as usize % bound
}

pub fn random_float(bound: f32) -> f32 {
    static VAR: Mutex<f32> = Mutex::new(0.2132454);

    let Ok(mut a) = VAR.try_lock() else {
        return 0.0;
    };

    *a = a.sin();
    *a *= 12427.0;
    *a %= 1.5707962;
    *a % bound
}
