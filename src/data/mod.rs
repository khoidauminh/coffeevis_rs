pub mod reader;
pub mod vislist;

use core::fmt;

use std::time::{Duration, Instant};

use crate::{graphics::Canvas, modes::Mode};

use crate::VisFunc;

pub const SAMPLE_RATE: usize = 44100;

pub const POWER: usize = 13;
pub const FFT_POWER: usize = 9;
pub const SAMPLE_SIZE: usize = 1 << POWER;
pub const FFT_SIZE: usize = 1 << FFT_POWER;

pub const INCREMENT: usize = 2;

pub const CAP_MILLI_HZ: u32 = 192 * 1000;
pub const DEFAULT_MILLI_HZ: u32 = 144 * 1000;
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

#[derive(PartialEq, Clone, Copy)]
pub enum RefreshRateMode {
    Sync,
    Specified,
}

macro_rules! eprintln_red {
    () => {
        eprintln!()
    };
    ($arg:tt) => {
        eprintln!("\x1B[31;1m{}\x1B[0m", $arg)
    };
}

macro_rules! format_red {
    () => {
        format!()
    };
    ($arg:tt) => {
        format!("\x1B[31;1m{}\x1B[0m", $arg)
    };
}

#[allow(unused_macros)]
macro_rules! panic_red {
    () => {
        format!()
    };
    ($arg:tt) => {
        panic!("\x1B[31;1m{}\x1B[0m", $arg)
    };
}

pub(crate) use eprintln_red;
pub(crate) use format_red;

/// Main program struct
///
/// Notes:
/// Windowed mode resolution will be stored in `window_width` and `window_height`.
/// Console mode reolution are stored in `console_width` and `console_height`,
/// with special fields: `console_max_width` and `console_max_height` for maximum
/// console resolution allowed.
pub(crate) struct Program {
    /// for experimental purposes. Console mode only.
    display: bool,

    quiet: bool,

    scale: u8,

    /// Allow for resizing. Windowed mode only.
    resize: bool,

    hidden: bool,

    crt: bool,

    pub pix: crate::graphics::Canvas,

    mode: Mode,

    milli_hz: u32,
    refresh_rate_mode: RefreshRateMode,
    refresh_rate_intervals: [Duration; 2],

    pub wav_win: usize,
    pub vol_scl: f32,
    pub smoothing: f32,

    #[cfg(not(feature = "console_only"))]
    window_width: u16,

    #[cfg(not(feature = "console_only"))]
    window_height: u16,

    #[cfg(not(feature = "window_only"))]
    console_width: u16,

    #[cfg(not(feature = "window_only"))]
    console_height: u16,

    #[cfg(not(feature = "window_only"))]
    console_max_width: u16,

    #[cfg(not(feature = "window_only"))]
    console_max_height: u16,

    vis_navigator: vislist::VisNavigator,

    visualizer: VisFunc,

    #[cfg(not(feature = "window_only"))]
    pub flusher: crate::modes::console_mode::Flusher,

    switch: Instant,
    auto_switch: bool,
    auto_switch_interval: Duration,
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

            pix: Canvas::new(DEFAULT_SIZE_WIN as usize, DEFAULT_SIZE_WIN as usize),

            milli_hz: DEFAULT_MILLI_HZ,
            refresh_rate_mode: RefreshRateMode::Sync,

            refresh_rate_intervals: [rate, rate * 8],

            vis_navigator: vislist_,

            visualizer: vis.func(),

            #[cfg(not(feature = "window_only"))]
            flusher: default_mode.get_flusher(),

            switch: Instant::now() + Duration::from_secs(8),
            auto_switch: true,
            auto_switch_interval: Duration::from_secs(8),

            wav_win: DEFAULT_WAV_WIN,
            vol_scl: DEFAULT_VOL_SCL,
            smoothing: DEFAULT_SMOOTHING,

            #[cfg(not(feature = "console_only"))]
            window_width: DEFAULT_SIZE_WIN,

            #[cfg(not(feature = "console_only"))]
            window_height: DEFAULT_SIZE_WIN,

            #[cfg(not(feature = "window_only"))]
            console_width: 50,

            #[cfg(not(feature = "window_only"))]
            console_height: 25,

            #[cfg(not(feature = "window_only"))]
            console_max_width: 50,

