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
