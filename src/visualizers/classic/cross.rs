use crate::{
    data::Program,
    graphics::{P2, Pixel},
};

pub const CROSS_COL: u32 = 0xFF_44_44_44;

static CROSS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn draw_cross(prog: &mut Program) {
    let (width, height) = prog.pix.sizeu().as_tuple();

    prog.pix.color(CROSS_COL);
    prog.pix.mixer(u32::over);

    if CROSS.fetch_xor(true, std::sync::atomic::Ordering::Relaxed) {
        prog.pix.rect(
            P2::new(width / 2, height / 10),
            1,
            height - height / 5 + 1,
        );
    } else {
        prog.pix.rect(
            P2::new(width / 10, height / 2),
            width - width / 5 + 1,
            1,
        );
    }
}
