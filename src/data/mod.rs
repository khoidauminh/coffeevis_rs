#[cfg(not(target_os = "windows"))]
pub mod desktop;

#[macro_use]
pub mod log;

pub mod reader;

use std::time::Duration;

use crate::graphics::RenderEffect;
use crate::visualizers::VisList;
use crate::{graphics::PixelBuffer, modes::Mode};

use crate::modes;

pub const SAMPLE_RATE: usize = 44100;

pub const POWER: usize = 13;
pub const FFT_POWER: usize = 9;
pub const SAMPLE_SIZE: usize = 1 << POWER;
pub const FFT_SIZE: usize = 1 << FFT_POWER;

pub const INCREMENT: usize = 2;

pub const CAP_MILLI_HZ: u32 = 192 * 1000;
pub const DEFAULT_MILLI_HZ: u32 = 60 * 1000;
pub const DEFAULT_HZ: u64 = DEFAULT_MILLI_HZ as u64 / 1000;

pub const DEFAULT_WAV_WIN: usize = 64 * INCREMENT;
pub const DEFAULT_ROTATE_SIZE: usize = 289; // 3539;
pub const PHASE_OFFSET: usize = SAMPLE_RATE / 50 / 4;

/// Stop rendering when get_no_sample() exceeds this value;
pub const STOP_RENDERING: u8 = 192;
pub const SLOW_DOWN_THRESHOLD: u8 = 86;

pub const DEFAULT_SIZE_WIN: u16 = 84;
pub const DEFAULT_WIN_SCALE: u8 = 2;

pub const MAX_WIDTH: u16 = 480;

pub const MAX_HEIGHT: u16 = 360;

pub const MAX_CON_WIDTH: u16 = 128;

pub const MAX_SCALE_FACTOR: u8 = 8;

pub const DEFAULT_VIS_SWITCH_DURATION: Duration = Duration::from_secs(8);

#[derive(PartialEq, Clone, Copy)]
pub enum RefreshRateMode {
    Sync,
    Specified,
}

/// Main program struct
pub(crate) struct Program {
    /// for experimental purposes. Console mode only.
    display: bool,

    quiet: bool,

    scale: u8,

    /// Allow for resizing. Windowed mode only.
    resize: bool,

    win_render_effect: crate::graphics::RenderEffect,

    pub pix: crate::graphics::PixelBuffer,

    mode: Mode,

    transparent: bool,

    milli_hz: u32,
    refresh_rate_mode: RefreshRateMode,
    refresh_rate_intervals: [Duration; 2],

    pub window_props: modes::windowed_mode::WindowProps,

    pub console_props: modes::console_mode::ConsoleProps,

    vislist: Option<crate::visualizers::VisList>,

    auto_switch: bool,
}

impl Program {
    pub fn new() -> Self {
        let rate = Duration::from_nanos(1_000_000_000 / DEFAULT_HZ);

        let default_mode = Mode::Win;

        Self {
            quiet: false,
            display: true,
            scale: DEFAULT_WIN_SCALE,
            resize: false,

            win_render_effect: RenderEffect::Interlaced,

            mode: default_mode,

            transparent: false,

            pix: PixelBuffer::new(DEFAULT_SIZE_WIN as usize, DEFAULT_SIZE_WIN as usize),

            milli_hz: DEFAULT_MILLI_HZ,
            refresh_rate_mode: RefreshRateMode::Sync,

            refresh_rate_intervals: [rate, rate * 8],

            vislist: Some(VisList::new()),

            auto_switch: true,

            window_props: modes::windowed_mode::WindowProps {
                width: DEFAULT_SIZE_WIN,
                height: DEFAULT_SIZE_WIN,
            },

            console_props: modes::console_mode::ConsoleProps {
                width: 50,
                height: 25,
                max_width: 50,
                max_height: 25,
                flusher: default_mode.get_flusher(),
            },
        }
    }

    pub fn get_rr_interval(&self, no_sample: u8) -> Duration {
        let rr_index = (no_sample > SLOW_DOWN_THRESHOLD) as usize;
        self.refresh_rate_intervals[rr_index]
    }

    pub fn get_rr_intervals(&self) -> [Duration; 2] {
        self.refresh_rate_intervals
    }

    pub fn get_win_render_effect(&self) -> RenderEffect {
        self.win_render_effect
    }

    pub fn set_win_render_effect(&mut self, e: RenderEffect) {
        self.win_render_effect = e;
    }

    pub fn is_resizable(&self) -> bool {
        self.resize
    }

    pub fn is_display_enabled(&self) -> bool {
        self.display
    }

    pub fn is_transparent(&self) -> bool {
        self.transparent
    }

    pub fn get_milli_hz(&self) -> u32 {
        self.milli_hz
    }

    pub fn get_rr_mode(&self) -> RefreshRateMode {
        self.refresh_rate_mode
    }

    pub fn change_fps(&mut self, amount: i16, replace: bool) {
        if replace {
            self.milli_hz = amount as u32 * 1000;
        } else {
            self.milli_hz = self
                .milli_hz
                .saturating_add_signed(amount as i32 * 1000)
                .clamp(1, 200 * 1000);
        }

        self.change_fps_frac(self.milli_hz);
    }

    pub fn construct_intervals(fps: u32) -> [Duration; 2] {
        let fps_f = fps as f32 / 1000.0;
        let rate = (1_000_000_000.0 / fps_f) as u64;
        // self.milli_hz = fps;
        let interval = Duration::from_nanos(rate);

        [interval, interval * 8]
    }

    pub fn change_fps_frac(&mut self, fps: u32) {
        self.milli_hz = fps;
        self.refresh_rate_intervals = Self::construct_intervals(fps);
    }

    pub fn change_visualizer(&mut self, forward: bool) {
        let vislist = self.vislist.as_mut().unwrap();

        if forward {
            vislist.next();
        } else {
            vislist.prev();
        }

        self.pix.clear();

        let conf = vislist.get().config();

        crate::audio::set_normalizer(conf.normalize);

        let vis_name = vislist.get().name();

        info!("Switching to {}", vis_name);
    }

    pub fn autoupdate_visualizer(&mut self) {
        if let Some(v) = self.vislist.as_mut() {
            v.update()
        }
    }

    pub fn update_size(&mut self, mut s: (u16, u16)) {
        match &self.mode {
            Mode::Win => self.window_props.set_size(s),
            _ => s = self.console_props.set_size(s, self.mode()),
        }

        self.pix.resize(s.0 as usize, s.1 as usize);
    }

    pub fn render(&mut self) {
        let mut vis = self.vislist.take().unwrap();
        crate::visualizers::Visualizer::perform(vis.get(), self, &mut crate::audio::get_buf());
        self.vislist = Some(vis)
    }

    pub fn toggle_auto_switch(&mut self) {
        self.auto_switch ^= true;

        info!(
            "Auto switch is now {}",
            if self.auto_switch { "on" } else { "off" }
        );
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, m: Mode) {
        self.mode = m;
    }

    pub fn scale(&self) -> u8 {
        self.scale
    }

    pub fn reset_parameters(&mut self) {
        self.change_con_max(50, true);

        if self.refresh_rate_mode != RefreshRateMode::Specified {
            self.change_fps_frac(DEFAULT_MILLI_HZ);
        }
    }
}
