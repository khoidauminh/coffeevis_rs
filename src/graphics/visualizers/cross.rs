use crate::graphics::graphical_fn::draw_rect;
use crate::constants::{WIN_H, WIN_W};

static mut grid: bool = false;

pub unsafe fn draw_cross(buf : &mut [u32]) {
    if {grid ^= true; grid} {
        draw_rect(buf, WIN_W/2, 8, 1, WIN_H-16, 0x00_55_55_55);
    } else {
        draw_rect(buf, 8, WIN_H/2, WIN_W-16, 1, 0x00_55_55_55);
    }
}
