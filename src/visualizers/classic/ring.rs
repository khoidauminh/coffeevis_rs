use crate::graphics::P2;
use crate::math::Cplx;

pub fn draw_ring(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let range = prog.WAV_WIN;

    let size = prog.pix.height().min(prog.pix.width()) as i32;

    let width = prog.pix.width() as i32;
    let height = prog.pix.height() as i32;

    let width_top_h = width >> 1;
    let height_top_h = height >> 1;

    prog.pix.clear();

    let rate = -1.0 / range as f64;

    let loop_range = size as usize * 2;

    for i in 1..loop_range {
        let di = i * range / loop_range;

        let smp = stream[di] - stream[di.saturating_sub(1)].scale(0.7);

        let p = (smp * Cplx::new(prog.VOL_SCL * 0.65, 0.0) + Cplx::new(0.4, 0.4))
            * crate::math::cos_sin(di as f64 * rate);
        let x = (p.x * size as f64) as i32;
        let y = (p.y * size as f64) as i32;

        let int = (smp.l1_norm() * 128.0) as u8;

        prog.pix.set_pixel_xy(
            P2::new(x / 2 + width_top_h, y / 2 + height_top_h),
            u32::from_be_bytes([
                255,
                ((128 + x.abs() * 64 / size) as u8).saturating_sub(int),
                255,
                ((128 + y.abs() * 64 / size) as u8).saturating_add(int),
            ]),
        );
    }

    stream.auto_rotate();
}
