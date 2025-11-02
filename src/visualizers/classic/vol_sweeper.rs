use std::cell::RefCell;

use crate::data::SAMPLE_SIZE;
use crate::graphics::P2;
use crate::math::fast::sin_norm;

const C: u32 = 16720064;

struct LocalData {
    sweepi: usize,
    pong: bool,
}

thread_local! {
    static DATA: std::cell::RefCell<LocalData> = RefCell::new(LocalData {
        sweepi: 0,
        pong: false,
    });
}

pub fn draw_vol_sweeper(para: &mut crate::Program, stream: &mut crate::AudioBuffer) {
    para.pix.fade(3);

    let w = {
        let mut sum = 0.0;
        for i in 0..SAMPLE_SIZE / 4 {
            sum += stream.get(i).l1_norm();
        }
        (sum / (SAMPLE_SIZE / 3) as f32 * para.pix.width() as f32) as usize
    };

    let color = (w * 255 / para.pix.width()).min(255) as u8;
    let color = u32::from_be_bytes([
        255,
        255,
        (sin_norm(color as f32 / 512.0) * 255.0) as u8,
        color,
    ]);

    let width = para.pix.width();

    DATA.with_borrow_mut(|local| {
        para.pix.color(0);
        para.pix.mixerm();
        para.pix.rect(P2(0, local.sweepi as i32), width, 1);
        para.pix.color(color);
        para.pix.rect(P2(0, local.sweepi as i32), w, 1);

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
    });

    stream.autoslide();
}
