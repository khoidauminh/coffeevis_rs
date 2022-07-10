pub mod visualizers;
pub mod graphical_fn;

use minifb;
use crate::{
    config::*,
    controls,
    VisFunc,
    graphics::visualizers::oscilloscope::draw_vectorscope
};

pub fn graphics_main() -> Result<(), minifb::Error> {
    let mut para = Parameters::new(crate::IMAGE);
    let mut win = controls::init_window(&para)?;
    let mut pix_buf = vec![0u32; para.WIN_R];
    let mut visualizer: VisFunc = draw_vectorscope;

    while win.is_open() && !win.is_key_down(minifb::Key::Q) {
        let s = win.get_size();
        if s.0 != para.WIN_W || s.1 != para.WIN_H {
            graphical_fn::update_size(s, &mut para);
            pix_buf = vec![0u32; s.0 * s.1];
        }

        if para.SWITCH_INCR == para.AUTO_SWITCH_ITVL {
            controls::change_visualizer(&mut pix_buf, &mut para);
            para.SWITCH_INCR = 0;
        }

        let stream_data = unsafe { crate::buf };
        (para.vis_func)(&mut pix_buf, unsafe{&crate::buf}, &mut para);

        controls::control_key_events_win(&mut win, &mut pix_buf, &mut para);

        // controls::usleep(FPS_ITVL);

        para.SWITCH_INCR += para.AUTO_SWITCH;

        win.update_with_buffer(&mut pix_buf, para.WIN_W, para.WIN_H);
    }

    Ok(())
}
