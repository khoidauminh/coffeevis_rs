#[allow(warnings)]

use crate::audio::SampleArr;
use crate::math::Cplx;
use crate::data::{
	Program
};

use crate::graphics::P2;

pub fn draw_raw_fft(prog: &mut Program, stream: &mut SampleArr) {
	let window = stream.len();
	let mut data = vec![Cplx::zero(); stream.len()];
	for i in 0..window {
		data[i] = stream[i];
	}
	let norm = 1.0 / window.ilog2().pow(2) as f64;
	
	crate::math::blackmannuttall::perform_window(&mut data);
	crate::math::fft(&mut data);
	
	let w = prog.pix.width();
	let _h = prog.pix.height();
	
	prog.pix.clear();
	
	for i in 0..w {
		let sample: f64 = data[i * window / w / 4].mag();
		let sample = (sample * w as f64 * norm *(i+1).ilog2() as f64) as usize;
		
		prog.pix.draw_rect_wh(P2::new(i as i32, w.saturating_sub(sample) as i32), 1, sample, 0xFF_FF_FF_FF);
	}
	
	stream.auto_rotate();
}
