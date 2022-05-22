use crate::constants::{Parameters, Image};
use crate::graphics::{graphical_fn};
use crate::math;

// soft shaking
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

pub fn prepare_img(file: &[u8]) -> Image {
    use std::fs::File;
	use std::io::Read;
	use std::path::Path;
	use std::io::{BufRead, BufReader};
	
    let mut img = Image {
		w: 0,
		h: 0,
		image_buffer: Vec::new()
	};
    let mut image_buffer: Vec<u8> = Vec::new();

    let reader = BufReader::new(file);

    for (index, line_raw) in reader.lines().enumerate() {
        let line = line_raw.unwrap();

        if index == 0 || index == 2 { continue; }
        if index == 1 {
            let split :Vec<&str> = line.split(' ').collect();
            img.w = split[0].parse::<usize>().expect("failed to parse string.");
            img.h = split[1].parse::<usize>().expect("failed to parse string.");
		
            continue;
        }

        let split :Vec<&str> = line.split(' ').collect();
        let mut i = 0;

        for i in 0..split.len()-1 {
            img.image_buffer.push(
                split[i].parse::<u8>().expect("failed to parse string.")
            );
        }
    }

	return img;
}
/*
pub unsafe fn prepare_img_default() {
//	use crate::assets::includer::coffee_pixart_file;

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
*/

//~ static mut x_ : i32 = 0;
//~ static mut y_ : i32 = 0;

pub fn draw_shaky(buf : &mut [u32], stream : &[(f32, f32)], para: &mut Parameters ) {

    apply_coord3(stream, para);

    let center = (para.WIN_W as i32 / 2 - para.IMG.w as i32 /2, para.WIN_H as i32 / 2 - para.IMG.h as i32 /2);
    
    let (x_soft_shake, y_soft_shake) = diamond_func(8.0, 1.0, para.shaky_coffee.0);
    
	for cx in 0..para.WIN_W {
		for cy in 0..para.WIN_H {
			let ix = cx*para.IMG.w/para.WIN_W;
			let iy = cy*para.IMG.h/para.WIN_H;
			
			let i = graphical_fn::flatten(ix as i32, iy as i32, para.IMG.w, para.IMG.h);
			let c = graphical_fn::flatten(cx as i32 + x_soft_shake + para.shaky_coffee.4, cy as i32 + y_soft_shake + para.shaky_coffee.5, para.WIN_W, para.WIN_H);
			
			let i3 = i*3;
			
			buf[c] = 
				graphical_fn::rgb_to_u32(
                para.IMG.image_buffer[i3],
                para.IMG.image_buffer[i3+1],
                para.IMG.image_buffer[i3+2]
            );
		}
	}

    para.shaky_coffee.0 = (para.shaky_coffee.0 + incr) % 1.0;
}

//~ static mut xshake : f32 = 0.0;
//~ static mut yshake : f32 = 0.0;

//~ static mut _j : f32 = 0.0;

const wrapper : f32 = crate::constants::pi2*725.0;

pub fn apply_coord3(stream : &[(f32, f32)], para: &mut Parameters) {
	
	let _j = &mut para.shaky_coffee.1;
	let xshake = &mut para.shaky_coffee.2;
	let yshake = &mut para.shaky_coffee.3;
	let x_ = &mut para.shaky_coffee.4;
	let y_ = &mut para.shaky_coffee.5;
	
    let mut data_f = [(0.0f32, 0.0f32); 1024];
    let mut amplitude : f32 = 0.0;
    
    for i in 0..1024 {
        data_f[i].0 = (stream[i.min(stream.len())].0*para.VOL_SCL);
    }
    
    math::hanning(&mut data_f);
    math::lowpass_array(&mut data_f, 0.005);
    
    for i in 0..1024 {
        amplitude += data_f[i].0.abs();
    }
    
    //smooth_amplitude = graphical_fn::linear_interp(smooth_amplitude, amplitude.min(1024.0), 0.5);
    let smooth_amplitude = (amplitude.clamp(1.0, 1024.0).log2()/1.9).powi(2);
    //println!("{}", smooth_amplitude);
    
    *_j = (*_j + amplitude / 240.0) % wrapper;
    
    *xshake = (smooth_amplitude)*(*_j).cos();
    *yshake = (smooth_amplitude)*(*_j*0.725).sin();
    
    *x_ = graphical_fn::linear_interp((*x_) as f32, (*xshake), 0.1) as i32;
    *y_ = graphical_fn::linear_interp((*y_) as f32, (*yshake), 0.1) as i32;
    
    *_j += 0.01;
}
