use crate::data::SAMPLE_SIZE;
use crate::graphics::Pixel;
use crate::graphics::P2;
use crate::math::fast::sin_norm;
use std::sync::Mutex;

const C: u32 = 16720064;

struct LocalData {
    sweepi: usize,
    pong: bool,
}

static DATA: Mutex<LocalData> = Mutex::new(LocalData {
    sweepi: 0,
    pong: false,
});

pub fn draw_vol_sweeper(para: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    para.pix.command.fade(3);

    let w = {
        let mut sum = 0.0;
        for i in 0..SAMPLE_SIZE / 4 {
            sum += stream[i].l1_norm();
        }
        (sum / (SAMPLE_SIZE / 3) as f32 * para.vol_scl * para.pix.width() as f32) as usize
    };

    let color_ = (w * 255 / para.pix.width()).min(255) as u8;
    let color = u32::from_be_bytes([
        255,
        255,
        (sin_norm(color_ as f32 / 512.0) * 255.0) as u8,
        color_,
    ]);

    let mut local = DATA.lock().unwrap();

    para.pix.command.rect_wh(
        P2::new(0, local.sweepi as i32),
        para.pix.width(),
        1,
        0,
        u32::mix,
    );
    para.pix
        .command
        .rect_wh(P2::new(0, local.sweepi as i32), w, 1, color, u32::mix);

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

    stream.auto_rotate();
}
