use core::f32;
use std::sync::Mutex;

use crate::{
    audio::AudioBuffer,
    data::{DEFAULT_SIZE_WIN, Program},
    graphics::{P2, Pixel, PixelBuffer, blend::Argb},
    math::{
        Cplx,
        rng::{random_float, random_int},
    },
};

#[derive(Copy, Clone)]
struct RainDrop {
    color: u32,
    length: u16,
    bound_width: u16,
    bound_height: u16,
    position: Cplx,
    fall_amount: f32,
}

const MAX_THUNDER_TRAILS: usize = 8;
const MAX_THUNDER_TRAIL_LENGTH: usize = 8;

#[derive(Copy, Clone)]
struct ThunderTrail {
    positions: [P2; MAX_THUNDER_TRAIL_LENGTH],
    size: usize,
}

impl ThunderTrail {
    pub fn new(start: Option<P2>) -> Self {
        let mut positions = [P2::new(0, 0); MAX_THUNDER_TRAIL_LENGTH];
        positions[0] = start.unwrap_or(P2::new(0, 0));
        Self { positions, size: 0 }
    }

    pub fn advance(&mut self) {
        if self.size == MAX_THUNDER_TRAIL_LENGTH {
            return;
        }

        let dx = random_int(3) as i32 - 1;
        let dy = random_int(3) as i32 - 1;

        let new_position = self.positions[self.size - 1] + P2::new(dx, dy);

        self.positions[self.size] = new_position;

        self.size += 1;
    }
}

struct Thunder {
    trails: [Option<ThunderTrail>; MAX_THUNDER_TRAILS],
    color: Argb,
    split_every: u16,
}

impl Thunder {
    pub fn new(split_every: u16, color: Argb, start: P2) -> Self {
        let mut trails = [None; MAX_THUNDER_TRAILS];

        trails[0] = Some(ThunderTrail::new(Some(start)));

        Self {
            trails,
            color,
            split_every,
        }
    }

    pub fn draw(&mut self, pix: &mut PixelBuffer) {}
}

impl RainDrop {
    pub const fn new(color: u32, length: u16, fall: f32, size: P2) -> Self {
        Self {
            color,
            length,
            bound_width: size.x as u16,
            bound_height: size.y as u16 + length,
            position: Cplx {
                x: 0.0,
                y: f32::MAX,
            },
            fall_amount: fall,
        }
    }

    pub fn randomize_start(&mut self) {
        let wf = self.bound_width as f32;
        let hf = self.bound_height as f32;
        let r = random_float(wf);
        self.position.x = r;
        self.fall_amount = 0.5 + random_int(128) as f32 * 0.02;
        self.position.y = -hf - random_float(hf);
    }

    pub fn set_bound(&mut self, size: P2) {
        self.bound_width = size.x as u16;
        self.bound_height = size.y as u16;
    }

    pub fn is_bounds_match(&self, size: P2) -> bool {
        self.bound_width == size.x as u16 && self.bound_height == size.y as u16
    }

    pub fn fall(&mut self, factor: f32) -> bool {
        self.position.y += self.fall_amount * factor;

        (self.position.y as u16) < self.bound_height
    }

    pub fn draw(&mut self, canvas: &mut PixelBuffer) {
        let _w = canvas.width();
        let _h = canvas.height();

        let mut current_length = self.length;

        if self.position.x as usize >= _w {
            return;
        }
        let mut p = self.position.to_p2();

        while current_length > 0 && p.y >= 0 {
            let fade = current_length * 256 / self.length;
            let fade = fade as u8;
            canvas.plot(p, self.color.fade(fade), u32::over);
            p.y -= 1;
            current_length -= 1;
        }
    }
}

const NUM_OF_DROPS: usize = 64;

const DEFAULT_BOUND: P2 = P2 {
    x: DEFAULT_SIZE_WIN as i32,
    y: DEFAULT_SIZE_WIN as i32,
};

// static mut drop: RainDrop = RainDrop::new(0xFF_FF_FF_FF, 8, 0.2, DEFAULT_SIZE_WIN as usize, DEFAULT_SIZE_WIN as usize);

pub fn draw(prog: &mut Program, stream: &mut AudioBuffer) {
    static LIST_OF_DROPS: Mutex<[RainDrop; NUM_OF_DROPS]> =
        Mutex::new([RainDrop::new(0xFF_FF_FF_FF, 8, 0.2, DEFAULT_BOUND); NUM_OF_DROPS]);

    static OLD_VOLUME: Mutex<f32> = Mutex::new(0.0);

    let Ok(mut list) = LIST_OF_DROPS.try_lock() else {
        return;
    };

    let mut new_volume: f32 = 0.0;
    {
        let mut y: f32 = stream[0].into();
        for i in 1..200 {
            y = y + 0.25 * (stream[i].max() - y);
            new_volume += y;
        }
    }

    let mut old = OLD_VOLUME.lock().unwrap();
    *old = crate::math::interpolate::linearf(*old, new_volume, 0.2);

    let blue = 0.7 - *old * 0.005;
    prog.pix.fill(u32::from_be_bytes([
        0xFF,
        0,
        (119.0 * blue) as u8,
        (255.0 * blue) as u8,
    ]));

    let size = prog.pix.size();

    for drop in list.iter_mut() {
        let size = prog.pix.size();

        if !drop.is_bounds_match(size) {
            drop.set_bound(size);
            drop.randomize_start();
        }

        drop.draw(&mut prog.pix);

        let p = drop.fall(*old * 0.01);
        if !p {
            drop.randomize_start();
        }
    }

    stream.auto_rotate();
}
