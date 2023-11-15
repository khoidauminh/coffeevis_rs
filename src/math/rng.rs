pub struct Rng {
    a: f64,
    b: f64,
    c: f64,
    d: f64
}

impl Rng {
    pub fn new(bound: f64) -> Self {
        Self {
            a: 0.0,
            b: 12321.0,
            c: 1424124.0,
            d: bound
        }
    }

    pub fn advance(&mut self) -> f64 {
        self.a = self.a.mul_add(self.b, self.c + 3.0) % self.d;
        self.b = (self.a * self.b).max(self.d);
        self.c = (self.b + self.a) % (self.b + 1.0);
        self.a
    }

    pub fn set_bound(&mut self, bound: f64) {
        self.d = bound;
    }
}
