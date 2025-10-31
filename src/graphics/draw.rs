use crate::graphics::PixelBuffer;

use super::{P2, Pixel};

const COMMAND_BUFFER_INIT_CAPACITY: usize = super::MAX_WIDTH as usize;

pub fn get_idx_fast(cwidth: usize, p: P2) -> usize {
    let x = p.x.cast_unsigned();
    let y = p.y.cast_unsigned();
    y.wrapping_mul(cwidth as u32).wrapping_add(x) as usize
}

impl PixelBuffer {
    pub fn plot_i(&mut self, i: usize) {
        if let Some(p) = self.buffer.get_mut(i) {
            *p = (self.mixer)(*p, self.color);
        }
    }

    pub fn plot(&mut self, p: P2) {
        let i = get_idx_fast(self.width, p);
        self.plot_i(i);
    }

    pub fn rect_xy(&mut self, ps: P2, pe: P2) {
        let [xs, ys] = [ps.x as usize, ps.y as usize];

        let [xe, ye] = [pe.x as usize, pe.y as usize];

        let xe = xe.min(self.width);

        self.buffer
            .chunks_exact_mut(self.width)
            .skip(ys)
            .take(ye.saturating_sub(ys).wrapping_add(1))
            .flat_map(|l| l.get_mut(xs..xe))
            .flatten()
            .for_each(|p| *p = (self.mixer)(*p, self.color));
    }

    pub fn fade(&mut self, a: u8) {
        self.buffer
            .iter_mut()
            .for_each(|smp| *smp = smp.fade(255 - a));
    }

    pub fn fill(&mut self) {
        self.buffer.fill(self.color);
    }

    pub fn rect(&mut self, ps: P2, w: usize, h: usize) {
        let pe = P2::new(
            ps.x.wrapping_add(w as i32),
            ps.y.wrapping_add(h as i32).wrapping_sub(1),
        );

        self.rect_xy(ps, pe);
    }

    // Using Bresenham's line algorithm.
    pub fn line(&mut self, ps: P2, pe: P2) {
        let dx = (pe.x - ps.x).abs();
        let sx = if ps.x < pe.x { 1 } else { -1 };
        let dy = -(pe.y - ps.y).abs();
        let sy = if ps.y < pe.y { 1 } else { -1 };
        let mut error = dx + dy;

        let mut p = ps;

        loop {
            self.plot(p);

            if p.x == pe.x && p.y == pe.y {
                return;
            }

            let e2 = error * 2;

            if e2 >= dy {
                if p.x == pe.x {
                    return;
                }
                error += dy;
                p.x += sx;
            }

            if e2 <= dx {
                if p.y == pe.y {
                    return;
                }
                error += dx;
                p.y += sy;
            }
        }
    }

    pub fn circle(&mut self, center: P2, radius: i32, filled: bool) {
        let mut t1 = radius / 16;
        let mut t2;
        let mut x = radius;
        let mut y = 0;

        let mut draw_symmetric = |x: i32, y: i32| {
            let coords = [
                (-x, y),
                (x, y),
                (-x, -y),
                (x, -y),
                (-y, x),
                (y, x),
                (-y, -x),
                (y, -x),
            ];

            if filled {
                coords.chunks_exact(2).for_each(|pair| {
                    let [c1, c2, ..] = pair else {
                        return;
                    };

                    let ps = P2::new(center.x + c1.0, center.y + c1.1);
                    let pe = P2::new(center.x + c2.0, center.y + c2.1);

                    self.rect_xy(ps, pe);
                });
            } else {
                coords.iter().for_each(|c| {
                    self.plot(P2::new(center.x + c.0, center.y + c.1));
                });
            }
        };

        loop {
            draw_symmetric(x, y);

            y += 1;
            t1 += y;
            t2 = t1 - x;

            if t2 >= 0 {
                t1 = t2;
                x -= 1;
            }

            if x < y {
                break;
            }
        }
    }

    pub fn paste(&mut self, pix_pos: P2, pix_width: usize, pix_vec: &Vec<u32>) {
        let canvas_iter = self
            .buffer
            .chunks_exact_mut(self.width)
            .skip(pix_pos.y.max(0) as usize);

        let pix_iter = pix_vec
            .chunks_exact(pix_width)
            .skip((-pix_pos.y.min(0)) as usize);

        let dest_x = pix_pos.x.max(0) as usize;
        let src_x = (-pix_pos.x.min(0)) as usize;

        for (line_dest, line_src) in canvas_iter.zip(pix_iter) {
            let line_iter = line_dest.iter_mut().skip(dest_x).take(pix_width);
            let src_iter = line_src.iter().skip(src_x);

            for (p_dest, p_src) in line_iter.zip(src_iter) {
                *p_dest = (self.mixer)(*p_dest, *p_src);
            }
        }
    }
}
