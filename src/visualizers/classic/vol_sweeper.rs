use crate::data::SAMPLE_SIZE;
use crate::math::{fast::sin_norm};
use crate::graphics::P2;

//static mut _i: usize = 0;
//static mut _sweep: usize = 0;

const C: u32 = 16720064;

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
    para.pix.fade(3);

    let w = {
        /*let sum = stream
            .iter()
            .take(SAMPLE_SIZE / 2)
            .fold(0f64, |s, &x| s + x.mag());*/
        let mut sum = 0.0;
        for i in 0..SAMPLE_SIZE/2 {
            sum += stream[i].l1_norm();
        }
        (sum / (SAMPLE_SIZE / 3) as f64 * para.VOL_SCL * para.pix.width() as f64) as usize
    };

    let color_ = (w * 255 / para.pix.width()).min(255) as u8;
    let color = u32::from_be_bytes([
        255,
        255,
        (sin_norm(color_ as f64 / 512.0) * 255.0) as u8,
        color_,
    ]);

    let mut local = DATA.write().unwrap();

    para.pix.draw_rect_wh(P2::new(2, local.sweepi as i32), para.pix.width(), 1, 0);
    para.pix.draw_rect_wh(P2::new(2, local.sweepi as i32), w, 1, color);

    crate::visualizers::classic::dash_line::draw_dash_line(para, stream, false, 0, false);

    // para.vol_sweeper.1 = math::advance_with_limit(para.vol_sweeper.1, para.pix.height());

    match (local.sweepi >= para.pix.height(), local.pong) {
        (false, true) => local.sweepi = local.sweepi.wrapping_add(1),
        (false, false) => local.sweepi = local.sweepi.wrapping_sub(1),

        (true, true) => {
            local.sweepi -= 1;
            local.pong ^= true;
        }

        (true, false) => {
            local.sweepi = 0;
            local.pong ^= true;
        }
    }
    
    stream.rotate_left(SAMPLE_SIZE >> 5);
}
