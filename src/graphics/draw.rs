use super::{Canvas, P2};
use crate::math::Cplx;
impl Canvas {
	pub fn draw_rect_xy(&mut self, ps: P2, pe: P2, c: u32) {
		let [xs, ys] = [ps.x.max(0) as usize, ps.y.max(0) as usize];
		let [xe, ye] = [
			(pe.x as usize).min(self.width),
			(pe.y as usize).min(self.height)
		];

		let w = xe.saturating_sub(xs);
		
		let l = self.pix.len();

		for y in ys..=ye {
			let i = xs + y*self.width;
			if let Some(chunk) = self.pix.get_mut(i..i+w) {
			    chunk.fill(c)
			}
		}
	}

	pub fn draw_rect_wh(&mut self, p: P2, w: usize, h: usize, c: u32) {
		let [xs, ys] = [p.x.max(0) as usize, p.y.max(0) as usize];

		let ye = (ys+h.saturating_sub(if p.y < 0 {-p.y as usize} else {0})).min(self.height);
		// let ye = if p.y < 0 {ye.saturating_sub((-p.y) as usize)} else {ye};
		let wi = w.min(
		    self.width.saturating_sub(p.x.abs() as usize)
	    );

		for y in ys..ye {
			let i = xs + y*self.width;
			self.pix[i..(i+wi)].fill(c);
		}
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
			self.set_pixel(p, c);
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

	pub fn draw_circle(&mut self, p: P2, r: i32) {
	    todo!();
    }

	/*pub fn draw_image(&mut self, img: &super::Image, p: super::P2)
	{
		let cw = self.width as i32;
		let ch = self.height as i32;

		let iw = img.width as i32;
		let ih = img.height as i32;

		let xstart = p.x.max(0).max(cw-1) as usize;
		let xend   = (iw + p.x).max(0).max(cw) as usize;

		let rstart = p.y.max(0).max(ch-1) as usize;
		let rend   = (ih - p.y).max(0).max(ch) as usize;

		let img_slice  = img.as_slice();
		let img_xstart = (-p.x).max(0).max(iw) as usize;
		let img_xend   = (cw - p.x.max(0)).min(iw) as usize;

		for row in rstart..rend {
            let ibase  = row * self.width;
            let istart = ibase + xstart;
            let iend   = ibase + xend;

            let img_ibase = (row - rstart)*img.width;
            let img_ibase_start = img_ibase + img_xstart;
            let img_ibase_end   = img_ibase + img_xend;

            self.pix[istart..iend].copy_from_slice(&img_slice[img_xstart..img_xend]);
		}
	}*/
    /*
	pub fn draw_image(&mut self, img: &super::Image, p: super::P2) {
        let cw = self.width as i32;
		let ch = self.height as i32;

		let iw = img.width as i32;
		let ih = img.height as i32;

        let canvas_x_start = p.x.clamp(0, cw) as usize;
//        let canvas_x_end   = ((img.width as i32 + p.x).max(0) as usize).max(self.width);
        let canvas_y_start = p.y.clamp(0, ch) as usize;
//        let canvas_y_end   = ((img.height as i32 + p.y).max(0) as usize).max(self.height);

        let img_x_start = if p.x < 0 { (-p.x).max(iw) as usize } else { 0 };
        let img_y_start = if p.y < 0 { (-p.y).max(ih) as usize } else { 0 };

        let copy_width =
            (if p.x < 0 {
                (iw + p.x).max(0)
            } else {
                iw.min((cw - p.x).max(0))
            }).min(cw) as usize;

        let copy_height =
            (if p.y < 0 {
                (ih + p.y).max(0)
            } else {
                ih.min((ch - p.y).max(0))
            }).min(ch) as usize;

        let img_slice = img.as_slice();

        for y in 0..copy_height {
            let cbase = (y + canvas_y_start)*self.width;
            let ibase = (y + img_y_start)*img.width;

            let cstart = cbase + canvas_x_start;
            let istart = ibase + img_x_start;

            let cend   = cstart + copy_width;
            let iend   = istart + copy_width;

            // if cend > self.sizel() || iend > img.sizel() {continue}

            self.pix[cstart..cend]
            .copy_from_slice(
                &img_slice[istart..iend]
            );
        }
    }*/

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
    }
}
