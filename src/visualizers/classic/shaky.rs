use crate::math::{self, Cplx, fast};
use crate::graphics::blend::Blend;

// soft shaking
const INCR: f64 = 0.0001;

struct LocalData {
    i: f64,
    js: f64,
    jc: f64,
    xshake: f64,
    yshake: f64,
    x: i32,
    y: i32
}

static DATA: std::sync::RwLock<LocalData> = std::sync::RwLock::new(
    LocalData{i: 0.0, js: 0.0, jc: 0.0, xshake: 0.0, yshake: 0.0, x: 0, y: 0}
);

fn diamond_func(amp: f64, prd: f64, t: f64) -> (i32, i32) {
    (
        triangle_wav(amp, prd, t) as i32,
        triangle_wav(amp, prd, t+prd/4.0) as i32
    )
}

fn triangle_wav(amp: f64, prd: f64, t: f64) -> f64 {
    (4.0*(t/prd - (t/prd+0.5).trunc()).abs()-1.0)*amp
}

pub fn draw_shaky(
	prog: &mut crate::data::Program, 
	stream: &mut crate::audio::SampleArr
) {
    let mut LOCALDATA = DATA.write().unwrap();
   
    let mut data_f = [Cplx::zero(); 512];

    let sizef = prog.pix.width().min(prog.pix.height()) as f64;

    data_f
    .iter_mut()
    .enumerate()
    .for_each(|(i, x)|
        *x = stream[i] //.scale(prog.VOL_SCL)
    );
    math::integrate_inplace(&mut data_f, 128, false);

    let amplitude = data_f.iter().fold(0f64, |acc, x| acc + x.l1_norm())*sizef;
    
    // let mut LOCALDATA = DATA.write().unwrap();

    //smooth_amplitude = graphical_fn::linear_interp(smooth_amplitude, amplitude.min(1024.0), 0.5);
    let smooth_amplitude = amplitude * 0.00003;
    let amplitude_scaled = amplitude * 0.00000002;
    
    LOCALDATA.js = (LOCALDATA.js + amplitude_scaled) % 2.0;
    LOCALDATA.jc = (LOCALDATA.jc + amplitude_scaled*crate::math::PIH) % 2.0;

    LOCALDATA.xshake = (smooth_amplitude)*fast::cos_norm(fast::wrap(LOCALDATA.jc));
    LOCALDATA.yshake = (smooth_amplitude)*fast::sin_norm(fast::wrap(LOCALDATA.js));

    // println!("{:.2} {:.2}", LOCALDATA.xshake, LOCALDATA.yshake);

    LOCALDATA.x = math::interpolate::linearf(LOCALDATA.x as f64, LOCALDATA.xshake, 0.1) as i32;
    LOCALDATA.y = math::interpolate::linearf(LOCALDATA.y as f64, LOCALDATA.yshake, 0.1) as i32;

    LOCALDATA.js += 0.01;
    LOCALDATA.jc += 0.01;
    LOCALDATA.i = (LOCALDATA.i + INCR +amplitude_scaled) % 1.0;

    
    prog.pix.fade(4);
    
    let (x_soft_shake, y_soft_shake) = diamond_func(8.0, 1.0, LOCALDATA.i);
    
    let final_x = x_soft_shake + LOCALDATA.x;
    let final_y = y_soft_shake + LOCALDATA.y;
    
    let width = prog.pix.width() as i32;
    let height = prog.pix.height() as i32;
    
    let red = 255i32.saturating_sub(final_x.abs()*5) as u8;
    let blue  = 255i32.saturating_sub(final_y.abs()*5) as u8;
    let green = (amplitude * 0.0001) as u8;
    
    /*prog.pix.draw_image(
		&prog.IMG,
		crate::graphics::P2::new(
		    x_soft_shake + LOCALDATA.x,
		    y_soft_shake + LOCALDATA.y,
		), 
		150
	);*/
	
	prog.pix.draw_rect_wh(
		crate::graphics::P2::new(
		   final_x + width /2 - 1,
		   final_y + height/2 - 1,
		),
		3, 3,
		u32::compose([0xFF, red, green, blue])
	); 

}

const WRAPPER: f64 = 725.0;
