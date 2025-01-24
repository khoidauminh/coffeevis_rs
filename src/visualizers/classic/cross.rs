use crate::{data::Program, graphics::P2};

pub const CROSS_COL: u32 = 0xFF_44_44_44;

static CROSS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn draw_cross(prog: &mut Program) {
    let (width, height) = prog.pix.sizeu().as_tuple();

    if CROSS.fetch_xor(true, std::sync::atomic::Ordering::Relaxed) {
        prog.pix.rect_wh(
            P2::new(width / 2, height / 10),
            1,
            height - height / 5,
            CROSS_COL,
            |_, y| y,
        );
    } else {
        prog.pix.rect_wh(
            P2::new(width / 10, height / 2),
            width - width / 5,
            1,
            CROSS_COL,
            |_, y| y,
        );
    }
}
