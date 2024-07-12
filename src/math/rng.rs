pub struct Rng {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
}

use std::sync::Mutex;
// static E: RwLock<f64> = RwLock::new(12.0);
/*
impl Rng {
    pub const fn new(bound: f64) -> Self {
        Self {
            a: 0.0,
            b: 12321.0,
            c: 1424124.0,
            d: bound
        }
    }

    pub fn advance(&mut self) -> f64 {
        let mut e = E.write().unwrap();

        self.a = *e;

        self.a = self.a.mul_add(self.b, self.c + 3.0) % self.d;
        self.b = (self.a * self.b).max(self.d);
        self.c = (self.b + self.a) % (self.b + 1.0);
        *e = self.a;
        self.a
    }

    pub fn set_bound(&mut self, bound: f64) {
        self.d = bound;
    }
}*/

pub fn random_int(bound: u32) -> u32 {
    static VARS: Mutex<(u32, u32, u32)> = Mutex::new((131, 1242, 391));
    let garbage = std::time::Instant::now();

    let (mut a, mut b, mut c) = *VARS.lock().unwrap();

    a = a.wrapping_mul(b) % 7919;

    b ^= a;

    c = c
        .wrapping_mul(a)
        .wrapping_add(b.wrapping_mul(c))
        .wrapping_add(a)
        .wrapping_add(b);

    a ^= c;

    *VARS.lock().unwrap() = (a, b, c);

    a += garbage.elapsed().as_nanos() as u32;

    a % bound
}

pub fn faster_random_int(seed: usize, i: usize, bound: usize) -> usize {
    let num = seed.wrapping_add(i.wrapping_mul(349323)) as f64 * 0.5707962;

    let mut num = num.to_bits();
    num ^= num.wrapping_shr(32);

    num as usize % bound
}

pub fn random_float(bound: f64) -> f64 {
    static VAR: Mutex<f32> = Mutex::new(0.2132454);

    let mut a = *VAR.lock().unwrap();

    a = a.sin();

    a *= 12427.0;

    *VAR.lock().unwrap() = a % 1.5707962;

    (a % bound as f32) as f64
}
