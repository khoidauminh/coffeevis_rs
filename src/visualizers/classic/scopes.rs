use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

use crate::data::{Program, INCREMENT, PHASE_OFFSET, SAMPLE_SIZE};
use crate::graphics::P2;
use crate::visualizers::classic::cross::{draw_cross, CROSS_COL};

use crate::math::{self, Cplx};

static LOCALI: AtomicUsize = AtomicUsize::new(0);


pub const draw_oscilloscope: crate::VisFunc = |prog, stream| {
    let l = stream.len();
    let li = l as i32;
    let range = li * prog.WAV_WIN as i32 / 100;

    let (width, height) = prog.pix.sizet();

    let width_top_h = width >> 1;
    let height_top_h = height >> 1;

    let scale = prog.pix.height as f32 * prog.VOL_SCL * 0.85;

    prog.clear_pix();
    
    let mut stream_ = stream.to_vec();
    const up_count: usize = 50;
    math::integrate_inplace(&mut stream_, up_count, true);
    
    let mut zeroi = up_count;
    /*let mut zero = 0;
    'restart: while zeroi < l {
        if stream_[zeroi].x.abs() < 1e-2 {
            zero = zeroi;
            if stream_[zeroi..(zeroi+up_count).min(l)].iter().fold(0.0, |acc, x| x.x) < 0.0 { break; }
        }
        zeroi += 1;
    }
    
    if zeroi == l {
        zeroi = zero;
    }*/

    while zeroi < stream_.len() {

		let mut smp1 = stream_[zeroi].x + stream_[zeroi].y;
		let mut smp2 = stream_[zeroi-1].x + stream_[zeroi-1].y;

		if smp1 < 0.0 && smp2 >= 0.0
		{ break }

    	zeroi += 1;
    }

    if zeroi == stream_.len() {
    	zeroi = up_count;
    }

    for di in (0..range).step_by(INCREMENT +1) {
        let x = (di * width / range);
        // let xu = x as usize;

        let i_ = (di + zeroi as i32).saturating_sub(width_top_h).rem_euclid(li); 
        let i_ = i_ as usize;

        let y1 = height_top_h + math::squish(stream[i_].x, 0.6, scale) as i32;
        let y2 = height_top_h + math::squish(stream[i_].y, 0.6, scale) as i32;

        prog.pix.set_pixel_by(P2::new(x, y1), 0xFF_55_FF_55, |a, b| { a | b });
        prog.pix.set_pixel_by(P2::new(x, y2), 0xFF_55_55_FF, |a, b| { a | b });
    }

    let li = (LOCALI.load(Relaxed) + prog.pix.width*3 / 2 + 1) % prog.pix.width;

    prog.pix.draw_rect_wh(
		P2::new(li as i32, height / 10),
        1,
        prog.pix.height - prog.pix.height / 5,
        CROSS_COL,
    );
    
    prog.pix.draw_rect_wh(P2::new(li as i32, height / 2), prog.pix.width >> 3, 1, CROSS_COL);
    
    stream.rotate_left(83);
    LOCALI.store(li, Relaxed);
};


pub const draw_vectorscope: crate::VisFunc = |prog, stream| {
    let range = stream.len() * prog.WAV_WIN / 100;
    let l = stream.len();

    let size = prog.pix.height.min(prog.pix.width) as i32;
    let sizei = size as i32;
    let scale = size as f32 * prog.VOL_SCL * 0.5;

    let (width, height) = prog.pix.sizet();

    let width_top_h = width >> 1;
    let height_top_h = height >> 1;

    let mut di = 0;
    // let mut dj = (PHASE_OFFSET / INCREMENT) % l;

    prog.clear_pix();

    // let mut smooth = Cplx::<f32>::new(stream[0].x, stream[PHASE_OFFSET % l].y);
    // let smooth_factor = 0.05f32;

    //let mut data = stream.iter().step_by(INCREMENT).map(|x| *x).collect::<Vec<_>>();
    //math::integrate_streamlace(&mut data, 10, true);
    
    let mut smoothed_sample = Cplx::<f32>::new(stream[0].x, stream[PHASE_OFFSET].y);

    while di < range {
        let sample = Cplx::<f32>::new(stream[di].x, stream[di+PHASE_OFFSET].y);
        
        //let sample = math::interpolate::linearfc(smoothed_sample, sample, 0.35);
        /*let sample = Cplx::<f32>::new(
			math::interpolate::sqrt(smoothed_sample.x, sample.x, 0.1),
			math::interpolate::sqrt(smoothed_sample.y, sample.y, 0.1)
        );*/
		
		smoothed_sample = sample;

        // smooth = crate::math::interpolate::linearfc(smooth, sample, smooth_factor);

        let x = (sample.x * scale) as i32;
        let y = (sample.y * scale) as i32;
		let amp = (x.abs() + y.abs()) * 3/2;

        prog.pix.set_pixel(
			P2::new(x + width_top_h, y + height_top_h),
			u32::from_be_bytes([
				255,
				to_color(amp, sizei),
				255, //.saturating_sub(to_color(amp, sizei),
				64
			])
        );

        di += INCREMENT;
    }

    draw_cross(prog);
    
    stream.rotate_left(range);
};

fn to_color(s: i32, size: i32) -> u8 {
    (s.abs() * 256 / size).min(255) as u8
}
