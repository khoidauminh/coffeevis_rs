use crate::{
    math::Cplx,
    graphics::P2,
    audio
};

fn prepare(_stream: &mut crate::audio::SampleArr, _bar_num: usize, _volume_scale: f64, _width: usize) {
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
    let _w = prog.pix.width() as i32;
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


use crate::math;
use crate::graphics;
use crate::data::*;
use std::thread;

const L: usize = 1 << 8;
const LL: usize = L+1;

pub fn draw_quick_sort_iter(prog: &mut Program, stream: &mut crate::audio::SampleArr) {

    static mut a: [u16; L] = [0u16; L];
    static mut stack: Vec<(usize, usize)> = Vec::new();
    static mut top: usize = 0;
 
    static mut i: usize = 0;
    static mut j: usize = 0;
 
    static mut x: u16 = 0u16;
    static mut l: usize = 0;
    static mut h: usize = 0;
 
    static mut label: u8 = 0;
    static mut finished_timeout: u64 = 0; // keeps finished fft in display for 1 second

    unsafe {match label {
        0 => {
            stack = Vec::new();

            stack.push((0, L));

            for _i in (0..L) {
                a[_i] = (stream[_i << 2].mag()*65535.0) as u16;
            }

            label = 1;
        },
        1 => {
            if stack.is_empty() {
                label = 4;
                return;
            }

            (l, h) = stack.pop().unwrap();

            if h == 0 {return};

            i = l;
            x = a[h-1];

            j = l;

            label = 2;
        },
        2 => {
            if j == h-1 {
                a.swap(i, h-1);
                label = 3;
            }

            if a[j] <= x {
                a.swap(i, j);
                i += 1;
            }
            j += 1;
        },
        3 => {
            if i+1 < h {
                stack.push((i+1, h));
            }

            if i-1 > l {
                stack.push((l, i-1));
            }

            label = 1;
        },
        _ => {
            finished_timeout += 1;
            if finished_timeout >= prog.FPS {
                finished_timeout = 0;
                label = 0;
            }
        }
    }

    //println!("{}", label);

    prog.pix.clear();
    
    for (idx, smp) in a.iter().enumerate() {
        let ix = idx *prog.pix.width() / L;
        let bar = (prog.pix.height() * *smp as usize / 65536);
        let y = prog.pix.height().saturating_sub(bar);
        let c =
            if      idx == i    {0xFF_00_00_FF}
            else if idx == j    {0xFF_FF_00_00}
            else                {0xFF_FF_FF_FF};
			prog.pix.draw_rect_wh(P2::new(ix as i32, y), 1, bar, c);
    }
}
}
