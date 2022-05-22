use crate::graphics::graphical_fn::draw_rect;
use crate::constants::Parameters;

//static mut grid: bool = false;

pub fn draw_cross(buf : &mut [u32], para: &mut Parameters) {
    if {para.cross ^= true; para.cross} {
        draw_rect(buf, para.WIN_W/2, 8, 1, para.WIN_H-16, 0x00_55_55_55, para);
    } else {
        draw_rect(buf, 8, para.WIN_H/2, para.WIN_W-16, 1, 0x00_55_55_55, para);
    }
}
