#![allow(unused_variables)]

use super::{Argb, P2, Pixel, blend::Mixer};
use std::sync::Arc;

const COMMAND_BUFFER_INIT_CAPACITY: usize = super::MAX_WIDTH as usize;

#[derive(Clone, Debug)]
pub enum DrawParam {
    Rect { ps: P2, pe: P2 },
    RectWh { ps: P2, w: usize, h: usize },
    Line { ps: P2, pe: P2 },
    Plot { p: P2 },
    PlotIdx { i: usize },
    Fill {},
    Fade { a: u8 },
    Circle { p: P2, r: i32, f: bool },
    Pix { p: P2, w: usize, v: Arc<[Argb]> },
}

pub type DrawFunction = fn(&mut [Argb], usize, usize, Argb, Mixer, DrawParam);

#[derive(Debug)]
pub struct DrawCommand {
    pub func: DrawFunction,
    pub param: DrawParam,
    pub color: Argb,
    pub blending: Mixer,
}

#[derive(Debug)]
pub struct DrawCommandBuffer(Vec<DrawCommand>);

macro_rules! make_draw_func {
    ($fn_name:ident, $func:ident, $param_struct:ident $(, $param:ident: $type:ty)*) => {
        pub fn $fn_name(&mut self, $($param: $type), *, c: Argb, b: Mixer) {
           	self.0.push(DrawCommand {
          		func: $func,
          		param: DrawParam::$param_struct{ $($param), * },
          		color: c,
          		blending: b
           	});
        }
	}
}

impl DrawCommandBuffer {
    pub fn new() -> Self {
        Self(Vec::with_capacity(COMMAND_BUFFER_INIT_CAPACITY))
    }

    pub fn from(vec: Vec<DrawCommand>) -> Self {
        Self(vec)
    }

    make_draw_func!(rect, draw_rect_xy_by, Rect, ps: P2, pe: P2);
    make_draw_func!(rect_wh, draw_rect_wh_by, RectWh, ps: P2, w: usize, h: usize);
    make_draw_func!(line, draw_line_by, Line, ps: P2, pe: P2);
    make_draw_func!(plot, set_pixel_xy_by, Plot, p: P2);
    make_draw_func!(plot_index, set_pixel_by, PlotIdx, i: usize);
    make_draw_func!(circle, draw_cirle_by, Circle, p: P2, r: i32, f: bool);

    pub fn fill(&mut self, c: Argb) {
        // Discards all previous commands since this
        // fill overwrites the entire buffer.
        self.0.clear();
        self.0.push(DrawCommand {
            func: fill,
            param: DrawParam::Fill {},
            color: c,
            blending: Pixel::over,
        });
    }

    pub fn fade(&mut self, a: u8) {
        self.0.push(DrawCommand {
            func: fade,
            param: DrawParam::Fade { a },
            color: Argb::trans(),
            blending: Pixel::over,
        });
    }

    pub fn execute(&mut self, canvas: &mut [Argb], cwidth: usize, cheight: usize) {
        self.0.iter().for_each(|command| {
            (command.func)(
                canvas,
                cwidth,
                cheight,
                command.color,
                command.blending,
                command.param.clone(),
            );
        });
    }

    pub fn reset(&mut self) {
        self.0.clear();
    }
}

pub fn get_idx_fast(cwidth: usize, p: P2) -> usize {
    let x = p.x.cast_unsigned();
    let y = p.y.cast_unsigned();
    y.wrapping_mul(cwidth as u32).wrapping_add(x) as usize
}

pub fn set_pixel_by(
    canvas: &mut [Argb],
    _cwidth: usize,
    _cheight: usize,
    c: Argb,
    b: Mixer,
    param: DrawParam,
) {
    let DrawParam::PlotIdx { i } = param else {
        return;
    };

    if let Some(p) = canvas.get_mut(i) {
        *p = b(*p, c);
    }
}

pub fn set_pixel_xy_by(
    canvas: &mut [Argb],
    cwidth: usize,
    cheight: usize,
    c: Argb,
    b: Mixer,
    param: DrawParam,
) {
    let DrawParam::Plot { p } = param else { return };

    let i = get_idx_fast(cwidth, p);
    set_pixel_by(canvas, cwidth, cheight, c, b, DrawParam::PlotIdx { i });
}

pub fn draw_rect_xy_by(
    canvas: &mut [Argb],
    cwidth: usize,
    cheight: usize,
    c: Argb,
    b: Mixer,
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

pub fn fade(
    canvas: &mut [Argb],
    _cwidth: usize,
    _cheight: usize,
    c: Argb,
    b: Mixer,
    param: DrawParam,
) {
    let DrawParam::Fade { a } = param else { return };
    canvas.iter_mut().for_each(|smp| *smp = smp.fade(255 - a));
}

pub fn fill(
    canvas: &mut [Argb],
    _cwidth: usize,
    _cheight: usize,
    c: Argb,
    _b: Mixer,
    _param: DrawParam,
) {
    canvas.fill(c);
}

pub fn draw_rect_wh_by(
    canvas: &mut [Argb],
    cwidth: usize,
    cheight: usize,
    c: Argb,
    b: Mixer,
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
pub fn draw_line_by(
    canvas: &mut [Argb],
    cwidth: usize,
    cheight: usize,
    c: Argb,
    b: Mixer,
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

pub fn draw_cirle_by(
    canvas: &mut [Argb],
    cwidth: usize,
    cheight: usize,
    color: Argb,
    b: Mixer,
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

pub fn draw_pix_by(
    canvas: &mut [Argb],
    cwidth: usize,
    cheight: usize,
    color: Argb,
    b: Mixer,
    param: DrawParam,
) {
    let DrawParam::Pix {
        p: pix_pos,
        w: pix_width,
        v: pix_vec,
    } = param
    else {
        return;
    };

    let pix_height = pix_vec.len() / pix_width;

    let canvas_iter = canvas
        .chunks_exact_mut(cwidth)
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
            *p_dest = b(*p_dest, *p_src);
        }
    }
}
