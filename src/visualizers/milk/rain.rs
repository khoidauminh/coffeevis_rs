use std::cell::RefCell;

use crate::{
    audio::AudioBuffer,
    data::{DEFAULT_SIZE_WIN, Program},
    graphics::{Argb, P2, Pixel, PixelBuffer},
    math::{
        Cplx,
        interpolate::linearf,
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

const MAX_THUNDER_SEGMENTS: usize = 48;

struct Thunder {
    segments: [P2; MAX_THUNDER_SEGMENTS],
    fade: u8,
}

impl Thunder {
    const SPREAD_WIDTH: u32 = 5;
    const MAX_HEIGHT: u32 = 5;

    const DX_MAP: &[i32] = &[-1, -1, -1, -1, 0, 1, 1, 2];
    const DY_MAP: &[i32] = &[0, 1, 1, 1, 2, 5];

    pub fn generate(seed: u32, canvas_width: i32) -> Self {
        use crate::math::rng::FastU32;

        let mut segs = [P2(0, 0); MAX_THUNDER_SEGMENTS];
        let mut rng = FastU32::new(seed);

        let mut location = P2(random_int(canvas_width as u32) as i32, 0);

        segs[0] = location;

        for i in 1..MAX_THUNDER_SEGMENTS {
            let ix = rng.next() as usize % Self::DX_MAP.len();
            let iy = rng.next() as usize % Self::DY_MAP.len();

            let dx = Self::DX_MAP[ix];
            let dy = Self::DY_MAP[iy];

            location.0 += dx;
            location.1 += dy;

            segs[i] = location;
        }

        Self { segments: segs, fade: 255 }
    }

    pub fn draw(&self, canvas: &mut PixelBuffer) {
        self.segments.windows(2).for_each(|pair| {
            let p1 = pair[0];
            let p2 = pair[1];

            canvas.color(0xFF_FF_FF_FF.fade(self.fade));
            canvas.mixer(u32::add);
            canvas.line(p1, p2);
        });
    }

    pub fn fade(&mut self) {
        self.fade = self.fade.saturating_sub(10);
    }
}

impl RainDrop {
    pub const fn new(color: u32, length: u16, fall: f32, size: P2) -> Self {
        Self {
            color,
            length,
            bound_width: size.0 as u16,
            bound_height: size.1 as u16 + length,
            position: Cplx(0.0, f32::MAX),
            fall_amount: fall,
        }
    }

    pub fn randomize_start(&mut self) {
        let wf = self.bound_width as f32;
        let hf = self.bound_height as f32;
        let r = random_float(wf);
        self.position.0 = r;
        self.fall_amount = 0.5 + random_int(128) as f32 * 0.02;
        self.position.1 = -hf - random_float(hf);
    }

    pub fn set_bound(&mut self, size: P2) {
        self.bound_width = size.0 as u16;
        self.bound_height = size.1 as u16;
    }

    pub fn is_bounds_match(&self, size: P2) -> bool {
        self.bound_width == size.0 as u16 && self.bound_height == size.1 as u16
    }

    pub fn fall(&mut self, factor: f32) -> bool {
        self.position.1 += self.fall_amount * factor;

        (self.position.1 as u16) < self.bound_height
    }

    pub fn draw(&mut self, canvas: &mut PixelBuffer) {
        let _w = canvas.width();
        let _h = canvas.height();

        let mut current_length = self.length;

        if self.position.0 as usize >= _w {
            return;
        }
        let mut p = self.position.to_p2();

        while current_length > 0 && p.1 >= 0 {
            let fade = current_length * 255 / self.length;
            let fade = fade as u8;
            canvas.color(self.color.set_alpha(fade));
            canvas.mixerm();
            canvas.plot(p);
            p.1 -= 1;
            current_length -= 1;
        }
    }
}

const NUM_OF_DROPS: usize = 64;

const DEFAULT_BOUND: P2 = P2(DEFAULT_SIZE_WIN as i32, DEFAULT_SIZE_WIN as i32);

pub fn draw(prog: &mut Program, stream: &mut AudioBuffer) {
    thread_local! {
        static LIST_OF_DROPS: RefCell<[RainDrop; NUM_OF_DROPS]> =
        RefCell::new([RainDrop::new(0xFF_FF_FF_FF, 8, 0.2, DEFAULT_BOUND); NUM_OF_DROPS]);
        static THUNDER: RefCell<Thunder> = RefCell::new(Thunder::generate(0, 1));
        static OLD_VOLUME: RefCell<f32> = RefCell::new(0.0);
    }

    let mut new_volume: f32 = 0.0;
    {
        let mut y: f32 = stream.get(0).into();
        for i in 1..200 {
            y = y + 0.25 * (stream.get(i).max() - y);
            new_volume += y;
        }
    }

    let (vol1, vol2) = OLD_VOLUME.with_borrow_mut(|old| {
        let vol1 = *old;
        *old = linearf(*old, new_volume, 0.2);
        (vol1, *old)
    });

    let voldiff = vol2 - vol1;

    dbg!(voldiff>= 5.0);

    let blue = 0.7 - vol2 * 0.005;

    prog.pix.color(u32::from_be_bytes([
        0xFF,
        0,
        (119.0 * blue) as u8,
        (255.0 * blue) as u8,
    ]));
    prog.pix.fill();

    let size = prog.pix.size();

    LIST_OF_DROPS.with_borrow_mut(|list| {
        for drop in list.iter_mut() {
            let size = prog.pix.size();

            if !drop.is_bounds_match(size) {
                drop.set_bound(size);
                drop.randomize_start();
            }

            drop.draw(&mut prog.pix);

            let p = drop.fall(vol2 * 0.01);
            if !p {
                drop.randomize_start();
            }
        }
    });

    THUNDER.with_borrow_mut(|t| {
        if voldiff>= 6.7 {
            *t = Thunder::generate(random_int(1000), prog.pix.width() as i32)
        }

        t.draw(&mut prog.pix);
        t.fade();
    });

    stream.autoslide();
}
