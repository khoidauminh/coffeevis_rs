use std::collections::VecDeque;

use crate::graphics::P2;
use crate::math::rng::{self, FastU32};

#[derive(Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

struct Game {
    positions: VecDeque<P2>,
    cache_direction: Direction,
    direction: Direction,
    screen: P2,
    apple: P2,
    score: usize,
    rng: FastU32,
    lose: u16,
    frame_age: u8,
    frame_max_age: u8,
}

impl Game {
    const LOSE_TIME_OUT: u16 = 1024;
    const UNIT: i32 = 2;
    const FRAME_LENGTH: u8 = 5;

    fn round(x: i32) -> i32 {
        (x / Self::UNIT) * Self::UNIT
    }

    fn gen_apple(r: &mut FastU32, screen: P2) -> P2 {
        P2(Self::round(r.next() as i32), Self::round(r.next() as i32)) % screen
    }

    fn new(screen: P2) -> Self {
        let x = screen.0 / 3;
        let y = screen.1 / 3;
        let mut rng = FastU32::new(rng::time_seed() as u32);

        let apple = Self::gen_apple(&mut rng, screen);

        Self {
            positions: VecDeque::from([P2(x, y)]),
            cache_direction: Direction::Right,
            direction: Direction::Right,
            screen,
            apple,
            rng,
            score: 0,
            lose: 0,
            frame_age: 0,
            frame_max_age: Self::FRAME_LENGTH,
        }
    }

    pub fn set_screen_size(&mut self, s: P2) {
        self.screen = s;
        self.apple %= s;
    }

    pub fn set_direction(&mut self, newdirection: Direction) {
        self.cache_direction = newdirection;
    }

    pub fn update(&mut self) {
        if self.lose != 0 {
            self.lose -= 1;
            return;
        }

        if self.frame_age < self.frame_max_age {
            self.frame_age += 1;
            return;
        }

        self.frame_age = 0;

        let mut position = self.positions.front().cloned().unwrap_or(P2(0, 0));

        match (&self.direction, &self.cache_direction) {
            (Direction::Left, Direction::Right) => {}
            (Direction::Up, Direction::Down) => {}
            (Direction::Right, Direction::Left) => {}
            (Direction::Down, Direction::Up) => {}
            (_, _) => {
                self.direction = self.cache_direction;
            }
        }

        match self.direction {
            Direction::Left => position.0 -= Self::UNIT,
            Direction::Right => position.0 += Self::UNIT,
            Direction::Up => position.1 -= Self::UNIT,
            Direction::Down => position.1 += Self::UNIT,
        }

        position %= self.screen;

        self.positions.push_front(position);

        if position == self.apple {
            self.apple = Self::gen_apple(&mut self.rng, self.screen);
            self.score += 1;

            self.frame_max_age = (Self::FRAME_LENGTH - (self.score / 10).min(5) as u8).max(1);
        } else {
            self.positions.pop_back();
        }
    }

    pub fn draw(&self, pix: &mut crate::graphics::PixelBuffer) {
        pix.clear();
        pix.mixerd();

        pix.color(0xFF_FF_FF);

        for p in self.positions.iter() {
            pix.rect(*p, Self::UNIT as usize, Self::UNIT as usize);
        }

        pix.color(0xFF_00_00);

        pix.rect(self.apple, Self::UNIT as usize, Self::UNIT as usize);

        if self.lose != 0 {
            pix.fade(128);
        }
    }
}

#[derive(Default)]
pub struct Snake {
    game: Option<Game>,
}

impl crate::visualizers::Visualizer for Snake {
    fn name(&self) -> &'static str {
        "Snake"
    }

    fn perform(
        &mut self,
        pix: &mut crate::graphics::PixelBuffer,
        keys: &crate::data::KeyInput,
        stream: &mut crate::audio::AudioBuffer,
    ) {
        let game = self.game.get_or_insert(Game::new(pix.size()));

        game.set_screen_size(pix.size());

        if keys.left {
            game.set_direction(Direction::Left);
        } else if keys.right {
            game.set_direction(Direction::Right);
        } else if keys.up {
            game.set_direction(Direction::Up);
        } else if keys.down {
            game.set_direction(Direction::Down);
        }

        game.update();
        game.draw(pix);
    }
}
