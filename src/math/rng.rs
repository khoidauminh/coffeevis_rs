struct Rng {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
}

use std::{
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub struct FastU32 {
    state: u32,
}

impl FastU32 {
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> u32 {
        let mut x = self.state | 1;
        x ^= x.wrapping_shl(7);
        x ^= x.wrapping_shr(9);
        self.state = x;
        x
    }
}

pub fn random_int(bound: u32) -> u32 {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_nanos();

    t as u32 % bound
}

pub fn random_float(bound: f32) -> f32 {
    static VAR: Mutex<f32> = Mutex::new(0.2132454);

    let Ok(mut save) = VAR.try_lock() else {
        return random_int(bound as u32) as f32;
    };

    let mut a = *save;

    a = a.sin();
    a *= 12427.0;
    a %= bound;

    *save = a;

    a
}
