use crate::data::{INCREMENT, Program};
use crate::graphics::{self, P2};
use crate::math::Cplx;

//static mut prog._i: usize = 0;
//static mut wrap_rate.incremeter: f32 = 0.0;

pub fn draw_ring(
	prog: &mut crate::data::Program, 
	stream: &mut crate::audio::SampleArr
) {

    let range = prog.WAV_WIN;

    // if range < prog.pix.height()+prog.pix.width() { return (); }

    let size = prog.pix.height().min(prog.pix.width()) as i32;

    let width = prog.pix.width() as i32;
    let height = prog.pix.height() as i32;

    let width_top_h = width >> 1;
    let height_top_h = height >> 1;

    prog.pix.clear();

    let rate = -1.0 /* (1.5 + crate::math::fast_sin(wrap_rateprog._incremeter)) */ / range as f32;

    //let mut data = stream[..range].to_vec();
    //crate::math::highpass_array(&mut data, 0.995);
    
    let loop_range = size as usize * 2;
    
    //~ let start_smp = stream[0];
    //~ let end_smp = stream[range-1];

    for i in 1..loop_range {
        let di = i * range / loop_range;

		//~ let t = 1.0 * di as f32 / range as f32;
		//~ let shift = crate::math::interpolate::linearfc(end_smp, start_smp, t);
		
		let smp = stream[di] - stream[di.saturating_sub(1)].scale(0.7);

        let p = (smp * Cplx::<f32>::new(prog.VOL_SCL*0.65, 0.0) + Cplx::<f32>::new(0.4, 0.4)) *crate::math::cos_sin(di as f32 * rate);
        let x = (p.x*size as f32) as i32;
        let y = (p.y*size as f32) as i32;
	
		let int = (smp.l1_norm()*128.0) as u8;

        prog.pix.set_pixel_xy(
			P2::new(x/2+width_top_h, y/2+height_top_h),
            u32::from_be_bytes([
				255, 
				((128 + x.abs()*64/size as i32) as u8).saturating_sub(int), 
				255,
				((128 + y.abs()*64/size as i32) as u8).saturating_add(int)
			]),
		);
    }

    //~ wrap_rateprog._incremeter += 0.001;
    //~ if (wrap_rateprog._incremeter > pi2) {
        //~ wrap_rateprog._incremeter = 0.0;
    //~ }

    //crate::graphics::visualizers::cross::draw_cross(buf);

    stream.auto_rotate();
}
