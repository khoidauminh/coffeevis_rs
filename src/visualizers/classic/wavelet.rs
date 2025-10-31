use crate::{
    data::DEFAULT_ROTATE_SIZE,
    graphics::{P2, Pixel, PixelBuffer},
    math::Cplx,
};

const COPY_SIZE: usize = DEFAULT_ROTATE_SIZE;
const WT_POWER: usize = crate::data::POWER;
const WT_SIZE: usize = 1 << WT_POWER;

struct WaveletTable {
    table: [Cplx; WT_SIZE],
}

impl WaveletTable {
    pub fn init(inp: &mut crate::AudioBuffer) -> Self {
        let mut cloned = [Cplx::zero(); WT_SIZE];
        for i in 0..WT_SIZE {
            cloned[i] = inp.get(i >> 2);
        }
        inp.autoslide();
        hwt(&mut cloned);
        Self { table: cloned }
    }

    pub fn get(&self, i: usize) -> Option<Cplx> {
        if i >= WT_SIZE {
            return None;
        }
        Some(self.table[i])
    }

    pub fn translate_coord(x: usize, y: usize, w: usize, h: usize) -> usize {
        let depth = WT_POWER * y / h;
        let depth_width = 1 << depth;
        let index = depth_width * x / w;

        depth_width + index
    }

    pub fn draw(&self, canvas: &mut PixelBuffer) {
        let ch = canvas.height();
        let cw = canvas.width();
        for canvas_y in 0..ch {
            let canvas_y_rev = ch - canvas_y - 1;
            let depth = WT_POWER * canvas_y_rev / ch;
            let depth_width = 1 << depth;

            let offset = depth_width;

            for canvas_x in 0..cw {
                let index = depth_width * canvas_x / cw;

                if index >= WT_SIZE {
                    return;
                }

                let val = (self.table[offset + index].l1_norm() * 128.0) as u8;
                let color = u32::compose([val, val, val, val]);

                canvas.color(color);
                canvas.mixerm();
                canvas.plot(P2::new(canvas_x as i32, canvas_y as i32));
            }
        }
    }
}

pub fn draw_wavelet(prog: &mut crate::Program, stream: &mut crate::AudioBuffer) {
    let mut w = [Cplx::zero(); WT_SIZE];
    let l = stream.len();

    w.iter_mut().enumerate().for_each(|(i, smp)| {
        let copy_size = l >> 2;
        let start = 0;

        let inew = copy_size as f32 * i as f32 / WT_SIZE as f32;
        let ifloor = start + inew as usize;
        let iceil = start + inew.ceil() as usize;

        let t = inew.fract();
        *smp = crate::math::interpolate::linearfc(stream.get(ifloor), stream.get(iceil), t);
    });

    haar_wavelet_fast(&mut w);

    let _wl = WT_SIZE;
    let _pl = prog.pix.sizel();
    let (pw, ph) = (prog.pix.width(), prog.pix.height());

    for y in 0..ph {
        let yt = WT_SIZE as f32 * (ph - y - 1) as f32 / ph as f32;

        for x in 0..pw {
            let xt = WT_SIZE as f32 * x as f32 / pw as f32;

            let smp = wavelet_xy_interpolated(&w, Cplx::new(xt, yt), WT_POWER);

            let r = crate::math::squish(smp.x, 0.25, 255.9) as u8;
            let b = crate::math::squish(smp.y, 0.25, 255.9) as u8;
            let g = ((r as u16 + b as u16) / 2) as u8;

            let pi = y * pw + x;

            prog.pix.color(u32::from_be_bytes([b, r, g, b]));
            prog.pix.mixerd();
            prog.pix.plot_i(pi);
        }
    }

    stream.autoslide();
}

fn hwt(a: &mut [Cplx; WT_SIZE]) {
    let mut aux = [Cplx::zero(); WT_SIZE];
    let mut l = WT_SIZE / 2;

    while l > 0 {
        for i in 0..l {
            let i2 = i * 2;
            let i21 = i2 + 1;
            aux[i] = (a[i2] + a[i21]).scale(0.5);
            aux[i + l] = a[i2] - a[i21];
        }
        a[..l * 2].copy_from_slice(&aux[..l * 2]);
        l /= 2;
    }
}

fn haar_wavelet_fast(a: &mut [Cplx]) {
    let mut b = vec![Cplx::zero(); a.len()];
    let mut l = a.len() / 2;

    while l > 1 {
        for i in 0..l {
            let i2 = i * 2;
            b[i] = a[i2];
            b[i + l] = a[i2 + 1] - a[i2];
        }

        a[0..l * 2].copy_from_slice(&b[0..l * 2]);
        l >>= 1;
    }
}

// p ranges in 0..w.len()
fn wavelet_xy_interpolated(w: &[Cplx], p: Cplx, pow: usize) -> Cplx {
    let pf = pow as f32;
    let l = w.len();
    let lf = l as f32;
    let ll = l - 1;

    let idx = |x: f32, h: f32| -> f32 {
        let iy = ((1 << (h as u32)) - 1) as f32;
        let ix = iy * x / lf;
        iy + ix
    };

    let y = p.y * pf / lf;

    let i0 = idx(p.x, y.floor());
    let i2 = idx(p.x, y.ceil());

    let i1 = i0.ceil();
    let i3 = i2.ceil();

    if i0 >= lf {
        return w.last().unwrap().abs();
    }

    use crate::math::interpolate::linearfc;

    let c1 = linearfc(w[i0 as usize].abs(), w[i1 as usize].abs(), i0.fract());
    let c2 = linearfc(
        w[(i2 as usize).min(ll)].abs(),
        w[(i3 as usize).min(ll)].abs(),
        i2.fract(),
    );

    linearfc(c1, c2, y.fract())
}
