#![allow(unused_variables)]

use super::{P2, blend::{Blend, Mixer}};

pub fn get_idx_fast(cwidth: usize, p: P2) -> usize {
	let x = p.x as usize;
	let y = p.y as usize;

	y
	.wrapping_mul(cwidth)
	.wrapping_add(x)
}

pub fn set_pixel(canvas: &mut [u32], i: usize, c: u32) {
	set_pixel_by(canvas, i, c, u32::blend);
}

pub fn set_pixel_xy(
	canvas: &mut [u32], cwidth: usize, _cheight: usize, p: P2, c: u32) {
	let i = get_idx_fast(cwidth, p);
	set_pixel(canvas, i, c);
}

pub fn set_pixel_by(canvas: &mut [u32], i: usize, c: u32, b: Mixer) {
	if let Some(p) = canvas.get_mut(i) {
		*p = b(*p, c);
	}
}

pub fn set_pixel_xy_by(canvas: &mut [u32], cwidth: usize, _cheight: usize, p: P2, c: u32, b: Mixer) {
	let i = get_idx_fast(cwidth, p);
	set_pixel_by(canvas, i, c, b);
}

pub fn draw_rect_xy_by(canvas: &mut [u32], cwidth: usize, cheight: usize, ps: P2, pe: P2, c: u32, b: Mixer) {
	// let xbound = self.width;
	// let ybound = self.height.wrapping_sub(1);

	use crate::math::ToUsize;

	let [xs, ys] = [
		usize::new(ps.x),
		usize::new(ps.y)
	];

	let [xe, ye] = [
		usize::new(pe.x),
		usize::new(pe.y)
	];

	let w = xe.min(cwidth).saturating_sub(xs);

	let i = xs + ys*cwidth;
	let iend  = (xs + ye.min(cheight) *cwidth).min(canvas.len());

	(i..=iend).step_by(cwidth).for_each(|i| {
		let iw = i.wrapping_add(w);
		for p in canvas[i..iw].iter_mut() {
			*p = b(*p, c);
		}
	});
}

pub fn draw_rect_xy(canvas: &mut [u32], cwidth: usize, cheight: usize, ps: P2, pe: P2, c: u32) {
	draw_rect_xy_by(canvas, cwidth, cheight, ps, pe, c, u32::mix);
}

pub fn fade(canvas: &mut [u32], al: u8, background: u32) {
	let mut fader  = background & 0x00_FF_FF_FF;
	fader |= (al as u32) << 24;
	canvas.iter_mut().for_each(|smp| *smp = smp.mix(fader));
}

pub fn fill(canvas: &mut [u32], c: u32) {
	canvas.fill(c);
}

pub fn draw_rect_wh(canvas: &mut [u32], cwidth: usize, cheight: usize, p: P2, w: usize, h: usize, c: u32) {
	draw_rect_wh_by(canvas, cwidth, cheight, p, w, h, c, u32::mix);
}

pub fn draw_rect_wh_by(canvas: &mut [u32], cwidth: usize, cheight: usize, p: P2, w: usize, h: usize, c: u32, b: Mixer) {
	
	let ps = p;
	let pe = P2::new(
		ps.x.wrapping_add(w as i32), 
		ps.y.wrapping_add(h as i32).wrapping_sub(1)
	);
	
	draw_rect_xy_by(canvas, cwidth, cheight, ps, pe, c, b);
}

// Using Bresenham's line algorithm.
pub fn draw_line_by(canvas: &mut [u32], cwidth: usize, cheight: usize, ps: P2, pe: P2, c: u32, b: Mixer) {
	let dx = (pe.x-ps.x).abs();
	let sx = if ps.x < pe.x {1} else {-1};
	let dy = -(pe.y-ps.y).abs();
	let sy = if ps.y < pe.y {1} else {-1};
	let mut error = dx + dy;

	let mut p = ps;

	loop {
		
		set_pixel_xy_by(canvas, cwidth, cheight, p, c, b);
		
		if p.x == pe.x && p.y == pe.y {return}
		let e2 = error*2;

		if e2 >= dy {
			if p.x == pe.x {return}
			error += dy;
			p.x += sx;
		}

		if e2 <= dx {
			if p.y == pe.y {return}
			error += dx;
			p.y += sy;
		}
	}
}
