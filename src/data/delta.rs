use std::time::Instant;

pub struct Delta {
    last_call: Instant,
}

impl Delta {
    pub fn new() -> Self {
        Self {
            last_call: Instant::now(),
        }
    }

    pub fn tick(&mut self) -> f32 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_call);
        self.last_call = now;
        elapsed.as_secs_f32().clamp(0.0, 1.0)
    }
}
