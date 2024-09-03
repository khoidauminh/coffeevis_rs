use crate::graphics::P2;
// use std::sync::atomic::{AtomicUsize, Order::Relaxed};

// static offset: AtomicUsize = AtomicUsize::new(0);

const PERBYTE: usize = 16; // like percent but ranges from 0..256
pub fn draw_wave(para: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let l = (stream.len() * PERBYTE) >> 8;
    let _random = 0usize;

    for x in 0..para.pix.width() {
        let i = l * x / para.pix.width();
        let smp = stream[i];

        let r: u8 = (smp.x * 144.0 + 128.0) as u8;
        let b: u8 = (smp.y * 144.0 + 128.0) as u8;
        let g: u8 = ((r as u16 + b as u16) / 2) as u8;

        para.pix.draw_rect_wh(
            P2::new(x as i32, 0),
            1,
            para.pix.height(),
            u32::from_be_bytes([b, r, g, b]),
        );
    }

    stream.rotate_left(l >> 1);
}

//pub fn draw_wave__(para: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    //let l = (stream.len() * PERBYTE) >> 8;
	
	//let arc = para.pix.get_arc();
	//let Ok(ref mut canvas) = arc.try_lock() else {
		//return
	//};
	
    //let mut pixel = canvas.iter_mut();
    //let mut i = 0usize;

    //stream.rotate_left(l << 1);

    //loop {
        //let smp = stream[i];

        //let r: u8 = (smp.x * 128.0 + 128.0) as u8;
        //let b: u8 = (smp.y * 128.0 + 128.0) as u8;
        //let g: u8 = ((r as u16 + b as u16) / 2) as u8;

        //for _ in 0..8 {
            //match pixel.next() {
                //Some(p) => *p = u32::from_be_bytes([0xff, r, g, b]),
                //None => return,
            //}
        //}

        //i += 1;
    //}
//}

//pub fn draw_wave_(para: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    //let l = (stream.len() * PERBYTE) >> 8;

	

    //let _pixel = para.pix.as_mut_slice().iter_mut();
    //let mut i = 0usize;
    //let mut j = 0usize;
    //let mut smp = stream[i];

    //let mut r: u8 = (smp.x * 128.0 + 128.0) as u8;
    //let mut b: u8 = (smp.y * 128.0 + 128.0) as u8;
    //let mut g: u8 = ((r as u16 + b as u16) / 2) as u8;

    //for x in 0..para.pix.width() {
        //for y in 0..para.pix.height() {
            //para.pix.set_pixel_xy(
                //crate::graphics::P2::new(x as i32, y as i32),
                //u32::from_be_bytes([0xff, r, g, b]),
            //);

            //j += 1;

            //if j == 4 {
                //j = 0;
                //i += 1;

                //smp = stream[i];

                //r = (smp.x * 128.0 + 128.0) as u8;
                //b = (smp.y * 128.0 + 128.0) as u8;
                //g = ((r as u16 + b as u16) / 2) as u8;
            //}
        //}
    //}

    //stream.rotate_left(l << 1);
//}
