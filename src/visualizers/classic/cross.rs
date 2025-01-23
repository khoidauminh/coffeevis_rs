use crate::{data::Program, graphics::P2};

pub const CROSS_COL: u32 = 0xFF_44_44_44;

static CROSS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn draw_cross(prog: &mut Program) {
    if CROSS.fetch_xor(true, std::sync::atomic::Ordering::Relaxed) {
        prog.pix.command.rect_wh(
            P2::new(prog.pix.width() / 2, prog.pix.height() / 10),
            1,
            prog.pix.height() - prog.pix.height() / 5,
            CROSS_COL,
            |_, y| y,
        );
    } else {
        prog.pix.command.rect_wh(
            P2::new(prog.pix.width() / 10, prog.pix.height() / 2),
            prog.pix.width() - prog.pix.width() / 5,
            1,
            CROSS_COL,
            |_, y| y,
        );
    }
}
