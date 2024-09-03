#![allow(unused_variables)]

use super::{blend::Mixer, Pixel, P2};

macro_rules! make_struct {
	($i:item) => {
		#[derive(Clone)]
		$i
	}
}

make_struct!(pub struct Rect {pub ps: P2, pub pe: P2});
make_struct!(pub struct RectWh {pub ps: P2, pub w: usize, pub h: usize});
make_struct!(pub struct Line {pub ps: P2, pub pe: P2});
make_struct!(pub struct Plot {pub p: P2});
make_struct!(pub struct PlotIdx {pub i: usize});
make_struct!(pub struct Fill {});
make_struct!(pub struct Fade {pub a: u8});
make_struct!(pub struct Circle {pub p: P2, pub r: i32, pub f: bool});

macro_rules! impl_param {
	($name:ty, $func:ident) => {
		impl $name {
			pub fn exec<T: Pixel>(self, canvas: &mut[T], cwidth: usize, cheight: usize, c: T, b: Mixer<T>) {
				$func(canvas, cwidth, cheight, c, b, self);
			}
		}
	}
}

impl_param!(Rect, draw_rect_xy_by);
impl_param!(RectWh, draw_rect_wh_by);
impl_param!(Line, draw_line_by);
impl_param!(Plot, set_pixel_xy_by);
impl_param!(PlotIdx, set_pixel_by);
impl_param!(Fill, fill);
impl_param!(Fade, fade);
impl_param!(Circle, draw_cirle_by);

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
	param: PlotIdx
) {
    if let Some(p) = canvas.get_mut(param.i) {
        *p = b(*p, c);
    }
}

pub fn set_pixel_xy_by<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    cheight: usize,
    c: T,
    b: Mixer<T>,
    param: Plot
) {
    let i = get_idx_fast(cwidth, param.p);
    set_pixel_by(canvas, cwidth, cheight, c, b, PlotIdx{i});
}

pub fn draw_rect_xy_by<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    cheight: usize,
    c: T,
    b: Mixer<T>,
    param: Rect
) {
    let [xs, ys] = [param.ps.x as usize, param.ps.y as usize];

    let [xe, ye] = [param.pe.x as usize, param.pe.y as usize];

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

pub fn fade<T: Pixel>(canvas: &mut [T], _cwidth: usize, _cheight: usize, c: T, b: Mixer<T>, param: Fade) {
    let mut fader: T = c & T::from(0x00_FF_FF_FFu32);
    fader = fader | T::from((param.a as u32) << 24);
    canvas.iter_mut().for_each(|smp| *smp = smp.mix(fader));
}

pub fn fill<T: Pixel>(canvas: &mut [T], _cwidth: usize, _cheight: usize, c: T, _b: Mixer<T>, param: Fill) {
    canvas.fill(c);
}

pub fn draw_rect_wh_by<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    cheight: usize,
    c: T,
    b: Mixer<T>,
    param: RectWh
) {
    let pe = P2::new(
        param.ps.x.wrapping_add(param.w as i32),
        param.ps.y.wrapping_add(param.h as i32).wrapping_sub(1),
    );
    draw_rect_xy_by(canvas, cwidth, cheight, c, b, Rect{ps: param.ps, pe});
}

// Using Bresenham's line algorithm.
pub fn draw_line_by<T: Pixel>(
    canvas: &mut [T],
    cwidth: usize,
    cheight: usize,
    c: T,
    b: Mixer<T>,
    param: Line
) {
	let ps = param.ps;
	let pe = param.pe;
	
    let dx = (pe.x - ps.x).abs();
    let sx = if ps.x < pe.x { 1 } else { -1 };
    let dy = -(pe.y - ps.y).abs();
    let sy = if ps.y < pe.y { 1 } else { -1 };
    let mut error = dx + dy;

    let mut p = ps;

    loop {
        set_pixel_xy_by(canvas, cwidth, cheight, c, b, Plot{p});

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
    param: Circle
) {
	let center = param.p;
	let radius = param.r;
	let filled = param.f;
	
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

        for coord_pair in coords.chunks_exact(2) {
            let c1 = coord_pair[0];
            let c2 = coord_pair[1];

            if filled {
                let ps = P2::new(center.x + c1.0, center.y + c1.1);
                let pe = P2::new(center.x + c2.0, center.y + c2.1);

                draw_rect_xy_by(canvas, cwidth, cheight, color, b, Rect{ps, pe});
            } else {
                for c in [c1, c2] {
                    set_pixel_xy_by(
                        canvas,
                        cwidth,
                        cheight,
                        color,
                        b,
                        Plot{p: P2::new(center.x + c.0, center.y + c.1)}
                    );
                }
            }
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
