use crate::{
    math::Cplx,
    graphics::P2,
    audio
};

fn prepare(stream: &mut crate::audio::SampleArr, bar_num: usize, volume_scale: f64, width: usize) {
}
/*
pub const test2: crate::VisFunc = |prog, _| {
    let hh = prog.pix.height()/2;
    let hf = (prog.pix.height()/2) as f64;
    let w = prog.pix.width() as f64;

    for i in 0..prog.pix.width() {
        let x = (i as f64 / w)*0.5;

        let sin = fast::sin_norm_first_quarter(x)*10.0;

        let sin = sin as i32 + hh as i32;

        prog.pix.set_pixel(P2::new(i as i32, sin), 0xFF_FF_FF_FF);
    }
};*/

pub fn test(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let w = prog.pix.width() as i32;
    let wf = prog.pix.width() as f64;
    let h = prog.pix.height();
    let vec_size = h.next_power_of_two() << 2;
    let mut vec = vec![Cplx::zero(); vec_size];
    for i in 0..h {
        vec[i] = stream[i*2];
    }
    
    /*crate::math::fft(&mut vec);
    
    vec.iter_mut().enumerate().for_each(|(i, bin)| {
        *bin = *bin * crate::math::fast::fsqrt((i+1) as f64)*0.05;
    });*/
    
    audio::limiter(&mut vec, 0.75, 10, 0.75);
    
    prog.pix.clear();
    
    //prog.pix.draw_rect_wh(P2::new((wf*0.1) as i32, 0), 1, h, 0xFF_FF_00_00);
    //prog.pix.draw_rect_wh(P2::new((wf*0.9) as i32, 0), 1, h, 0xFF_FF_00_00);
    
    for y in 0..h {
        let smp_left = vec[y].x;
        let smp_right = vec[y].y;
        
        let (low, high) = 
            if smp_left < smp_right {
                (smp_left, smp_right)
            } else {
                (smp_right, smp_left)
            }
        ;
            
        let low = (0.5+low)*wf;
        let high = (0.5+high)*wf;
        
        let low = low as i32;
        let high = high as i32 +1;
        
        let y = y as i32;
        
        prog.pix.draw_rect_xy(P2::new(low, y), P2::new(high, y), 0xFF_FF_FF_FF); 
    }
    
    stream.rotate_left(h);
}
