use crate::data::Program;
use crate::data::{INCREMENT, PHASE_OFFSET, SAMPLE_SIZE};
use crate::graphics::{Canvas, P2};
use crate::math::Cplx;

struct LocalData {
    p0: P2,
    p1: P2,
}
static DATA: std::sync::RwLock<LocalData> = std::sync::RwLock::new(LocalData {
    p0: P2 { x: 0, y: 0 },
    p1: P2 { x: 0, y: 0 },
});

pub const draw_lazer: crate::VisFunc = |para, stream| {
    let w = para.pix.width()  as i32;
    let h = para.pix.height() as i32;

    let (ax, ay) = {
        let mut sum = Cplx::<f32>::zero();
        
        for i in 0..SAMPLE_SIZE/4 {
            sum = sum + stream[i]
        }
        
        (
            (sum.x * para.VOL_SCL as f32 * 0.2) as i32,
            (sum.y * para.VOL_SCL as f32 * 0.2 * stream[0].y.signum()) as i32,
        )
    };

    //~ pos.x = linear_interp(pos.x as f32, (pos.x+ax).clamp(0, w -1) as f32, 0.7) as i32;
    //~ pos.y = linear_interp(pos.y as f32, (pos.y+ay).clamp(0, h -1) as f32, 0.7) as i32;

    // no smoothing version
    //~ pos.x = (pos.x+ax).clamp(0, w -1);
    //~ pos.y = (pos.y+ay).clamp(0, h -1);

    let mut LOCAL = DATA.write().unwrap();

    LOCAL.p0.x = (LOCAL.p0.y + ax + w) % w;
    LOCAL.p0.y = (LOCAL.p0.x + ay + h) % h;

    let color = u32::from_be_bytes([
        0xff,
        (48 + LOCAL.p0.x * 255 / w).min(255) as u8,
        (48 + LOCAL.p0.y * 255 / h).min(255) as u8,
        ((255 - ax * ay * 2).abs().min(255)) as u8,
    ]);

    para.pix.fade(254);

    para.pix.draw_line(LOCAL.p1, LOCAL.p0, color);

    LOCAL.p1 = LOCAL.p0;
};
