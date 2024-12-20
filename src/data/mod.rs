pub mod reader;
pub mod vislist;

use std::time::{Duration, Instant};

use crate::modes::Mode;

use crate::VisFunc;

pub const SAMPLE_RATE: usize = 44100;

pub const POWER: usize = 13;
pub const FFT_POWER: usize = 9;
pub const SAMPLE_SIZE: usize = 1 << POWER;
pub const FFT_SIZE: usize = 1 << FFT_POWER;

pub const INCREMENT: usize = 2;

pub const DEFAULT_MILLI_HZ: u32 = 144 * 1000;
pub const DEFAULT_HZ: u64 = DEFAULT_MILLI_HZ as u64 / 1000;

pub const DEFAULT_WAV_WIN: usize = 144 * INCREMENT;
pub const DEFAULT_ROTATE_SIZE: usize = 289; // 3539;
pub const PHASE_OFFSET: usize = SAMPLE_RATE / 50 / 4;
pub const DEFAULT_VOL_SCL: f32 = 0.86;
pub const DEFAULT_SMOOTHING: f32 = 0.65;

/// Stop rendering when get_no_sample() exceeds this value;
pub const STOP_RENDERING: u8 = 192;

pub const DEFAULT_SIZE_WIN: u16 = 84;
pub const DEFAULT_WIN_SCALE: u8 = 2;
pub const MAX_PIXEL_BUFFER_SIZE: u32 = u16::MAX as u32 *3;

#[derive(PartialEq)]
pub enum RefreshRateMode {
    Sync,
    Specified,
}

/// Main program struct
///
/// Notes:
/// Windowed mode resolution will be stored in `WIN_W` and `WIN_H`.
/// Console mode reolution are stored in `CON_W` and `CON_H`,
/// with special fields: `CON_MAX_W` and `CON_MAX_H` for maximum
/// console resolution allowed.
pub(crate) struct Program {
    /// for experimental purposes. Console mode only.
    DISPLAY: bool,

    SCALE: u8,

    /// Allow for resizing. Windowed mode only.
    RESIZE: bool,

    /// Scaling purposes
    WAYLAND: bool,

    pub pix: crate::graphics::Canvas,

    pub mode: Mode,

    pub MILLI_HZ: u32,
    pub REFRESH_RATE_MODE: RefreshRateMode,
    pub DURATIONS: [std::time::Duration; 4],
    pub REFRESH_RATE: std::time::Duration,

    pub WAV_WIN: usize,
    pub VOL_SCL: f32,
    pub SMOOTHING: f32,

    WIN_W: u16,
    WIN_H: u16,

    #[cfg(feature = "terminal")]
    pub CON_W: u16,
    #[cfg(feature = "terminal")]
    pub CON_H: u16,

    pub CON_MAX_W: u16,
    pub CON_MAX_H: u16,

    VIS: vislist::VisNavigator,

    visualizer: VisFunc,

    #[cfg(feature = "terminal")]
    pub flusher: crate::modes::console_mode::Flusher,

    SWITCH: Instant,
    AUTO_SWITCH: bool,
    AUTO_SWITCH_ITVL: Duration,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    VisualizerNext,
    VisualizerPrev,
    SwitchConMode,
    Resize(u16, u16),
    ConMax(i16, bool),
    Fps(i16, bool),
    FpsFrac(u32),
    SwitchVisList,
    VolUp,
    VolDown,
    SmoothUp,
    SmoothDown,
    WavUp,
    WavDown,
    AutoSwitch,
    Reset,
    Blank,
    Close,
}

impl Command {
    pub fn is_close_requested(&self) -> bool {
        *self == Command::Close
    }
}

impl Program {
    pub fn new() -> Self {
        let vislist_ = vislist::VisNavigator::new();
        let vis = vislist_.current_vis();

        let rate = std::time::Duration::from_micros(1_000_000 / DEFAULT_HZ);

        Self {
            DISPLAY: true,
            SCALE: DEFAULT_WIN_SCALE,
            RESIZE: false,

            mode: Mode::Win,

            WAYLAND: true,

            pix: crate::graphics::Canvas::new(DEFAULT_SIZE_WIN as usize, DEFAULT_SIZE_WIN as usize),

            MILLI_HZ: DEFAULT_MILLI_HZ,
            REFRESH_RATE_MODE: RefreshRateMode::Sync,
            REFRESH_RATE: rate,

            DURATIONS: [rate, rate * 4, rate * 16, Duration::from_millis(500)],

            VIS: vislist_,

            visualizer: vis.func(),

            #[cfg(feature = "terminal")]
            flusher: Program::print_alpha,

            SWITCH: Instant::now() + Duration::from_secs(8),
            AUTO_SWITCH: true,
            AUTO_SWITCH_ITVL: Duration::from_secs(8),

            WAV_WIN: DEFAULT_WAV_WIN,
            VOL_SCL: DEFAULT_VOL_SCL,
            SMOOTHING: DEFAULT_SMOOTHING,

            WIN_W: DEFAULT_SIZE_WIN,
            WIN_H: DEFAULT_SIZE_WIN,

            #[cfg(feature = "terminal")]
            CON_W: 50,
            #[cfg(feature = "terminal")]
            CON_H: 25,

            CON_MAX_W: 50,
            CON_MAX_H: 25,
        }
    }

