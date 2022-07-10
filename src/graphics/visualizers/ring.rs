use crate::config::{INCREMENT, pi2};
use crate::config::Parameters;
use crate::graphics::graphical_fn::{rgb_to_u32, flatten, win_clear, win_clear_alpha, draw_line, P2, p2_add};
use crate::math::{cplx_add, cplx_mul};

//static mut para._i : usize = 0;
//static mut wrap_rate.incremeter : f32 = 0.0;

pub fn draw_ring(buf : &mut [u32], stream : &[(f32, f32)], para: &mut Parameters ) {

    let range = stream.len()*para.WAV_WIN/100;

    if range < para.WIN_H+para.WIN_W { return (); }

    let size = para.WIN_H.min(para.WIN_W) as i32;

    let width = para.WIN_W as i32;
    let height = para.WIN_H as i32;

    let width_top_h = width >> 1;
    let height_top_h = height >> 1;


    let mut di = 0;

    win_clear(buf);

    let rate = 1.0 * pi2 /* (1.5 + crate::math::fast_sin(wrap_ratepara._incremeter)) */ / range as f32;

    while di < range {

        let p = crate::math::euler_wrap(cplx_add(cplx_mul(stream[para._i % stream.len()], (para.VOL_SCL*0.25, 0.0)), (0.4, 0.4)), (di as f32 * rate));
        let x = (p.0*size as f32) as i32;
        let y = (p.1*size as f32) as i32;

        buf[flatten(x/2+width_top_h, y/2+height_top_h, para.WIN_W, para.WIN_H)] =
            rgb_to_u32(128, (x.abs()*256/size as i32 +64) as u8, (y.abs()*256/size as i32 +64) as u8);

        para._i = (para._i+INCREMENT+1) % stream.len();
        di = di+INCREMENT+1;
    }

    //~ wrap_ratepara._incremeter += 0.001;
    //~ if (wrap_ratepara._incremeter > pi2) {
        //~ wrap_ratepara._incremeter = 0.0;
    //~ }

    //crate::graphics::visualizers::cross::draw_cross(buf);
}
