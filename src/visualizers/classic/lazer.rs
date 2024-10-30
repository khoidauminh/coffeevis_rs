use crate::data::SAMPLE_SIZE;
use crate::math::{interpolate::linearf, Cplx};

struct LocalData {
    p0: Cplx,
    p1: Cplx,
}

static DATA: std::sync::Mutex<LocalData> = std::sync::Mutex::new(LocalData {
    p0: Cplx { x: 1.0, y: 1.0 },
    p1: Cplx { x: 1.0, y: 1.0 },
});

pub fn draw_lazer(para: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let w = para.pix.width() as f32;
    let h = para.pix.height() as f32;

    let mut a = {
        let mut sum = Cplx::zero();

        let left = 0..SAMPLE_SIZE / 8;
        let right = SAMPLE_SIZE / 8..SAMPLE_SIZE / 4;

        let mut smooth = 0.0;

        for i in left {
            smooth = linearf(smooth, stream[i].x, 0.1);
            sum.x += smooth;
        }

        let mut smooth = 0.0;

        for i in right {
            smooth = linearf(smooth, stream[i].y, 0.1);
            sum.y += smooth;
        }

        Cplx::new(sum.x * para.VOL_SCL * 0.0035, sum.y * para.VOL_SCL * 0.0035)
    };

    let mut LOCAL = DATA.lock().unwrap();

    a *= LOCAL.p0;

    LOCAL.p0.x = (LOCAL.p0.x + a.x + w) % w;
    LOCAL.p0.y = (LOCAL.p0.y + a.y + h) % h;

    let color = u32::from_be_bytes([
        0xff,
        (48.0 + LOCAL.p0.x * 255.0 / w).min(255.0) as u8,
        (48.0 + LOCAL.p0.y * 255.0 / h).min(255.0) as u8,
        ((255.0 - a.x * a.y * 2.0).abs().min(255.0)) as u8,
    ]);

    para.pix.fade(3);

    para.pix
        .draw_line(LOCAL.p1.to_p2(), LOCAL.p0.to_p2(), color);

    LOCAL.p1 = LOCAL.p0;
}
