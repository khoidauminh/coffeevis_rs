use std::collections::VecDeque;

use crate::graphics::P2;
use crate::math::rng::{self, FastU32};


enum Direction {
    Left,
    Right,
    Up,
    Down,
}

struct Game {
    positions: VecDeque<P2>,
    direction: Direction,
    apple: P2,
    score: usize,
    rng: FastU32,
}

impl Game {
    fn new(screen: P2) -> Self {
        let x = screen.0 / 3;
        let y = screen.1 / 3;
        let mut rng = FastU32::new(rng::time_seed() as u32);

        let apple = P2(rng.next() as i32, rng.next() as i32) % screen;

        Self {
            positions: VecDeque::from([P2(x, y)]),
            direction: Direction::Right,
            apple,
            rng,
            score: 0,
        }
    }
}
