use crate::data::{Program, SAMPLE_SIZE};
use crate::data::{INCREMENT, PHASE_OFFSET};
use crate::math::{self, Cplx, fast::sin_norm};
use crate::graphics::P2;

//static mut _i: usize = 0;
//static mut _sweep: usize = 0;

const c: u32 = 16720064;

struct LocalData {
    sweepi: usize,
    pong: bool,
}

static DATA: std::sync::RwLock<LocalData> = std::sync::RwLock::new(LocalData {
    sweepi: 0,
    pong: false,
});

pub fn draw_vol_sweeper(
	para: &mut crate::data::Program, 
	stream: &mut crate::audio::SampleArr
) {
    //let w = PIX_W*stream[_sweep].abs() as usize /32768;
    para.clear_pix_alpha(253);

    let w = {
        /*let sum = stream
            .iter()
            .take(SAMPLE_SIZE / 2)
            .fold(0f32, |s, &x| s + x.mag());*/
        let mut sum = 0.0;
        for i in 0..SAMPLE_SIZE/2 {
            sum += stream[i].l1_norm();
        }
        (sum / (SAMPLE_SIZE / 3) as f32 * para.VOL_SCL * para.pix.width() as f32) as usize
    };

    let color_ = (w * 255 / para.pix.width()).min(255) as u8;
    let color = u32::from_be_bytes([
        0,
        255,
        (sin_norm(color_ as f32 / 512.0) * 255.0) as u8,
        color_,
    ]);

    let mut LOCAL = DATA.write().unwrap();

    para.pix.draw_rect_wh(P2::new(2, LOCAL.sweepi as i32), para.pix.width(), 1, 0);
    para.pix.draw_rect_wh(P2::new(2, LOCAL.sweepi as i32), w, 1, color);

    crate::visualizers::classic::dash_line::draw_dash_line(para, stream, false, 0, false);

    // para.vol_sweeper.1 = math::advance_with_limit(para.vol_sweeper.1, para.pix.height());

    match (LOCAL.sweepi >= para.pix.height(), LOCAL.pong) {
        (false, true) => LOCAL.sweepi = LOCAL.sweepi.wrapping_add(1),
        (false, false) => LOCAL.sweepi = LOCAL.sweepi.wrapping_sub(1),

        (true, true) => {
            LOCAL.sweepi -= 1;
            LOCAL.pong ^= true;
        }

        (true, false) => {
            LOCAL.sweepi = 0;
            LOCAL.pong ^= true;
        }
    }
    
    stream.rotate_left(SAMPLE_SIZE >> 5);
}
