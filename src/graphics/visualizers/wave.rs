use crate::config::*;
use crate::graphics::graphical_fn;
use crate::math;

const NO_OF_ITER: isize = 0;

pub fn discrete_haarwavelet_transform_(a: &mut Vec<(f32, f32)>) {
    let mut o = a.to_vec();

    let mut l = a.len();
    while {l /= 2; l > 1} {
        for i in 0..l {
            let sum = math::cplx_add(a[i*2], a[i*2+1]);
            let dif = math::cplx_sub(a[i*2], a[i*2+1]);
            o[i  ] = sum;
            o[l+i] = dif;
        }

        std::mem::swap(a, &mut o);
    }

    std::mem::swap(a, &mut o);
}

pub fn discrete_haarwavelet_transform_i16(a: &mut Vec<(i32, i32)>) {
    let mut l = a.len() >> 1;
	if NO_OF_ITER == 0 {return;}
    
	let n = NO_OF_ITER;
    
    while l > 0 && n != 0 {
        let mut o = a.to_vec();

        for i in 0..l {
			let i2 = 2*i;
			let i21 = i2+1;

            let s = (a[i2].0+a[i21].0, a[i2].1+a[i21].1);
            let d = (a[i2].0-a[i21].0, a[i2].1-a[i21].1);
            o[i  ] = s;
            o[l+i] = d;
        }

        l >>= 1;
		n.saturating_sub(1);

        *a = o;
    }

    //std::mem::swap(a, &mut o);
}

pub fn discrete_haarwavelet_transform_i(a: &mut Vec<(i32, i32)>) {
    let la = a.len();
    let mut l = 1;
    while l < la {
        let mut o = a.to_vec();

        for i in 0..l {
			let li = l+i;
            let i2 = i*2;

            let sum = (a[i].0.wrapping_add(a[li].0) >> 1, a[i].1.wrapping_add(a[li].1) >> 1);
            let dif = (a[i].0.wrapping_sub(a[li].0) >> 1, a[i].1.wrapping_sub(a[li].1) >> 1);
            o[i2  ] = sum;
            o[i2+1] = dif;
        }

        l <<= 1;

        *a = o;
    }

   /* for (i, smp) in a.iter_mut().enumerate() {
        *smp -= la as i16 -1 -i as i16;
    } */
}

pub fn fast_walsh_hadamard_transform(a: &mut Vec<(i32, i32)>) {
    let mut h = 1;
    let l = a.len();

    while h < l {
        let mut i = 0;
        while i < h {

            for j in i..i+h {
                let u = a[j];
                let v = a[j+h];

                a[j] = (u.0+v.0, u.1+v.1);
                a[h+j] = (u.0-v.0, u.1-v.1);
            }

            i += 2*h;
        }
        h *= 2;
    }
}

const SIZE: usize = 512;
pub fn draw_wave(pix: &mut [u32], stream: &[(f32, f32)], para: &mut Parameters) {
    let mut data_f = vec![(0i32, 0i32); SIZE];

    for i in 0..SIZE {
		let smp = stream[{para._i = math::advance_with_limit(para._i, stream.len()); para._i}];
		let smp = math::cplx_mul(smp, (32767.0, 0.0));
        data_f[i] = (smp.0 as i32, smp.1 as i32);
    }

    //fast_walsh_hadamard_transform(&mut data_f);
    discrete_haarwavelet_transform_i16(&mut data_f);
    graphical_fn::win_clear(pix);

    let hh = (para.WIN_H >> 1);
    let wh = (para.WIN_W >> 1);

    let mut oldcoord = (0i32, 0i32);

    let mut x = 0usize;
    let mut i = 0usize;
    let mut l = SIZE >> 1;
    let mut ic = 0;

    while x < para.WIN_W {
        let i = (x*SIZE/para.WIN_W);
		// i += l;
		// 
		// if i >= SIZE {
			// ic += 1;
			// i = ic;
			// l >>= 1;
		// }
// 
		// if l == 0 {
			// break;
		// }

        let scale = math::log2i(i+1) as f32;

        let mut coord = (
            (data_f[i].0 >> 8),
            (data_f[i].1 >> 8)
        );

        let r = ((128 +coord.0 as u32)) & 255;
        let b = ((128 +coord.1 as u32)) & 255;
        let g = (r+b) / 2;
        let c = (r << 16) | (g << 8)| b;

        graphical_fn::draw_rect(pix, x, 0, 1, para.WIN_H, c, para);
        //graphical_fn::draw_rect(pix, x, hh.saturating_sub(coord.1 as usize), 1, coord.1 as usize, 0x00FFFFFF, para);

        //pix[graphical_fn::flatten(x as i32, hh as i32 + coord.0, para.WIN_W, para.WIN_H)] = 0x00FFFFFF;

        /*draw_rect_xor(pix,
            (wh+oldcoord.0) as usize,
            (hh+oldcoord.1) as usize,
            (coord.0) as usize,
            (coord.0) as usize,
            0x00FFFFFF,
            para
        );

        oldcoord = coord;*/
        x += 1;
    }
}


pub fn draw_wave_(pix: &mut [u32], stream: &[(f32, f32)], para: &mut Parameters) {
    /*let mut data_f: Vec<(f32, f32)> = vec![(0f32, 0f32); SIZE];

    let mut i2 = (para._i+PHASE_OFFSET)%stream.len();
    for i in 0..SIZE {
        data_f[i].0 = stream[para._i].0;
        data_f[i].0 = stream[para._i].1;



        para._i = math::advance_with_limit(para._i, stream.len());
        i2 = math::advance_with_limit(i2, stream.len());
    }*/

    let mut data_f: Vec<(f32, f32)> = vec![(0f32, 0f32); SIZE];

    for i in 0..SIZE {
        data_f[i] = stream[{para._i = math::advance_with_limit(para._i, stream.len()); para._i}];
    }

    discrete_haarwavelet_transform_(&mut data_f);

    let mut val = vec![1f32; para.WIN_W];

    for i in 0..8 {
        let s = 1 << i;
        let e = 1 << (i+1);

        let d = e-s;

        for i in 0..val.len() {
            val[i] += math::cplx_mag(data_f[s + d * i / val.len()]);
        }
    }


    graphical_fn::win_clear(pix);
    val.iter().enumerate().for_each(|(i, h)| {
        graphical_fn::draw_rect(pix, i, 0, 1, *h as usize * para.WIN_H / SIZE * 6, 0x00FFFFFF, para);
    });
}

pub fn draw_rect_xor(
    buf: &mut [u32],
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    color: u32,
    para: &Parameters,
) {
    let m = w.min(para.WIN_W.saturating_sub(x));
    let mut start_index = x + y * para.WIN_W;
    for _ in 0..h.min(para.WIN_H.saturating_sub(y)) {
        for i in start_index..(start_index+m).min(buf.len()) {
            buf[i] ^= color;
        }
        start_index += para.WIN_W;
    }
}
