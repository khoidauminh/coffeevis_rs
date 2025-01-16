#![allow(unused_variables)]

use super::{blend::Mixer, Pixel, P2};

#[derive(Copy, Clone)]
pub enum DrawParam {
    Rect { ps: P2, pe: P2 },
    RectWh { ps: P2, w: usize, h: usize },
    Line { ps: P2, pe: P2 },
    Plot { p: P2 },
    PlotIdx { i: usize },
    Fill {},
    Fade { a: u8 },
    Circle { p: P2, r: i32, f: bool },
}

pub type DrawFunction<T> = fn(&mut [T], usize, usize, T, Mixer<T>, DrawParam);

pub fn get_idx_fast(cwidth: usize, p: P2) -> usize {
    let x = p.x as u32;
    let y = p.y as u32;
    y.wrapping_mul(cwidth as u32).wrapping_add(x) as usize
}

pub fn set_pixel_by<T: Pixel>(
    canvas: &mut [T],
    _cwidth: usize,
    _cheight: usize,
    c: T,
    b: Mixer<T>,
    param: DrawParam,
) {
    let DrawParam::PlotIdx { i } = param else {
        return;
    };

    if let Some(p) = canvas.get_mut(i) {
        *p = b(*p, c);
    }
}

pub fn set_pixel_xy_by<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    cheight: usize,
    c: T,
    b: Mixer<T>,
    param: DrawParam,
) {
    let DrawParam::Plot { p } = param else { return };

    let i = get_idx_fast(cwidth, p);
    set_pixel_by(canvas, cwidth, cheight, c, b, DrawParam::PlotIdx { i });
}

pub fn draw_rect_xy_by<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    cheight: usize,
    c: T,
    b: Mixer<T>,
    param: DrawParam,
) {
    let DrawParam::Rect { ps, pe } = param else {
        return;
    };

    let [xs, ys] = [ps.x as usize, ps.y as usize];

    let [xe, ye] = [pe.x as usize, pe.y as usize];

    let xe = xe.min(cwidth);

    canvas
        .chunks_exact_mut(cwidth)
        .skip(ys)
        .take(ye.saturating_sub(ys).wrapping_add(1))
        .flat_map(|l| l.get_mut(xs..xe))
        .flatten()
        .for_each(|p| *p = b(*p, c));
}

pub fn fade<T: Pixel>(
    canvas: &mut [T],
    _cwidth: usize,
    _cheight: usize,
    c: T,
    b: Mixer<T>,
    param: DrawParam,
) {
    let DrawParam::Fade { a } = param else { return };
    canvas.iter_mut().for_each(|smp| *smp = smp.fade(255 - a));
}

pub fn fill<T: Pixel>(
    canvas: &mut [T],
    _cwidth: usize,
    _cheight: usize,
    c: T,
    _b: Mixer<T>,
    _param: DrawParam,
) {
    canvas.fill(c);
}

pub fn draw_rect_wh_by<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    cheight: usize,
    c: T,
    b: Mixer<T>,
    param: DrawParam,
) {
    let DrawParam::RectWh { ps, w, h } = param else {
        return;
    };

    let pe = P2::new(
        ps.x.wrapping_add(w as i32),
        ps.y.wrapping_add(h as i32).wrapping_sub(1),
    );

    draw_rect_xy_by(canvas, cwidth, cheight, c, b, DrawParam::Rect { ps, pe });
}

// Using Bresenham's line algorithm.
pub fn draw_line_by<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    cheight: usize,
    c: T,
    b: Mixer<T>,
    param: DrawParam,
) {
    let DrawParam::Line { ps, pe } = param else {
        return;
    };

    let dx = (pe.x - ps.x).abs();
    let sx = if ps.x < pe.x { 1 } else { -1 };
    let dy = -(pe.y - ps.y).abs();
    let sy = if ps.y < pe.y { 1 } else { -1 };
    let mut error = dx + dy;

    let mut p = ps;

    loop {
        set_pixel_xy_by(canvas, cwidth, cheight, c, b, DrawParam::Plot { p });

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

pub fn draw_cirle_by<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    cheight: usize,
    color: T,
    b: Mixer<T>,
    param: DrawParam,
) {
    let DrawParam::Circle {
        p: center,
        r: radius,
        f: filled,
    } = param
    else {
        return;
    };

    let mut t1 = radius / 16;
    let mut t2;
    let mut x = radius;
    let mut y = 0;

    let half_center = P2::new(center.x / 2, center.y / 2);

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
                let [c1, c2] = pair[0..2] else {
                    return;
                };

                let ps = P2::new(center.x + c1.0, center.y + c1.1);
                let pe = P2::new(center.x + c2.0, center.y + c2.1);

                draw_rect_xy_by(
                    canvas,
                    cwidth,
                    cheight,
                    color,
                    b,
                    DrawParam::Rect { ps, pe },
                );
            });
        } else {
            coords.iter().for_each(|c| {
                set_pixel_xy_by(
                    canvas,
                    cwidth,
                    cheight,
                    color,
                    b,
                    DrawParam::Plot {
                        p: P2::new(center.x + c.0, center.y + c.1),
                    },
                )
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
