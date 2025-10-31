use crate::{
    data::Program,
    graphics::{P2, Pixel},
};

pub const CROSS_COL: u32 = 0xFF_44_44_44;

static CROSS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn draw_cross(prog: &mut Program) {
    let P2(width, height) = prog.pix.size();

    prog.pix.color(CROSS_COL);
    prog.pix.mixer(u32::over);

    if CROSS.fetch_xor(true, std::sync::atomic::Ordering::Relaxed) {
        prog.pix.rect(
            P2(width / 2, height / 10),
            1,
            (height - height / 5 + 1) as usize,
        );
    } else {
        prog.pix.rect(
            P2(width / 10, height / 2),
            (width - width / 5 + 1) as usize,
            1,
        );
    }
}
