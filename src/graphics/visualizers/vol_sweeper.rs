use crate::config::{PHASE_OFFSET, INCREMENT};
use crate::config::Parameters;
use crate::graphics::graphical_fn::{rgb_to_u32, coord_to_1d, win_clear, win_clear_alpha, draw_line, P2, p2_add};
use crate::graphics::graphical_fn;
use crate::math;

//static mut _i : usize = 0;
//static mut _sweep : usize = 0;

const c : u32 = 16720064;

pub fn draw_vol_sweeper(buf : &mut [u32], stream : &[(f32, f32)], para: &mut Parameters ) {

    //let w = WIN_W*stream[_sweep].abs() as usize /32768;
    win_clear_alpha(buf, 0.99);

    let w = {
        let mut sum : f32 = 0.0;
        if (para.vol_sweeper.1 & 1 == 0) {
            for j in 0..stream.len()/2 {
                sum += stream[j].0.abs();
            }
        } else {
            for j in stream.len()/2..stream.len() {
                sum += stream[j].0.abs();
            }
        }
        (2.0* sum / (stream.len()/2) as f32 * para.VOL_SCL * para.WIN_W as f32) as usize
    };

    let color_ = (w*255/para.WIN_W).min(255) as u8;
    let color = rgb_to_u32(255, ((color_ as f32 / 255.0 * 3.141592).sin()*255.0) as u8, color_);
    graphical_fn::draw_rect(buf, 2, para.vol_sweeper.1, para.WIN_W, 1, 0, para);
    graphical_fn::draw_rect(buf, 2, para.vol_sweeper.1, w, 1, color, para);

    crate::graphics::visualizers::dash_line::draw_dash_line(buf, stream, false, 0, false, para);

    para.vol_sweeper.1 = math::advance_with_limit(para.vol_sweeper.1, para.WIN_H);
}
