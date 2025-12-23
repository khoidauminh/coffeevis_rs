use crate::graphics::PixelBuffer;

#[allow(unused_imports)]
use super::{P2, Pixel};

pub fn get_idx_fast(cwidth: usize, p: P2) -> usize {
    let x = p.0.cast_unsigned();
    let y = p.1.cast_unsigned();
    y.wrapping_mul(cwidth as u32).wrapping_add(x) as usize
}

impl PixelBuffer {
    pub fn plot_i(&mut self, i: usize) {
        if let Some(p) = self.buffer.get_mut(i) {
            #[cfg(feature = "fast")]
            {
                *p = self.color;
            }

            #[cfg(not(feature = "fast"))]
            {
                *p = (self.mixer)(*p, self.color);
            }
        }
    }

    pub fn plot(&mut self, p: P2) {
        let i = get_idx_fast(self.width, p);
        self.plot_i(i);
    }

    #[allow(unused_mut)]
    pub fn rect_xy(&mut self, mut ps: P2, mut pe: P2) {
        #[cfg(not(feature = "fast"))]
        {
            ps.0 = ps.0.max(0);
            ps.1 = ps.1.min(self.width as i32);

            pe.0 = pe.0.max(0);
            pe.1 = pe.1.min(self.height as i32);
        }

        let [xs, ys] = [ps.0 as usize, ps.1 as usize];
        let [xe, ye] = [pe.0 as usize, pe.1 as usize];

        let xe = xe.min(self.width);

        self.buffer
            .chunks_exact_mut(self.width)
            .skip(ys)
            .take(ye.saturating_sub(ys).wrapping_add(1))
            .flat_map(|l| l.get_mut(xs..xe))
            .flatten()
            .for_each(|p| {
                #[cfg(feature = "fast")]
                {
                    *p = self.color;
                }

                #[cfg(not(feature = "fast"))]
                {
                    *p = (self.mixer)(*p, self.color);
                }
            });
    }

    #[allow(unused_variables)]
    pub fn fade(&mut self, a: u8) {
        #[cfg(not(feature = "fast"))]
        self.buffer
            .iter_mut()
            .for_each(|smp| *smp = smp.fade(255 - a));
    }

    pub fn fill(&mut self) {
        self.buffer.fill(self.color);
    }

    pub fn rect(&mut self, ps: P2, w: usize, h: usize) {
        let pe = P2(
            ps.0.wrapping_add(w as i32),
            ps.1.wrapping_add(h as i32).wrapping_sub(1),
        );

        self.rect_xy(ps, pe);
    }

    // Using Bresenham's line algorithm.
    pub fn line(&mut self, ps: P2, pe: P2) {
        let dx = (pe.0 - ps.0).abs();
        let sx = if ps.0 < pe.0 { 1 } else { -1 };
        let dy = -(pe.1 - ps.1).abs();
        let sy = if ps.1 < pe.1 { 1 } else { -1 };
        let mut error = dx + dy;

        let mut p = ps;

        loop {
            self.plot(p);

            if p.0 == pe.0 && p.1 == pe.1 {
                return;
            }

            let e2 = error * 2;

            if e2 >= dy {
                if p.0 == pe.0 {
                    return;
                }
                error += dy;
                p.0 += sx;
            }

            if e2 <= dx {
                if p.1 == pe.1 {
                    return;
                }
                error += dx;
                p.1 += sy;
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

                    let ps = P2(center.0 + c1.0, center.1 + c1.1);
                    let pe = P2(center.0 + c2.0, center.1 + c2.1);

                    self.rect_xy(ps, pe);
                });
            } else {
                coords.iter().for_each(|c| {
                    self.plot(P2(center.0 + c.0, center.1 + c.1));
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

    #[allow(dead_code)]
    pub fn paste(&mut self, pix_pos: P2, pix_width: usize, pix_vec: &[u32]) {
        let canvas_iter = self
            .buffer
            .chunks_exact_mut(self.width)
            .skip(pix_pos.1.max(0) as usize);

        let pix_iter = pix_vec
            .chunks_exact(pix_width)
            .skip((-pix_pos.1.min(0)) as usize);

        let dest_x = pix_pos.0.max(0) as usize;
        let src_x = (-pix_pos.0.min(0)) as usize;

        for (line_dest, line_src) in canvas_iter.zip(pix_iter) {
            let line_iter = line_dest.iter_mut().skip(dest_x).take(pix_width);
            let src_iter = line_src.iter().skip(src_x);

            for (p_dest, p_src) in line_iter.zip(src_iter) {
                #[cfg(feature = "fast")]
                {
                    *p_dest = *p_src;
                }

                #[cfg(not(feature = "fast"))]
                {
                    *p_dest = (self.mixer)(*p_dest, *p_src);
                }
            }
        }
    }
}
