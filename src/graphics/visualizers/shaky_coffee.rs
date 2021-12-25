use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::io::{BufRead, BufReader};

use crate::constants::{WIN_W, WIN_H, VOL_SCL};
use crate::graphics::{graphical_fn};
use crate::math;

// default image.
use crate::assets::coffee_pixart::coffee_pixart_file;

static mut w: usize = 128;
static mut h: usize = 128;

// When this feature is finished, you can modify this image path to whatever file you like.
// Make sure that it is a P3 PPM and all pixels on every height index is on the same line.
/* Example : 

P3
5 5 
255
0 0 0 0 0 
0 0 1 0 0
0 1 1 1 0 
0 0 1 0 0 
0 0 0 0 0 
 
*/
const img : &str = "coffee_pixart_2x.ppm";

// soft shaking
static mut _i : f32 = 0.0;
const incr : f32 = 0.0001;

fn diamond_func(amp : f32, prd : f32, t : f32) -> (i32, i32) {
    (
        triangle_wav(amp, prd, t) as i32,
        triangle_wav(amp, prd, t+prd/4.0) as i32
    )
}   

fn triangle_wav(amp : f32, prd : f32, t : f32) -> f32 {
    (4.0*(t/prd - (t/prd+0.5).trunc()).abs()-1.0)*amp
}

static mut image_buffer : Vec<u8> = Vec::new();

pub unsafe fn prepare_img() {
    let path = Path::new(img);
    
    let mut f = File::open(
        path
    ).expect("Failed to open file.");
    
    let reader = BufReader::new(f);
    
    for (index, line_raw) in reader.lines().enumerate() {
        let line = line_raw.unwrap();
        
        if index == 0 || index == 2 { continue; }
        if index == 1 {
            let split :Vec<&str> = line.split(' ').collect();
            w = split[0].parse::<usize>().expect("failed to parse string.");
            h = split[1].parse::<usize>().expect("failed to parse string.");
            
            continue;
        }
        
        let split :Vec<&str> = line.split(' ').collect();
        let mut i = 0;
        
        for i in 0..split.len()-1 {
            image_buffer.push(
                split[i].parse::<u8>().expect("failed to parse string.")
            );
        }
    }
    
    //sprintln!("{} {}", w, h);
    
    //println!("{}", image_buffer.len());
}

pub unsafe fn prepare_img_default() {
    
    for (index, line) in coffee_pixart_file.lines().enumerate() {
        
        if index == 0 || index == 2 { continue; }
        if index == 1 {
            let split :Vec<&str> = line.split(' ').collect();
            w = split[0].parse::<usize>().expect("failed to parse string.");
            h = split[1].parse::<usize>().expect("failed to parse string.");
            
            continue;
        }
        
        let split :Vec<&str> = line.split(' ').collect();
        let mut i = 0;
        
        for i in 0..split.len()-1 {
            image_buffer.push(
                split[i].parse::<u8>().expect("failed to parse string.")
            );
        }
    }
    
    //sprintln!("{} {}", w, h);
    
    //println!("{}", image_buffer.len());
}

static mut x_ : i32 = 0;
static mut y_ : i32 = 0;

pub unsafe fn draw_shaky(buf : &mut Vec<u32>, stream : Vec<(f32, f32)>) {
    
    apply_coord3(&stream);
    
    let center = (WIN_W as i32 / 2 - w as i32 /2, WIN_H as i32 / 2 - h as i32 /2);

    for i in 0..(w*h) {
        let x = i%w;
        let y = i/w;
        
        let (x_soft_shake, y_soft_shake) = diamond_func(8.0, 1.0, _i);
            
        buf[graphical_fn::coord_to_1d(center.0 + x as i32 + x_ + 6 + x_soft_shake, center.1 + y as i32 + y_ + y_soft_shake)] = 
            graphical_fn::rgb_to_u32(
                image_buffer[i*3], 
                image_buffer[i*3+1], 
                image_buffer[i*3+2]
            );
    }
    
    _i = (_i + incr) % 1.0;
}

static mut xshake : f32 = 0.0;
static mut yshake : f32 = 0.0;

static mut _j : f32 = 0.0;

static mut smooth_amplitude : f32 = 0.0;
const wrapper : f32 = crate::constants::pi2*725.0;

pub unsafe fn apply_coord3(stream : &Vec<(f32, f32)>) {
	
    let mut data_f = vec![(0.0f32, 0.0f32); 1024];
    let mut amplitude : f32 = 0.0;
    
    for i in 0..1024 {
        data_f[i].0 = (stream[i.min(stream.len())].0*VOL_SCL);
    }
    
    math::hanning(&mut data_f);
    math::lowpass(&mut data_f, 0.005);
    
    for i in 0..1024 {
        amplitude += data_f[i].0.abs();
    }
    
    //smooth_amplitude = graphical_fn::linear_interp(smooth_amplitude, amplitude.min(1024.0), 0.5);
    smooth_amplitude = (amplitude.clamp(1.0, 1024.0).log2()/1.9).powi(2);
    //println!("{}", smooth_amplitude);
    
    _j = (_j + amplitude / 240.0) % wrapper;
    
    xshake = (smooth_amplitude)*(_j).cos();
    yshake = (smooth_amplitude)*(_j*0.725).sin();
    
    x_ = graphical_fn::linear_interp(x_ as f32, xshake, 0.1) as i32;
    y_ = graphical_fn::linear_interp(y_ as f32, yshake, 0.1) as i32;
    
    _j += 0.01;
}
