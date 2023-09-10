use crate::data::{Program};
use crate::graphics::{self, Image};
use crate::math::{self, Cplx, fast};

// soft shaking
const incr: f32 = 0.0001;

struct LocalData {
    i: f32,
    js: f32,
    jc: f32,
    xshake: f32,
    yshake: f32,
    x: i32,
    y: i32
}

static DATA: std::sync::RwLock<LocalData> = std::sync::RwLock::new(
    LocalData{i: 0.0, js: 0.0, jc: 0.0, xshake: 0.0, yshake: 0.0, x: 0, y: 0}
);

fn diamond_func(amp: f32, prd: f32, t: f32) -> (i32, i32) {
    (
        triangle_wav(amp, prd, t) as i32,
        triangle_wav(amp, prd, t+prd/4.0) as i32
    )
}

fn triangle_wav(amp: f32, prd: f32, t: f32) -> f32 {
    (4.0*(t/prd - (t/prd+0.5).trunc()).abs()-1.0)*amp
}

pub const draw_shaky: crate::VisFunc = |prog, stream| {
    let mut LOCALDATA = DATA.write().unwrap();
   
    LOCALDATA.i = (LOCALDATA.i + incr) % 1.0;

    let mut data_f = [Cplx::<f32>::zero(); 512];

    let sizef = prog.pix.width().min(prog.pix.height()) as f32;

    data_f
    .iter_mut()
    .enumerate()
    .for_each(|(i, x)|
        *x = stream[i] //.scale(prog.VOL_SCL)
    );
    math::integrate_inplace(&mut data_f, 128, false);

    let amplitude = data_f.iter().fold(0f32, |acc, x| acc + x.l1_norm())*sizef;
    
    // let mut LOCALDATA = DATA.write().unwrap();

    //smooth_amplitude = graphical_fn::linear_interp(smooth_amplitude, amplitude.min(1024.0), 0.5);
    let smooth_amplitude = amplitude * 0.00001;
    let amplitude_scaled = amplitude * 0.00000002;
    
    LOCALDATA.js = (LOCALDATA.js + amplitude_scaled) % 2.0;
    LOCALDATA.jc = (LOCALDATA.jc + amplitude_scaled*crate::math::PIH) % 2.0;

    LOCALDATA.xshake = (smooth_amplitude)*fast::cos_norm(fast::wrap(LOCALDATA.jc));
    LOCALDATA.yshake = (smooth_amplitude)*fast::sin_norm(fast::wrap(LOCALDATA.js));

    // println!("{:.2} {:.2}", LOCALDATA.xshake, LOCALDATA.yshake);

    LOCALDATA.x = math::interpolate::linearf(LOCALDATA.x as f32, LOCALDATA.xshake, 0.1) as i32;
    LOCALDATA.y = math::interpolate::linearf(LOCALDATA.y as f32, LOCALDATA.yshake, 0.1) as i32;

    LOCALDATA.js += 0.01;
    LOCALDATA.jc += 0.01;

    
    prog.clear_pix();
    
    let (x_soft_shake, y_soft_shake) = diamond_func(8.0, 1.0, LOCALDATA.i);
    
    prog.pix.draw_image(
		&prog.IMG,
		crate::graphics::P2::new(
		    x_soft_shake + LOCALDATA.x,
		    y_soft_shake + LOCALDATA.y,
		), 
		150
	);

};

const wrapper: f32 = 725.0;
