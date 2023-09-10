use crate::{data::Program, graphics::P2};

//static mut grid: bool = false;
pub const CROSS_COL: u32 = 0xFF_44_44_44;

static CROSS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn draw_cross(prog: &mut Program) {
    if CROSS.fetch_xor(true, std::sync::atomic::Ordering::Relaxed) {
        prog.pix.draw_rect_wh(P2::new(prog.pix.width() as i32 /2, prog.pix.height() as i32 / 10), 1, prog.pix.height() - prog.pix.height() / 5, CROSS_COL);
    } else {
        prog.pix.draw_rect_wh(P2::new(prog.pix.width() as i32 / 10, prog.pix.height() as i32 /2), prog.pix.width() - prog.pix.width() / 5, 1, CROSS_COL);
    }
}
