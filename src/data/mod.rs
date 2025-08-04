pub mod foreign;

#[cfg(not(target_os = "windows"))]
pub mod desktop;

#[macro_use]
pub mod log;

pub mod reader;
pub mod vislist;

use std::time::{Duration, Instant};

use crate::{graphics::PixelBuffer, modes::Mode};

use crate::{VisFunc, modes};

pub const SAMPLE_RATE: usize = 44100;

pub const POWER: usize = 13;
pub const FFT_POWER: usize = 9;
pub const SAMPLE_SIZE: usize = 1 << POWER;
pub const FFT_SIZE: usize = 1 << FFT_POWER;

pub const INCREMENT: usize = 2;

pub const CAP_MILLI_HZ: u32 = 192 * 1000;
pub const DEFAULT_MILLI_HZ: u32 = 60 * 1000;
pub const DEFAULT_HZ: u64 = DEFAULT_MILLI_HZ as u64 / 1000;
pub const IDLE_INTERVAL: Duration = Duration::from_millis(1000 / 5);

pub const DEFAULT_WAV_WIN: usize = 144 * INCREMENT;
pub const DEFAULT_ROTATE_SIZE: usize = 289; // 3539;
pub const PHASE_OFFSET: usize = SAMPLE_RATE / 50 / 4;
pub const DEFAULT_VOL_SCL: f32 = 0.86;
pub const DEFAULT_SMOOTHING: f32 = 0.65;

/// Stop rendering when get_no_sample() exceeds this value;
pub const STOP_RENDERING: u8 = 192;
pub const SLOW_DOWN_THRESHOLD: u8 = 86;

pub const DEFAULT_SIZE_WIN: u16 = 84;
pub const DEFAULT_WIN_SCALE: u8 = 2;

pub const MAX_WIDTH: u16 = 480;

pub const MAX_HEIGHT: u16 = 360;

pub const MAX_CON_WIDTH: u16 = 128;
pub const MAX_CON_HEIGHT: u16 = 96;

pub const MAX_SCALE_FACTOR: u8 = 8;
pub const CRT_EFFECT: bool = false;

type Real = f64;
type Int = i32;

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

    hidden: bool,

    crt: bool,

    pub pix: crate::graphics::PixelBuffer,

    mode: Mode,

    milli_hz: u32,
    refresh_rate_mode: RefreshRateMode,
    refresh_rate_intervals: [Duration; 2],

    pub wav_win: usize,
    pub vol_scl: f32,
    pub smoothing: f32,

    #[cfg(not(feature = "console_only"))]
    pub window_props: modes::windowed_mode::WindowProps,

    #[cfg(all(not(feature = "window_only"), target_os = "linux"))]
    pub console_props: modes::console_mode::ConsoleProps,

    vis_navigator: vislist::VisNavigator,

    visualizer: VisFunc,

    switch: Instant,
    auto_switch: bool,
    auto_switch_interval: Duration,

    foreign_program_communicator: Option<foreign::ForeignProgramCommunicator>,
}

impl Program {
    pub fn new() -> Self {
        let vislist_ = vislist::VisNavigator::new();
        let vis = vislist_.current_vis();

        let rate = Duration::from_nanos(1_000_000_000 / DEFAULT_HZ);

        let default_mode = Mode::default();

        Self {
            quiet: false,
            display: true,
            scale: DEFAULT_WIN_SCALE,
            resize: false,

            hidden: false,
            crt: false,

            mode: default_mode,

            pix: PixelBuffer::new(DEFAULT_SIZE_WIN as usize, DEFAULT_SIZE_WIN as usize),

            milli_hz: DEFAULT_MILLI_HZ,
            refresh_rate_mode: RefreshRateMode::Sync,

            refresh_rate_intervals: [rate, rate * 8],

            vis_navigator: vislist_,

            visualizer: vis.func(),

            switch: Instant::now() + Duration::from_secs(8),
            auto_switch: true,
            auto_switch_interval: Duration::from_secs(8),

            wav_win: DEFAULT_WAV_WIN,
            vol_scl: DEFAULT_VOL_SCL,
            smoothing: DEFAULT_SMOOTHING,

            #[cfg(not(feature = "console_only"))]
            window_props: modes::windowed_mode::WindowProps {
                width: DEFAULT_SIZE_WIN,
                height: DEFAULT_SIZE_WIN,
            },

            #[cfg(all(not(feature = "window_only"), target_os = "linux"))]
            console_props: modes::console_mode::ConsoleProps {
                width: 50,
                height: 25,
                max_width: 50,
                max_height: 25,
                flusher: default_mode.get_flusher(),
            },

            foreign_program_communicator: None,
        }
    }

    pub fn init_program_communicator(&mut self) {
        self.foreign_program_communicator = foreign::ForeignProgramCommunicator::new();
    }

    pub fn get_rr_interval(&self, no_sample: u8) -> Duration {
        let rr_index = (no_sample > SLOW_DOWN_THRESHOLD) as usize;
        self.refresh_rate_intervals[rr_index]
    }