            #[cfg(not(feature = "window_only"))]
            console_max_height: 25,
        }
    }

    pub fn get_rr_interval(&self, no_sample: u8) -> Duration {
        let rr_index = (no_sample > SLOW_DOWN_THRESHOLD) as usize;
        self.refresh_rate_intervals[rr_index]
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

    pub fn change_fps_frac(&mut self, fps: u32) {
        let fps_f = fps as f32 / 1000.0;
        let rate = (1_000_000_000.0 / fps_f) as u64;
        self.milli_hz = fps;
        let interval = Duration::from_nanos(rate);

        self.refresh_rate_intervals = [interval, interval * 8];
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

        self.print_message(format!(
            "Switching to {} in list {}\r\n",
            vis_name, vis_list
        ))
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

        self.print_message(format!(
            "Switching to {} in list {}\r\n",
            vis_name, vis_list
        ))
    }

    pub fn print_message<S: fmt::Display>(&self, message: S) {
        if self.quiet {
            return;
        }

        use std::io::Write;
        let mut stdout = std::io::stdout();

        if self.display && self.mode.is_con() {
            #[cfg(not(feature = "window_only"))]
            {
                use crossterm::{
                    style::Print,
                    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
                };

                let _ = crossterm::queue!(
                    stdout,
                    LeaveAlternateScreen,
                    Print(message),
                    EnterAlternateScreen
                );
            }

            return;
        }

        print!("{}", message);
        let _ = stdout.flush();
    }

    #[allow(unused_mut)]
    pub fn update_size(&mut self, mut s: (u16, u16)) {
        match &self.mode {
            #[cfg(not(feature = "console_only"))]
            Mode::WinX11 | Mode::WinWayland => {
                self.window_width = s.0;
                self.window_height = s.1;
            }

            _ => {
                #[cfg(not(feature = "window_only"))]
                {
                    self.console_width = s.0;
                    self.console_height = s.1;
                    s = self.rescale(s);
                }
            }
        }

        self.pix.resize(s.0 as usize, s.1 as usize);
    }

    pub fn refresh(&mut self) {
        match &self.mode {
            #[cfg(not(feature = "console_only"))]
            Mode::WinX11 | Mode::WinWayland => self
                .pix
                .resize(self.window_width as usize, self.window_height as usize),
            _ => {
                #[cfg(not(feature = "window_only"))]
                self.pix
                    .resize(self.console_width as usize, self.console_height as usize);
            }
        }
    }

    pub fn render(&mut self) {
        let mut buf = crate::audio::get_buf();
        (self.visualizer)(self, &mut buf);
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

        self.print_message(format!(
            "Auto switch is now {}\n",
            if self.auto_switch { "on" } else { "off" }
        ));
    }

    pub fn reset_parameters(&mut self) {
        self.vol_scl = DEFAULT_VOL_SCL;
        self.smoothing = DEFAULT_SMOOTHING;
        self.wav_win = DEFAULT_WAV_WIN;

        #[cfg(not(feature = "window_only"))]
        self.change_con_max(50, true);

        if self.refresh_rate_mode != RefreshRateMode::Specified {
            self.change_fps_frac(DEFAULT_MILLI_HZ);
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn scale(&self) -> u8 {
        self.scale
    }

    #[cfg(not(feature = "window_only"))]
    pub fn switch_con_mode(&mut self) {
        self.mode = self.mode.next();
        self.flusher = self.mode.get_flusher();
        self.refresh_con();
    }

    #[cfg(not(feature = "window_only"))]
    pub fn change_con_max(&mut self, amount: i16, replace: bool) {
        self.console_max_width = if replace {
            amount as u16
        } else {
            self.console_max_width
                .saturating_add_signed(amount)
                .clamp(0, MAX_CON_WIDTH)
        };
        self.console_max_height = self.console_max_width / 2;
        self.clear_con();
    }

    #[cfg(not(feature = "window_only"))]
    pub fn refresh_con(&mut self) {
        self.update_size((self.console_width, self.console_height));
    }

    #[cfg(not(feature = "window_only"))]
    pub fn get_center(&self, divider_x: u16, divider_y: u16) -> (u16, u16) {
        (
            (self.console_width / 2).saturating_sub(self.pix.width() as u16 / divider_x),
            (self.console_height / 2).saturating_sub(self.pix.height() as u16 / divider_y),
        )
    }

    #[cfg(not(feature = "window_only"))]
    pub fn console_size(&self) -> (u16, u16) {
        (self.console_width, self.console_height)
    }

    #[cfg(not(feature = "window_only"))]
    pub fn rescale(&self, mut s: (u16, u16)) -> (u16, u16) {
        s.0 = s.0.min(self.console_max_width);
        s.1 = s.1.min(self.console_max_height);

        match self.mode() {
            Mode::ConBrail => {
                s.0 *= 2;
                s.1 *= 4;
            }
            _ => {
                s.1 *= 2;
            }
        }

        s
    }
}
