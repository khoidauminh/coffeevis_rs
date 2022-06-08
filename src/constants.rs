pub const pif32: f32 = std::f32::consts::PI;
pub const pih: f32 = pif32 / 2.0;
pub const pi2: f32 = pif32 * 2.0;
pub const minus_pi2: f32 = -pi2;

pub const FPS: u64 = 64;
pub const FPS_ITVL: u64 = 1_000_000 / FPS;
pub const CTRL_ITVL: u64 = FPS << 2;

pub const AUTO_SWITCH_ITVL: u16 = FPS as u16 * 8; // Visualizer will switch every N frames (default: 8 seconds);
pub const PHASE_OFFSET: usize = 2000;
pub const INCREMENT: usize = 8;

pub const POWER: usize = 12;
pub const FFT_SIZE: usize = 1 << POWER;
pub const SAMPLE_SIZE: usize = 3 << POWER;
pub const VISES: usize = 11;

pub struct Image {
    pub w: usize,
    pub h: usize,
    pub image_buffer: Vec<u8>,
}

pub struct Parameters {
    pub SWITCH: u16,
    pub AUTO_SWITCH: u16,
    pub SWITCH_INCR: u16,
    pub VIS_IDX: usize,

    pub WAV_WIN: usize,
    pub VOL_SCL: f32,
    pub SMOOTHING: f32,

    pub WIN_W: usize,
    pub WIN_H: usize,
    pub WIN_R: usize,
    pub WIN_RL: usize,
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

pub fn console_clear() {
    print!("\x1B[2J\x1B[H");
}

pub fn usleep(micros: u64) {
    std::thread::sleep(std::time::Duration::from_micros(micros));
}
