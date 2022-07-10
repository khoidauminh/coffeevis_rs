use crate::graphics::graphical_fn::draw_rect;
use crate::config::Parameters;

//static mut grid: bool = false;
pub const CROSS_COL: u32 = 0x00_44_44_44;

pub fn draw_cross(buf : &mut [u32], para: &mut Parameters) {
    if {para.cross ^= true; para.cross} {
        draw_rect(buf, para.WIN_W/2, para.WIN_H / 10, 2, para.WIN_H - para.WIN_H / 5, CROSS_COL, para);
    } else {
        draw_rect(buf, para.WIN_W / 10, para.WIN_H/2, para.WIN_W - para.WIN_W / 5, 2, CROSS_COL, para);
    }
}
