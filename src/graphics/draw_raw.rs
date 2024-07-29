#![allow(unused_variables)]

use super::{blend::Mixer, Pixel, P2};

pub fn get_idx_fast(cwidth: usize, p: P2) -> usize {
    let x = p.x as usize;
    let y = p.y as usize;

    y.wrapping_mul(cwidth).wrapping_add(x)
}

pub fn set_pixel<T: Pixel>(canvas: &mut [T], i: usize, c: T) {
    set_pixel_by(canvas, i, c, T::blend);
}

pub fn set_pixel_xy<T: Pixel>(canvas: &mut [T], cwidth: usize, _cheight: usize, p: P2, c: T) {
    let i = get_idx_fast(cwidth, p);
    set_pixel(canvas, i, c);
}

#[inline]
pub fn set_pixel_by<T: Pixel>(canvas: &mut [T], i: usize, c: T, b: Mixer<T>) {
    if let Some(p) = canvas.get_mut(i) {
        *p = b(*p, c);
    }
}

pub fn set_pixel_xy_by<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    _cheight: usize,
    p: P2,
    c: T,
    b: Mixer<T>,
) {
    let i = get_idx_fast(cwidth, p);
    set_pixel_by(canvas, i, c, b);
}

pub fn draw_rect_xy_by<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    cheight: usize,
    ps: P2,
    pe: P2,
    c: T,
    b: Mixer<T>,
) {
    let [xs, ys] = [ps.x as usize, ps.y as usize];

    let [xe, ye] = [pe.x as usize, pe.y as usize];

    let xe = xe.min(cwidth);

    let lines = canvas
        .chunks_exact_mut(cwidth)
        .skip(ys)
        .take(ye.saturating_sub(ys).wrapping_add(1));

    for line in lines {
        let Some(chunk) = line.get_mut(xs..xe) else {
            return;
        };

        for p in chunk {
            *p = b(*p, c);
        }
    }
}

pub fn draw_rect_xy<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    cheight: usize,
    ps: P2,
    pe: P2,
    c: T,
) {
    draw_rect_xy_by(canvas, cwidth, cheight, ps, pe, c, T::mix);
}

pub fn fade<T: Pixel>(canvas: &mut [T], al: u8, background: T) {
    let mut fader: T = background & T::from(0x00_FF_FF_FFu32);
    fader = fader | T::from((al as u32) << 24);
    canvas.iter_mut().for_each(|smp| *smp = smp.mix(fader));
}

pub fn fill<T: Pixel>(canvas: &mut [T], c: T) {
    canvas.fill(c);
}

pub fn draw_rect_wh<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    cheight: usize,
    p: P2,
    w: usize,
    h: usize,
    c: T,
) {
    draw_rect_wh_by(canvas, cwidth, cheight, p, w, h, c, T::mix);
}

pub fn draw_rect_wh_by<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    cheight: usize,
    p: P2,
    w: usize,
    h: usize,
    c: T,
    b: Mixer<T>,
) {
    let ps = p;
    let pe = P2::new(
        ps.x.wrapping_add(w as i32),
        ps.y.wrapping_add(h as i32).wrapping_sub(1),
    );

    draw_rect_xy_by(canvas, cwidth, cheight, ps, pe, c, b);
}

// Using Bresenham's line algorithm.
pub fn draw_line_by<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    cheight: usize,
    ps: P2,
    pe: P2,
    c: T,
    b: Mixer<T>,
) {
    let dx = (pe.x - ps.x).abs();
    let sx = if ps.x < pe.x { 1 } else { -1 };
    let dy = -(pe.y - ps.y).abs();
    let sy = if ps.y < pe.y { 1 } else { -1 };
    let mut error = dx + dy;

    let mut p = ps;

    loop {
        set_pixel_xy_by(canvas, cwidth, cheight, p, c, b);

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

pub fn merge<T: Pixel>(canvas: &mut [T], canvas_other: std::sync::Arc<[T]>) {
    assert_eq!(canvas.len(), canvas_other.len());

    for (p, p2) in canvas.iter_mut().zip(canvas_other.iter()) {
        *p = T::mix(*p, *p2)
    }
}
