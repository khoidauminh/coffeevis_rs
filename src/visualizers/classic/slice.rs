use crate::graphics::{P2, Pixel};
use crate::math::{Cplx, cos_sin, interpolate::linearf};
use crate::visualizers::Visualizer;
use std::f32::consts::{PI, TAU};

#[derive(Default)]
pub struct Slice {
    angle_amp: (f32, f32),
}

fn blend(c1: u32, c2: u32) -> u32 {
    c1.add(c2).wrapping_shl(4)
}

const GREEN: u32 = 0xFF_00_FF_00;

impl Visualizer for Slice {
    fn name(&self) -> &'static str {
        "Slice"
    }

    fn perform(&mut self, prog: &mut crate::data::Program, stream: &mut crate::audio::AudioBuffer) {
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

                high += stream.get(i) * j_high;
                bin += stream.get(i) * j;
            }

            ((bin.l1_norm() / sizef * 2.).min(TAU), high.l1_norm())
        };

        let mut mt = self.angle_amp;

        let amp = 2.5 * (sweep - mt.1).max(0.0) + sweep * 0.3 + high * 0.00005;
        mt.1 = sweep;

        let new_angle = mt.0 + amp;

        let d = 1.0 / (big_radius_f * PI);

        let channel = high as u8 / 2;
        let color = u32::compose([0xFF, channel, channel, channel]);

        let mut o = mt.0;
        while o < new_angle {
            let x = o.cos() * big_radius_f;
            let y = o.sin() * big_radius_f;

            let p = P2(center.0 + x as i32, center.1 + y as i32);

            prog.pix.color(color);
            prog.pix.mixer(blend);
            prog.pix.line(center, p);

            o += d;
        }

        mt.0 = new_angle % TAU;

        prog.pix.color(0xFF_FF_FF_FF);
        prog.pix.mixerd();
        prog.pix.circle(center, small_radius, true);

        stream.autoslide();

        self.angle_amp = mt;
    }
}
