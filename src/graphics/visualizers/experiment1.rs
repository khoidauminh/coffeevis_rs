// use crate::graphics::graphical_fn::coord_to_1d;
use crate::constants;
use crate::math;
//~ fn hsv2rgb(hsv: (i16, f32, f32)) -> (u8, u8. u8) {
    //~ let c = hsv.1*hsv.2;
    //~ let x = c*(-((hsv.0 as f32 /60.0)%2.0-1.0).abs());
    //~ let m = v-c;
    
    //~ match hsv.0/60 {
        //~ 0
    //~ }
//~ } 

pub unsafe fn draw_exp1(pix: &mut [u32], stream: &[(f32, f32)]) {
    //~ let mut r1 = (0f32, 0f32);
    //~ let mut b1 = (0f32, 0f32);
    //~ let mut b2 = (0f32, 0f32);
    for i in 0..pix.len() {
        let j = i%stream.len();
        //~ let r = math::lowpass(r1, stream[j], 0.05);
        //~ let b = math::highpass(b1, stream[j], 0.05)
        pix[i] = ((stream[j].0+0.5)*256.0) as u32;
    
        //~ r1 = r;
        //~ b1 = b;
    }
}

