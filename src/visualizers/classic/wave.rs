use crate::data::*;
use crate::graphics::P2;
use crate::math::{self, Cplx};

const PERBYTE: usize = 16; // like percent but ranges from 0..256
pub const draw_wave: crate::VisFunc = |para, stream| {
    let l = (stream.len() * PERBYTE) >> 8;
    for x in 0..para.pix.width {
        let i = l * x / para.pix.width;
        let smp = stream[i];
        
        let r: u8 = (smp.x*128.0 + 128.0) as u8;
        let b: u8 = (smp.y*128.0 + 128.0) as u8;
        let g: u8 = ((r as u16 + b as u16) / 2) as u8;
        
        para.pix.draw_rect_wh(P2::new(x as i32, 0), 1, para.pix.height, u32::from_be_bytes([0xff, r, g, b]));
    }
    
    stream.rotate_left(l >> 1);
};
