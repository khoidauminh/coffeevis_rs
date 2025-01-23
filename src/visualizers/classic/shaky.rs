use std::{f32::consts::FRAC_PI_2, sync::Mutex};

use crate::graphics::Pixel;
use crate::math::{self, fast, Cplx};

// soft shaking
const INCR: f32 = 0.0001;

struct LocalData {
    i: f32,
    js: f32,
    jc: f32,
    xshake: f32,
    yshake: f32,
    x: i32,
    y: i32,
}

static DATA: Mutex<LocalData> = Mutex::new(LocalData {
    i: 0.0,
    js: 0.0,
    jc: 0.0,
    xshake: 0.0,
    yshake: 0.0,
    x: 0,
    y: 0,
});

fn diamond_func(amp: f32, prd: f32, t: f32) -> (i32, i32) {
    (
        triangle_wav(amp, prd, t) as i32,
        triangle_wav(amp, prd, t + prd / 4.0) as i32,
    )
}

fn triangle_wav(amp: f32, prd: f32, t: f32) -> f32 {
    (4.0 * (t / prd - (t / prd + 0.5).trunc()).abs() - 1.0) * amp
}

pub fn draw_shaky(prog: &mut crate::data::Program, stream: &mut crate::audio::SampleArr) {
    let mut localdata = DATA.lock().unwrap();

    let mut data_f = [Cplx::zero(); 512];

    let sizef = prog.pix.width().min(prog.pix.height()) as f32;

    data_f
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = stream[i]);
    math::integrate_inplace(&mut data_f, 128, false);

    let amplitude = data_f.iter().fold(0f32, |acc, x| acc + x.l1_norm()) * sizef;

    let smooth_amplitude = amplitude * 0.00003;
    let amplitude_scaled = amplitude * 0.00000002;

    localdata.js = (localdata.js + amplitude_scaled) % 2.0;
    localdata.jc = (localdata.jc + amplitude_scaled * FRAC_PI_2) % 2.0;

    localdata.xshake = (smooth_amplitude) * fast::cos_norm(fast::wrap(localdata.jc));
    localdata.yshake = (smooth_amplitude) * fast::sin_norm(fast::wrap(localdata.js));

    localdata.x = math::interpolate::linearf(localdata.x as f32, localdata.xshake, 0.1) as i32;
    localdata.y = math::interpolate::linearf(localdata.y as f32, localdata.yshake, 0.1) as i32;

    localdata.js += 0.01;
    localdata.jc += 0.01;
    localdata.i = (localdata.i + INCR + amplitude_scaled) % 1.0;

    prog.pix.command.fade(4);

    let (x_soft_shake, y_soft_shake) = diamond_func(8.0, 1.0, localdata.i);

    let final_x = x_soft_shake + localdata.x;
    let final_y = y_soft_shake + localdata.y;

    let width = prog.pix.width() as i32;
    let height = prog.pix.height() as i32;

    let red = 255i32.saturating_sub(final_x.abs() * 5) as u8;
    let blue = 255i32.saturating_sub(final_y.abs() * 5) as u8;
    let green = (amplitude * 0.0001) as u8;

    prog.pix.command.rect_wh(
        crate::graphics::P2::new(final_x + width / 2 - 1, final_y + height / 2 - 1),
        3,
        3,
        u32::compose([0xFF, red, green, blue]),
        u32::mix,
    );
}

const WRAPPER: f32 = 725.0;