    pub fn is_resizable(&self) -> bool {
        self.RESIZE
    }

    pub fn is_display_enabled(&self) -> bool {
        self.DISPLAY
    }

    pub fn is_wayland(&self) -> bool {
        self.WAYLAND
    }

    pub fn eval_command(&mut self, cmd: &Command) -> bool {
        use Command::*;

        if *cmd == Command::Blank {
            return false;
        }

        match cmd {
            &VisualizerNext => {
                self.change_visualizer(true);
                return true;
            }
            &VisualizerPrev => {
                self.change_visualizer(false);
                return true;
            }

            // &Resize(w, h) => {
            // self.update_size((w, h));
            // return true;
            // }
            #[cfg(feature = "terminal")]
            &SwitchConMode => {
                self.switch_con_mode();
                return true;
            }

            &SwitchVisList => {
                self.change_vislist();
                return true;
            }

            &Fps(fps, replace) => {
                self.change_fps(fps, replace);
            }

            &FpsFrac(milli_hz) => {
                self.change_fps_frac(milli_hz);
            }

            #[cfg(feature = "terminal")]
            &ConMax(d, replace) => {
                self.change_con_max(d, replace);
            }

            &VolUp => {
                self.increase_vol_scl();
            }
            &VolDown => {
                self.decrease_vol_scl();
            }

            &SmoothUp => {
                self.increase_smoothing();
            }
            &SmoothDown => {
                self.decrease_smoothing();
            }

            &WavUp => {
                self.increase_wav_win();
            }
            &WavDown => {
                self.decrease_wav_win();
            }

            &AutoSwitch => {
                self.toggle_auto_switch();
            }
            &Reset => {
                self.reset_parameters();
            }

            &_ => {}
        }

        false
    }

    pub fn eval_commands(&mut self, cmds: &mut Vec<Command>) -> bool {
        let mut redraw = false;
        for cmd in cmds.iter() {
            redraw |= self.eval_command(cmd);
        }
        cmds.clear();

        redraw
    }

    pub fn reset_switch(&mut self) {
        self.SWITCH = Instant::now() + self.AUTO_SWITCH_ITVL;
    }

    pub fn update_vis(&mut self) -> bool {
        let elapsed = Instant::now();
        if elapsed >= self.SWITCH && self.AUTO_SWITCH {
            self.SWITCH = elapsed + self.AUTO_SWITCH_ITVL;

            self.change_visualizer(true);

            return true;
        }

        false
    }

    pub fn change_fps(&mut self, amount: i16, replace: bool) {
        if replace {
            self.MILLI_HZ = amount as u32 * 1000;
        } else {
            self.MILLI_HZ = self
                .MILLI_HZ
                .saturating_add_signed(amount as i32 * 1000)
                .clamp(1, 200 * 1000);
        }

        self.change_fps_frac(self.MILLI_HZ);

        //self.FPS = ((self.FPS * (!replace) as u64) as i16 + amount).clamp(1, 144_i16) as u64;
        //self.REFRESH_RATE = std::time::Duration::from_micros(1_000_000 / self.FPS);
    }

    pub fn change_fps_frac(&mut self, fps: u32) {
        let fps_f = fps as f32 / 1000.0;
        let rate = (1_000_000.0 / fps_f) as u64;
        self.MILLI_HZ = fps;
        self.REFRESH_RATE = std::time::Duration::from_micros(rate);

        self.DURATIONS = [
            self.REFRESH_RATE,
            self.REFRESH_RATE * 4,
            self.REFRESH_RATE * 16,
            Duration::from_millis(500),
        ];
    }

    pub fn change_visualizer(&mut self, forward: bool) {
        let new_visualizer = if forward {
            self.VIS.next_vis()
        } else {
            self.VIS.prev_vis()
        };

        self.visualizer = new_visualizer.func();

        self.pix.clear();

        self.reset_switch();

        crate::audio::set_normalizer(new_visualizer.request());

        let vis_name = self.VIS.current_vis_name();
        let vis_list = self.VIS.current_list_name();

        //println!("Switching to {}\r", self.VIS[self.VIS_IDX].1);
        //std::io::stdout().flush().unwrap();

        self.print_message(format!(
            "Switching to {} in list {}\r\n",
            vis_name, vis_list
        ))
    }

