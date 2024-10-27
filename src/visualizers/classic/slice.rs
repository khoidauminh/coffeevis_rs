use crate::graphics::{blend::Blend, P2};
use crate::math::{cos_sin, interpolate::linearf, Cplx, TAU};
use std::cell::RefCell;
use std::f32::consts::PI;

thread_local! {
    static ANGLE: RefCell<f32> = const { RefCell::new(0.0f32) };
    static AMP: RefCell<f32> = const { RefCell::new(0.0f32) };
}

fn blend(c1: u32, c2: u32) -> u32 {
    c1.add(c2).wrapping_shl(4)
}

const GREEN: u32 = 0xFF_00_FF_00;

pub fn draw_slice(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let center = prog.pix.size().center();
    let radius = prog.pix.width().min(prog.pix.height());
    let small_radius = radius as i32 / 16;
    let big_radius = (radius as i32 / 2) * 9 / 10;
    let big_radius_f = big_radius as f32;

    let sizef = (stream.input_size() + 1) as f32;
    let bass_low = 1.0 / sizef * 0.5;
    let bass_high = 1.0 / sizef * 2.0;
    // Dear god
    let treble_low = 1.0 / sizef * 50.0;
    let treble_high = 1.0 / sizef * 100.0;

    let (sweep, high) = {
        let mut high = Cplx::zero();
        let mut bin = Cplx::zero();
        for i in 0..stream.input_size() {
            let (j, j_high) = {
                let i = i as f32;
                let t = i / sizef;
                (
                    cos_sin(i * linearf(bass_low, bass_high, t)),
                    cos_sin(i * linearf(treble_low, treble_high, t)),
                )
            };

            high += stream[i] * j_high;
            bin += stream[i] * j;
        }

        ((bin.l1_norm() / sizef * 2.).min(TAU), high.l1_norm())
    };

    let mut amp = sweep;

    AMP.with_borrow_mut(|LOCAL| {
        amp = 2.5 * (sweep - *LOCAL).max(0.0) + sweep * 0.3 + high * 0.00005;
        *LOCAL = sweep;
    });

    ANGLE.with_borrow_mut(move |LOCAL| {
        let angle = LOCAL;
        let new_angle = *angle + amp;

        let d = 1.0 / (big_radius_f * PI);

        let channel = high as u8 / 2;
        let color = u32::compose([0xFF, channel, channel, channel]);

        let mut o = *angle;
        while o < new_angle {
            let x = o.cos() * big_radius_f;
            let y = o.sin() * big_radius_f;

            let p = P2::new(center.x + x as i32, center.y + y as i32);

            prog.pix.draw_line_by(center, p, color, blend);

            o += d;
        }

        *angle = new_angle % TAU;

        prog.pix
            .draw_circle_by(center, small_radius, true, 0xFF_FF_FF_FF, u32::over);
    });

    stream.auto_rotate();
}
