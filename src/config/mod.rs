pub const pif32: f32 = std::f32::consts::PI;
pub const pih: f32 = pif32 / 2.0;
pub const pi2: f32 = pif32 * 2.0;
pub const minus_pi2: f32 = -pi2;

pub const SAMPLE_RATE: u32 = 44100;
//pub const AUTO_SWITCH_ITVL: u16 = FPS as u16 * 8; // Visualizer will switch every N frames
pub const PHASE_OFFSET: usize = 2000;
pub const INCREMENT: usize = 8;

pub const POWER: usize = 12;
pub const FFT_SIZE: usize = 1 << POWER;
pub const SAMPLE_SIZE: usize = 3 << POWER;
pub const VISES: usize = 12;

pub struct Image {
    pub w: usize,
    pub h: usize,
    pub image_buffer: Vec<u8>,
}

pub struct Parameters<'a> {
    pub FPS: u64,
    pub FPS_ITVL: u64,
    pub AUTO_SWITCH_ITVL: u16,

    pub vis_func: crate::graphics::visualizers::VisFunc,
    pub con_func: crate::console_mode::OutFunc,
    pub con_bool: bool,
    pub char_bool: bool,
    pub ascii_set: &'a [char],

    pub SWITCH: u16,
    pub AUTO_SWITCH: u16,
    pub SWITCH_INCR: u16,
    pub VIS_IDX: usize,

    pub WAV_WIN: usize,
    pub VOL_SCL: f32,
    pub SMOOTHING: f32,

    pub WIN_W: usize,
    pub WIN_H: usize,
    pub CENTER: (u16, u16),
    pub WIN_R: usize,
    pub WIN_RL: usize,
    pub CON_MAX_W: u16,
    pub CON_MAX_H: u16,

    pub IMG: Image,

    pub _i: usize,

    // local parameters
    pub cross: bool,
    pub bars_smoothing_ft: Vec<f32>,
    pub bars_smoothing_ft_circle: Vec<f32>,
    pub shaky_coffee: (f32, f32, f32, f32, i32, i32),
    pub vol_sweeper: (usize, usize),
    pub lazer: (i32, i32, i32, i32, bool),
    pub spectrum_smoothing_ft: Vec<(f32, f32)>,
}

impl Parameters<'_> {
/*	pub fn new(image: &[u8]) -> Self {
		let mut para = Self::emp();
		crate::graphics::graphical_fn::update_size((113, 113), &mut para);

        para.vis_func = crate::graphics::visualizers::VisFunc;
        para.con_func =
		para.IMG = crate::graphics::visualizers::shaky_coffee::prepare_img(image);
		para.bars_smoothing_ft = vec![0.0f32; 144];
		para.bars_smoothing_ft_circle = vec![0.0f32; 144];
		para.spectrum_smoothing_ft = vec![(0.0, 0.0); FFT_SIZE + 1];

		para
	}

	pub const fn emp() -> Self {
	    Self {
	        vis_func: crate::graphics::visualizers::oscilloscope::draw_vectorscope,
	        con_func: crate::console_mode::prepare_stdout_ascii,
	        ascii_set: &crate::console_mode::CHARSET_SIZEOPAC,

			SWITCH: 0,
			AUTO_SWITCH: 1,
			SWITCH_INCR: 0,
			VIS_IDX: 0,

			WAV_WIN: 30,
			VOL_SCL: 0.8,
			SMOOTHING: 0.5,

			WIN_W: 128,
			WIN_H: 128,
			WIN_R: 128 * 128,
			WIN_RL: 128 * 128 - 1,

			IMG: Image { w: 0, h: 0, image_buffer: Vec::new()},

			_i: 0,

			cross: false,
			bars_smoothing_ft: Vec::new(),
			bars_smoothing_ft_circle: Vec::new(),
			shaky_coffee: (0.0, 0.0, 0.0, 0.0, 0, 0),
			vol_sweeper: (0, 0),
			lazer: (0, 0, 0, 0, false),
			spectrum_smoothing_ft: Vec::new(),
		}
	}*/

	pub fn new(image: &[u8]) -> Self {
	     Self {
	        FPS: 60,
            FPS_ITVL: 1000 / 60,
            AUTO_SWITCH_ITVL: 60*8,

	        vis_func: crate::graphics::visualizers::oscilloscope::draw_vectorscope,
	        con_func: crate::console_mode::prepare_stdout_ascii,
            con_bool: false,
            char_bool: false,
	        ascii_set: &crate::console_mode::CHARSET_SIZEOPAC,

			SWITCH: 0,
			AUTO_SWITCH: 1,
			SWITCH_INCR: 0,
			VIS_IDX: 0,

			WAV_WIN: 30,
			VOL_SCL: 0.8,
			SMOOTHING: 0.5,

			WIN_W: 128,
			WIN_H: 128,
			WIN_R: 128 * 128,
			WIN_RL: 128 * 128 - 1,
			CENTER: (64, 64),
		    CON_MAX_W: 50,
            CON_MAX_H: 25,

			IMG: crate::graphics::visualizers::shaky_coffee::prepare_img(image),

			_i: 0,

			cross: false,
			bars_smoothing_ft: vec![0.0f32; 144],
			bars_smoothing_ft_circle: vec![0.0f32; 144],
			shaky_coffee: (0.0, 0.0, 0.0, 0.0, 0, 0),
			vol_sweeper: (0, 0),
			lazer: (0, 0, 0, 0, false),
			spectrum_smoothing_ft: vec![(0.0, 0.0); FFT_SIZE + 1],
		}
	}
}
