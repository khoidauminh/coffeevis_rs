use crate::data::SAMPLE_SIZE;
use crate::math::{Cplx, interpolate::linearf};

#[derive(Copy, Clone)]
struct LocalData {
    p0: Cplx,
    p1: Cplx,
}

thread_local! {
    static DATA: std::cell::RefCell<LocalData> = std::cell::RefCell::new(LocalData {
        p0: Cplx(1.0, 1.0),
        p1: Cplx(1.0, 1.0),
    });
}

pub fn draw_lazer(para: &mut crate::Program, stream: &mut crate::AudioBuffer) {
    let w = para.pix.width() as f32;
    let h = para.pix.height() as f32;

    let mut a = {
        let mut sum = Cplx::zero();

        let left = 0..SAMPLE_SIZE / 8;
        let right = SAMPLE_SIZE / 8..SAMPLE_SIZE / 4;

        let mut smooth = 0.0;

        for i in left {
            smooth = linearf(smooth, stream.get(i).0, 0.1);
            sum.0 += smooth;
        }

        let mut smooth = 0.0;

        for i in right {
            smooth = linearf(smooth, stream.get(i).1, 0.1);
            sum.1 += smooth;
        }

        Cplx(sum.0 * para.vol_scl * 0.0035, sum.1 * para.vol_scl * 0.0035)
    };

    let LocalData { mut p0, mut p1 } = DATA.with_borrow(|d| *d);

    a *= p0;

    p0.0 = (p0.0 + a.0 + w) % w;
    p0.1 = (p0.1 + a.1 + h) % h;

    let color = u32::from_be_bytes([
        0xff,
        (48.0 + p0.0 * 255.0 / w).min(255.0) as u8,
        (48.0 + p0.1 * 255.0 / h).min(255.0) as u8,
        ((255.0 - a.0 * a.1 * 2.0).abs().min(255.0)) as u8,
    ]);

    para.pix.fade(3);
    para.pix.color(color);
    para.pix.mixerd();
    para.pix.line(p1.to_p2(), p0.to_p2());

    p1 = p0;

    DATA.with_borrow_mut(|d| {
        d.p0 = p0;
        d.p1 = p1;
    });
}
