#[cfg(not(target_os = "windows"))]
pub mod desktop;

#[macro_use]
pub mod log;

pub mod config;

use std::time::Duration;

use crate::audio::AudioBuffer;
use crate::graphics::RenderEffect;
use crate::visualizers::{VisList, VisualizerConfig};
use crate::{graphics::PixelBuffer, modes::Mode};

use crate::modes;

pub const SAMPLE_RATE: usize = 44100;

pub const POWER: usize = 13;
pub const FFT_POWER: usize = 9;
pub const SAMPLE_SIZE: usize = 1 << POWER;
pub const FFT_SIZE: usize = 1 << FFT_POWER;

pub const DEFAULT_MILLI_HZ: u32 = 60 * 1000;
pub const DEFAULT_HZ: u64 = DEFAULT_MILLI_HZ as u64 / 1000;

/// Stop rendering when get_no_sample() exceeds this value;
pub const STOP_RENDERING: u8 = 192;

pub const DEFAULT_SIZE_WIN: u16 = 84;
pub const DEFAULT_WIN_SCALE: u8 = 2;

pub const MAX_WIDTH: u16 = 480;
pub const MAX_HEIGHT: u16 = 360;

pub const MAX_CON_WIDTH: u16 = 256;

pub const MAX_SCALE_FACTOR: u8 = 4;

pub const DEFAULT_BG_COLOR: u32 = u32::from_be_bytes([0, 0x24, 0x24, 0x24]);

pub const DEFAULT_VIS_SWITCH_DURATION: Duration = Duration::from_secs(8);

#[derive(PartialEq, Clone, Copy)]
pub enum RefreshRateMode {
    Sync,
    Specified,
}

#[derive(Default, Debug)]
pub struct KeyInput {
    pub z: bool,
    pub x: bool,
    pub c: bool,

    pub left: bool,
    pub up: bool,
    pub right: bool,
    pub down: bool,
}

pub(crate) struct Program {
    quiet: bool,

    nosleep: bool,

    scale: u8,

    resize: bool,

    win_render_effect: crate::graphics::RenderEffect,

    pub pix: crate::graphics::PixelBuffer,
    pub key: KeyInput,

    mode: Mode,

    milli_hz: u32,
    refresh_rate_mode: RefreshRateMode,
    refresh_rate_interval: Duration,

    pub window_props: modes::windowed_mode::WindowProps,

    pub console_props: modes::console_mode::ConsoleProps,

    vislist: crate::visualizers::VisList,

    auto_switch: bool,
}

impl Program {
    pub fn new() -> Self {
        let rate = Duration::from_nanos(1_000_000_000 / DEFAULT_HZ);

        let default_mode = Mode::Win;

        Self {
            quiet: false,

            nosleep: false,

            scale: DEFAULT_WIN_SCALE,
            resize: false,

            win_render_effect: RenderEffect::Interlaced,

            mode: default_mode,

            pix: PixelBuffer::new(DEFAULT_SIZE_WIN as usize, DEFAULT_SIZE_WIN as usize),
            key: KeyInput::default(),

            milli_hz: DEFAULT_MILLI_HZ,
            refresh_rate_mode: RefreshRateMode::Sync,
            refresh_rate_interval: rate,

            vislist: VisList::new(),

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

    pub fn nosleep(&self) -> bool {
        self.nosleep
    }

    pub fn get_rr_interval(&self) -> Duration {
        self.refresh_rate_interval
    }

    pub fn get_win_render_effect(&self) -> RenderEffect {
        self.win_render_effect
    }

    pub fn is_resizable(&self) -> bool {
        self.resize
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

    pub fn construct_interval(fps: u32) -> Duration {
        let fps_f = fps as f32 / 1000.0;
        let rate = (1_000_000_000.0 / fps_f) as u64;

        Duration::from_nanos(rate)
    }

    pub fn change_fps_frac(&mut self, fps: u32) {
        self.milli_hz = fps;
        self.refresh_rate_interval = Self::construct_interval(fps);
    }

    fn apply_vis_config(&mut self, conf: VisualizerConfig) {
        self.nosleep = conf.nosleep;
        crate::audio::get_buf().set_normalize(conf.normalize);
    }

    pub fn change_visualizer(&mut self, forward: bool) {
        if forward {
            self.vislist.next();
        } else {
            self.vislist.prev();
        }

        let conf = self.vislist.get().config();
        self.apply_vis_config(conf);

        self.pix.clear();
    }

    pub fn autoupdate_visualizer(&mut self) {
        if let Some(conf) = self.vislist.update() {
            self.apply_vis_config(conf);
        }
    }

    pub fn update_size(&mut self, mut s: (u16, u16)) {
        match &self.mode {
            Mode::Win => self.window_props.set_size(s),
            _ => s = self.console_props.set_size(s, self.mode()),
        }

        self.pix.resize(s.0 as usize, s.1 as usize);
    }

    pub fn render(&mut self, buf: &mut AudioBuffer) {
        self.vislist.get().perform(&mut self.pix, &self.key, buf);
    }

    pub fn toggle_auto_switch(&mut self) {
        self.vislist.auto_switch ^= true;

        info!(
            "Auto switch is now {}",
            if self.vislist.auto_switch {
                "on"
            } else {
                "off"
            }
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
