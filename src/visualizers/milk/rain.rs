use crate::{
    graphics::{self, Canvas},
    math::{self, rng::Rng}
};

pub const draw: crate::VisFunc = |prog, stream| {
    let mut rng = Rng::new(255.0);
    
    prog.pix.as_mut_slice().iter_mut().for_each(|pixel| {
        let n = rng.advance();
        let r = n as u8;

        *pixel = u32::from_be_bytes([0xFF, r, r, r]);
    });
};

fn draw_rain_drop(canvas: &mut Canvas, x: i32, y: i32, length: usize, intensity: u8, color: u32) {
	
}