    pub fn change_vislist(&mut self) {
        self.VIS.next_list();

        let new_visualizer = self.VIS.current_vis();

        self.visualizer = new_visualizer.func();

        self.pix.clear();

        self.reset_switch();

        crate::audio::set_normalizer(new_visualizer.request());

        let vis_name = self.VIS.current_vis_name();
        let vis_list = self.VIS.current_list_name();

        self.print_message(format!(
            "Switching to {} in list {}\r\n",
            vis_name, vis_list
        ))
    }

    pub fn print_message(&self, message: String) {
        use std::io::Write;
        let mut stdout = std::io::stdout();

        if self.DISPLAY && self.mode.is_con() {
            #[cfg(feature = "terminal")]
            use crossterm::{
                style::Print,
                terminal::{EnterAlternateScreen, LeaveAlternateScreen},
            };

            #[cfg(feature = "terminal")]
            let _ = crossterm::queue!(
                stdout,
                LeaveAlternateScreen,
                Print(message),
                EnterAlternateScreen
            );
        } else {
            print!("{}", message);
            let _ = stdout.flush();
        }
    }

    //pub fn update_fps(&mut self, new_fps: u64) {
    //self.MILLI_HZ = new_fps;
    //self.REFRESH_RATE = std::time::Duration::from_micros(1_000_000 / new_fps);
    //}

    pub fn update_size_win<T>(&mut self, s: (T, T))
    where
        usize: From<T>,
    {
        let size = (usize::from(s.0), usize::from(s.1));
        self.WIN_W = size.0 as u16;
        self.WIN_H = size.1 as u16;
        self.pix.resize(size.0, size.1);
    }

    pub fn update_size<T>(&mut self, s: (T, T))
    where
        T: Copy,
        u16: From<T>,
        usize: From<T>,
    {
        let size = (u16::from(s.0), u16::from(s.1));
        let sizeu = (usize::from(s.0), usize::from(s.1));

        match &self.mode {
            Mode::Win => {
                (self.WIN_W, self.WIN_H) = size;
                self.pix.resize(sizeu.0, sizeu.1);
            }

            #[cfg(feature = "minifb")]
            Mode::WinLegacy => {
                (self.WIN_W, self.WIN_H) = size;
                self.pix.resize(sizeu.0, sizeu.1);
            }

            _ => {
                #[cfg(feature = "terminal")]
                {
                    (self.CON_W, self.CON_H) = size;
                    let size = crate::modes::console_mode::rescale(size, self);
                    self.pix.resize(size.0 as usize, size.1 as usize);
                }
            }
        }
    }

    pub fn refresh(&mut self) {
        match &self.mode {
            Mode::Win | Mode::WinLegacy => {
                self.pix.resize(self.WIN_W as usize, self.WIN_H as usize)
            }
            _ => {
                #[cfg(feature = "terminal")]
                self.pix.resize(self.CON_W as usize, self.CON_H as usize);
            }
        }
    }

    pub fn force_render(&mut self) {
        let mut buf = crate::audio::get_buf();

        (self.visualizer)(self, &mut buf);

        self.pix.draw_to_self();
    }

    pub fn increase_vol_scl(&mut self) {
        self.VOL_SCL = (self.VOL_SCL * 1.2).clamp(0.0, 10.0);
    }

    pub fn decrease_vol_scl(&mut self) {
        self.VOL_SCL = (self.VOL_SCL / 1.2).clamp(0.0, 10.0);
    }

    pub fn increase_smoothing(&mut self) {
        self.SMOOTHING = (self.SMOOTHING + 0.05).clamp(0.0, 0.95);
    }

    pub fn decrease_smoothing(&mut self) {
        self.SMOOTHING = (self.SMOOTHING - 0.05).clamp(0.0, 0.95);
    }

    pub fn increase_wav_win(&mut self) {
        self.WAV_WIN = (self.WAV_WIN * 5 / 4).clamp(3, 500)
    }

    pub fn decrease_wav_win(&mut self) {
        self.WAV_WIN = (self.WAV_WIN * 4 / 5).clamp(3, 500)
    }

    pub fn toggle_auto_switch(&mut self) {
        self.AUTO_SWITCH ^= true;

        self.print_message(format!(
            "Auto switch is now {}\n",
            if self.AUTO_SWITCH { "on" } else { "off" }
        ));
    }

    pub fn reset_parameters(&mut self) {
        self.VOL_SCL = DEFAULT_VOL_SCL;
        self.SMOOTHING = DEFAULT_SMOOTHING;
        self.WAV_WIN = DEFAULT_WAV_WIN;

        #[cfg(feature = "terminal")]
        self.change_con_max(50, true);

        self.change_fps_frac(DEFAULT_MILLI_HZ);
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn scale(&self) -> u8 {
        self.SCALE
    }
}
