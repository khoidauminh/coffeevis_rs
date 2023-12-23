use super::{Canvas, P2, blend::{Blend, Mixer}};
use crate::math::{ToUsize};

impl Canvas {

	pub fn set_pixel_xy(&mut self, p: P2, c: u32) {
		let i = self.get_idx_fast(p);
		self.set_pixel(i, c);
	}

	pub fn set_pixel(&mut self, i: usize, c: u32) {
		self.set_pixel_by(i, c, u32::blend);
	}

	pub fn set_pixel_by(&mut self, i: usize, c: u32, b: Mixer) {
		if let Some(p) = self.pix.get_mut(i) {
			*p = b(*p, c);
		}
	}

	pub fn set_pixel_xy_by(&mut self, p: P2, c: u32, b: Mixer) {
		let i = self.get_idx_fast(p);
		self.set_pixel_by(i, c, b);
	}

	pub fn draw_rect_xy_by(&mut self, ps: P2, pe: P2, c: u32, b: Mixer) {
		// let xbound = self.width;
		// let ybound = self.height.wrapping_sub(1);

		let [xs, ys] = [
			usize::new(ps.x),
			usize::new(ps.y)
		];

		let [xe, ye] = [
			pe.x as usize,
			pe.y as usize
		];

		let w = xe.min(self.width).saturating_sub(xs);

		let mut i = xs + ys*self.width;
		let iend  = (xs + ye*self.width).min(self.len);

		while i <= iend {
			let iw = i.wrapping_add(w);
			for p in self.pix[i..iw].iter_mut() {
				*p = b(*p, c);
			}
			i  = i.wrapping_add(self.width);
		}
	}
	
	pub fn draw_rect_xy(&mut self, ps: P2, pe: P2, c: u32) {
		self.draw_rect_xy_by(ps, pe, c, u32::mix);
	}
	
	pub fn fade(&mut self, al: u8) {
		let mut fader  = self.background & 0x00_FF_FF_FF;
		fader |= (al as u32) << 24;
		self.pix.iter_mut().take(self.len).for_each(|smp| *smp = smp.mix(fader));
	}
	
	pub fn fill(&mut self, c: u32) {
		self.pix[0..self.len].fill(c);
	}
	
	pub fn draw_rect_wh(&mut self, p: P2, w: usize, h: usize, c: u32) {
		self.draw_rect_wh_by(p, w, h, c, u32::mix);
	}

	pub fn draw_rect_wh_by(&mut self, p: P2, w: usize, h: usize, c: u32, b: Mixer) {
		
		let ps = p;
		let pe = P2::new(
			ps.x.wrapping_add(w as i32), 
			ps.y.wrapping_add(h as i32).wrapping_sub(1)
		);
		
		self.draw_rect_xy_by(ps, pe, c, b);
	}

	// Using Bresenham's line algorithm.
	pub fn draw_line(&mut self, ps: P2, pe: P2, c: u32) {
		let dx = (pe.x-ps.x).abs();
		let sx = if ps.x < pe.x {1} else {-1};
		let dy = -(pe.y-ps.y).abs();
		let sy = if ps.y < pe.y {1} else {-1};
		let mut error = dx + dy;

		let mut p = ps;

		loop {
			
			self.set_pixel_xy(p, c);
			
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
	
	/*
	pub fn draw_circle(&mut self, _p: P2, _r: i32) {
	    todo!();
    }
    
	
    pub fn draw_image2(&mut self, img: &crate::graphics::Image, p: super::P2) {
        let cw = self.width as i32;
        let ch = self.height as i32;

        let iw = img.width as i32;
        let ih = img.height as i32;

        // image completely out of bounds
        if
        p.x >= cw ||
        p.x + iw < cw ||
        p.y >= ch ||
        p.x + ih < ch
        {return}

        for (irow, line) in
            img.as_slice()
            .chunks_exact(img.width)
            .enumerate()
        {
            let cy = irow as i32 + p.y;
            if cy >= ch {return}
            if cy < 0 {continue}
            /*
            let xstart = p.x.clamp(0, cw) as usize;
            let copy_width =
                if p.x < 0 {
                    (iw + p.x).max(0).min(cw)
                } else {
                    iw.min((cw - p.x).max(0))
                } as usize;

            let cstart = cy as usize * self.width + xstart;
            let cend   = cstart + copy_width;

            let istart = if p.x < 0 { (-p.x).max(iw) } else {0} as usize;
            let iend   = istart + copy_width;

            self.pix[cstart..cend].copy_from_slice(&line[istart..iend]);*/

            let cbase = cy as usize * self.width;

            for (ix, pixel) in line.iter().enumerate() {
                let cx = ix as i32 + p.x;
                if cx < 0 {continue}

                if cx >= cw {break}

                self.pix[cx as usize + cbase] = *pixel;
            }
        }
    }

    pub fn draw_image(&mut self, img: &crate::graphics::Image, p: super::P2, scale: i32) {
        let wc = self.width as i32;
        let hc = self.height as i32;
        let wi = img.width as i32;
        let hi = img.height as i32;

        let cl = self.sizel();
        let il = img.sizel();

        let i_center_x = wi / 2;
        let i_center_y = hi / 2;
	
        let c_center_x = wc / 2;
        let c_center_y = hc / 2;

        let size = wc.max(hc);

        for cy in 0..hc {
            let iy = (cy - c_center_y)*hi/size * 100 / scale + i_center_y - p.y;

            if iy < 0 {continue}
            if iy >= hi {return}

            let cbase = cy as usize * self.width;
            let ibase = iy as usize * img.width;

            for cx in 0..wc {
                let ix = (cx - c_center_x)*wi/size * 100 / scale + i_center_x - p.x;

                if ix >= wi {break}
                if ix < 0 {continue}

                let ci = cbase + cx as usize;
                let ii = ibase + ix as usize;

                if ci >= cl || ii >= il {break}

                /*let pixel1 = img.pixel(ii);
                let pixel2 = &mut self.pix[ci];
                if pixel1 != *pixel2 {
                    *pixel2 = pixel1;
                }*/

                self.pix[ci] = img.pixel(ii);
            }
        }
    }*/
}