    pub fn get_rr_intervals(&self) -> [Duration; 2] {
        self.refresh_rate_intervals
    }

    pub fn is_crt(&self) -> bool {
        self.crt
    }

    pub fn is_resizable(&self) -> bool {
        self.resize
    }

    pub fn is_display_enabled(&self) -> bool {
        self.display
    }

    pub fn set_hidden(&mut self, b: bool) {
        self.hidden = b;
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn reset_switch(&mut self) {
        self.switch = Instant::now() + self.auto_switch_interval;
    }

    pub fn get_milli_hz(&self) -> u32 {
        self.milli_hz
    }

    pub fn get_rr_mode(&self) -> RefreshRateMode {
        self.refresh_rate_mode
    }

    pub fn update_vis(&mut self) -> bool {
        let elapsed = Instant::now();

        if elapsed < self.switch || !self.auto_switch {
            return false;
        }

        self.switch = elapsed + self.auto_switch_interval;

        self.change_visualizer(true);

        true
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
        let new_visualizer = if forward {
            self.vis_navigator.next_vis()
        } else {
            self.vis_navigator.prev_vis()
        };

        self.visualizer = new_visualizer.func();

        self.pix.clear();

        self.reset_switch();

        crate::audio::set_normalizer(new_visualizer.request());

        let vis_name = self.vis_navigator.current_vis_name();
        let vis_list = self.vis_navigator.current_list_name();

        info!("Switching to {} in list {}", vis_name, vis_list);
    }

    pub fn change_vislist(&mut self) {
        self.vis_navigator.next_list();

        let new_visualizer = self.vis_navigator.current_vis();

        self.visualizer = new_visualizer.func();

        self.pix.clear();

        self.reset_switch();

        crate::audio::set_normalizer(new_visualizer.request());

        let vis_name = self.vis_navigator.current_vis_name();
        let vis_list = self.vis_navigator.current_list_name();

        info!("Switching to {} in list {}\r\n", vis_name, vis_list);
    }

    #[allow(unused_mut)]
    pub fn update_size(&mut self, mut s: (u16, u16)) {
        match &self.mode {
            #[cfg(not(feature = "console_only"))]
            Mode::Win => {
                self.window_props.set_size(s);
            }

            _ => {
                #[cfg(all(not(feature = "window_only"), target_os = "linux"))]
                {
                    s = self.console_props.set_size(s, self.mode());
                }
            }
        }

        self.pix.resize(s.0 as usize, s.1 as usize);

        self.write_to_foreign_communicator();
    }

    pub fn write_to_foreign_communicator(&mut self) {
        if let Some(c) = self.foreign_program_communicator.as_mut() {
            let _ = c.write(format_args!(
                "0.6.0\n{} {}\n{}",
                self.pix.width(),
                self.pix.height(),
                self.milli_hz / 1000
            ));
        }
    }

    pub fn refresh(&mut self) {
        match &self.mode {
            #[cfg(not(feature = "console_only"))]
            Mode::Win => self.pix.resize(
                self.window_props.width as usize,
                self.window_props.height as usize,
            ),
            _ => {
                #[cfg(all(not(feature = "window_only"), target_os = "linux"))]
                self.pix.resize(
                    self.console_props.width as usize,
                    self.console_props.height as usize,
                );
            }
        }
    }

    pub fn render(&mut self) {
        if !self.pix.is_foreign() {
            let mut buf = crate::audio::get_buf();
            (self.visualizer)(self, &mut buf);
        }
        self.pix.draw_to_self();
    }

    pub fn increase_vol_scl(&mut self) {
        self.vol_scl = (self.vol_scl * 1.2).clamp(0.0, 10.0);
    }

    pub fn decrease_vol_scl(&mut self) {
        self.vol_scl = (self.vol_scl / 1.2).clamp(0.0, 10.0);
    }

    pub fn increase_smoothing(&mut self) {
        self.smoothing = (self.smoothing + 0.05).clamp(0.0, 0.95);
    }

    pub fn decrease_smoothing(&mut self) {
        self.smoothing = (self.smoothing - 0.05).clamp(0.0, 0.95);
    }

    pub fn increase_wav_win(&mut self) {
        self.wav_win = (self.wav_win * 5 / 4).clamp(3, 500)
    }

    pub fn decrease_wav_win(&mut self) {
        self.wav_win = (self.wav_win * 4 / 5).clamp(3, 500)
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
        self.vol_scl = DEFAULT_VOL_SCL;
        self.smoothing = DEFAULT_SMOOTHING;
        self.wav_win = DEFAULT_WAV_WIN;

        #[cfg(all(not(feature = "window_only"), target_os = "linux"))]
        self.change_con_max(50, true);

        if self.refresh_rate_mode != RefreshRateMode::Specified {
            self.change_fps_frac(DEFAULT_MILLI_HZ);
        }
    }
}
