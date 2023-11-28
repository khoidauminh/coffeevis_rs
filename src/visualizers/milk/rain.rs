use crate::{
    graphics::{Canvas, P2, blend::{self, Blend}},
    math::{rng::Rng}
};

pub const draw: crate::VisFunc = |prog, _stream| {
    let mut rng = Rng::new(255.0);

    prog.pix.as_mut_slice().iter_mut().for_each(|pixel| {
        let n = rng.advance();
        let r = n as u8;

        *pixel = u32::from_be_bytes([0xFF, r, r, r]);
    });
};

fn draw_rain_drop(
	canvas: &mut Canvas,
	mut p: P2,
	length: usize,
	_intensity: u8,
	color: u32
) {
	let _w = canvas.width();
	let _h = canvas.height();

	let mut current_length = length;

	let color = color.decompose();

	if !canvas.is_in_bound(p) {
		return
	}

	while current_length > 0 {

		let fade = current_length * 256 / length;
		let fade = fade as u8;
		
		let mut color_ready = color;
		color_ready[3] = blend::u8_mul(color_ready[3], fade);
		
		let color_ready = u32::compose(color_ready);
		
		canvas.set_pixel_xy_by(p, color_ready, u32::mix);
		
		if p.y == 0 {break}
		
		p.y -= 1;
		current_length -= 1;
	}
}
