pub mod reader;
pub mod vislist;

use std::time::{Duration, Instant};
use crate::modes::{Mode, console_mode::Flusher};

use crate::VisFunc;

pub const SAMPLE_RATE_MAX: usize = 384000;
pub const SAMPLE_RATE: usize = 44100;

pub const POWER: usize = 13;
pub const FFT_POWER: usize = 9;
pub const SAMPLE_SIZE: usize = 1 << POWER;
pub const FFT_SIZE: usize = 1 << FFT_POWER;

pub const INCREMENT: usize = 2;
pub const DEFAULT_FPS: u8 = 144;
pub const DEFAULT_WAV_WIN: usize = 144 * INCREMENT;
pub const DEFAULT_ROTATE_SIZE: usize = 289; // 3539;
pub const PHASE_OFFSET: usize = SAMPLE_RATE / 50 / 4;
pub const DEFAULT_VOL_SCL: f64   = 0.86;
pub const DEFAULT_SMOOTHING: f64 = 0.65;

/// How long silence has happened to trigger render slow down.
pub const SILENCE_LIMIT: u8 = 7;
#[doc(hidden)]
pub const IDLE_REFRESH_RATE: Duration = Duration::from_millis(1000/24);

pub const DEFAULT_SIZE_WIN: u16 = 60;
#[doc(hidden)]
pub const ERR_MSG: &str = "Configration error at line";

pub const DEFAULT_WIN_SCALE: u8 = 3;

/// Main program struct
///
/// Notes:
/// Windowed mode resolution will be stored in `WIN_W` and `WIN_H`.
/// Console mode reolution are stored in `CON_W` and `CON_H`,
/// with special fields: `CON_MAX_W` and `CON_MAX_H` for maximum
/// console resolution allowed.
pub struct Program
{
    /// for experimental purposes. Console mode only.
	DISPLAY: bool,
	SCALE: u8,

	/// Allow for resizing. Windowed mode only.
	RESIZE: bool,
	
	/// Scaling purposes
	WAYLAND: bool,

	pub pix: crate::graphics::Canvas,

	mode: Mode,
	
	pub transparency: u8,
	pub background: u32,

	pub FPS: u64,
	pub REFRESH_RATE: std::time::Duration,
	pub WAV_WIN: usize,
	pub VOL_SCL: f64,
	pub SMOOTHING: f64,
	pub ROTATE_SIZE: usize,

	WIN_W: u16,
	WIN_H: u16,

	pub CON_W: u16,
	pub CON_H: u16,
	pub CON_MAX_W: u16,
	pub CON_MAX_H: u16,

	VIS: vislist::VisNavigator,

	visualizer: VisFunc,
	pub flusher: Flusher,

	SWITCH: Instant,
    AUTO_SWITCH: bool,
    AUTO_SWITCH_ITVL: Duration,
}

impl Program {
	pub fn new() -> Self {
		let vislist_ = vislist::VisNavigator::new();
		let vis = vislist_.current_vis();

		Self {
			DISPLAY: true,
			SCALE: DEFAULT_WIN_SCALE,
			RESIZE: false,

			mode: Mode::Win,
			
			WAYLAND: true,
			
			transparency: 255,
			background: 0x00_00_00_00,

			pix: crate::graphics::Canvas::new(DEFAULT_SIZE_WIN as usize, DEFAULT_SIZE_WIN as usize, 0x00_00_00_00),

	        FPS: DEFAULT_FPS as u64,
            REFRESH_RATE: std::time::Duration::from_micros(1_000_000 / DEFAULT_FPS as u64),

            VIS: vislist_,

	        visualizer: vis.func(),
	        flusher: Program::print_alpha,

			SWITCH: Instant::now() + Duration::from_secs(8),
			AUTO_SWITCH: true,
			AUTO_SWITCH_ITVL: Duration::from_secs(8),

			WAV_WIN: DEFAULT_WAV_WIN,
			VOL_SCL: DEFAULT_VOL_SCL,
			SMOOTHING: DEFAULT_SMOOTHING,
			ROTATE_SIZE: DEFAULT_ROTATE_SIZE,

			WIN_W: DEFAULT_SIZE_WIN,
			WIN_H: DEFAULT_SIZE_WIN,
			CON_W: 50,
			CON_H: 25,
		    CON_MAX_W: 50,
            CON_MAX_H: 25,
		}
	}
	
	pub fn is_display_enabled(&self) -> bool {
		self.DISPLAY
	}
	
	pub fn is_wayland(&self) -> bool {
		self.WAYLAND
	}

	pub fn as_con(mut self) -> Self {
		match self.mode {
			Mode::Win => self.set_con_mode(Mode::ConAlpha),
			_ 		  => {},
		}
		self
	}

	pub fn as_con_force(mut self, mode: Mode) -> Self {
		 self.set_con_mode(mode);
		 self
	}

	pub fn as_win(mut self) -> Self {
		self.pix.resize(
			DEFAULT_SIZE_WIN as usize,
			DEFAULT_SIZE_WIN as usize
		);
		self.mode = Mode::Win;
		self.refresh();
		self
	}

	pub fn reset_switch(&mut self) {
		self.SWITCH = Instant::now() + self.AUTO_SWITCH_ITVL;
	}

	pub fn update_vis(&mut self) {
		let elapsed = Instant::now();
		if elapsed >= self.SWITCH && self.AUTO_SWITCH
		{
			self.SWITCH = elapsed + self.AUTO_SWITCH_ITVL;

			self.change_visualizer(true);
		}
	}
	
	pub fn change_fps(&mut self, amount: i16, replace: bool) {
		self.FPS =
			((self.FPS * (!replace) as u64) as i16 + amount)
			.clamp(1, 144_i16)
			as u64
			;
		self.REFRESH_RATE = std::time::Duration::from_micros(1_000_000 / self.FPS);
	}

	pub fn change_visualizer(&mut self, forward: bool) {

		let new_visualizer =  if forward {
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

		self.print_message(format!("Switching to {} in list {}\r\n", vis_name, vis_list))
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
		
		self.print_message(format!("Switching to {} in list {}\r\n", vis_name, vis_list))
	}

	pub fn print_message(&self, message: String) {
        use std::io::Write;
        use crossterm::{
			terminal::{
				EnterAlternateScreen,
				LeaveAlternateScreen
			},
			style::Print
		};

        let mut stdout = std::io::stdout();

		if
			self.DISPLAY &&
			self.mode.is_con()
		{
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

	pub fn update_fps(&mut self, new_fps: u64) {
	    self.FPS = new_fps;
	    self.REFRESH_RATE = std::time::Duration::from_micros(1_000_000 / new_fps);
	}

	pub fn update_size_win<T>(&mut self, s: (T, T))
	where usize: From<T> {
		let size = (usize::from(s.0), usize::from(s.1));
		self.WIN_W = size.0 as u16;
		self.WIN_H = size.1 as u16;
		self.pix.resize(size.0, size.1);
	}

	pub fn update_size<T>(&mut self, s: (T, T))
	where u16: From<T> {
		let mut size = (u16::from(s.0), u16::from(s.1));

		match &self.mode {
			Mode::Win | Mode::WinLegacy => (self.WIN_W, self.WIN_H) = size,

			_ => {
				(self.CON_W, self.CON_H) = size;
				size = crate::modes::console_mode::rescale(size, self);
			}
		}

		self.pix.resize(size.0 as usize, size.1 as usize);
	}

	pub fn refresh(&mut self) {
		match &self.mode {
			Mode::Win | Mode::WinLegacy 
				=> self.pix.resize(self.WIN_W as usize, self.WIN_H as usize),
			_ 	=> self.pix.resize(self.CON_W as usize, self.CON_H as usize),
		}
	}

	pub fn force_render(&mut self) {
		let mut buf = crate::audio::get_buf();

	    // if self.render_trigger == 0 {
			(self.visualizer)(self, &mut buf);
			self.pix.draw_to_self();
		//}
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
		self.WAV_WIN = (self.WAV_WIN + 3).clamp(3, 50)
	}
	
	pub fn decrease_wav_win(&mut self) {
		self.WAV_WIN = (self.WAV_WIN - 3).clamp(3, 50)
	}
	
	pub fn toggle_auto_switch(&mut self) {
		self.AUTO_SWITCH ^= true;
		self.print_message(format!("Auto switch is now {}\n",
		        if self.AUTO_SWITCH {"on"}
		        else {"off"}
		    )
		)
	}
	
	pub fn reset_parameters(&mut self) {
		self.VOL_SCL = DEFAULT_VOL_SCL;
		self.SMOOTHING = DEFAULT_SMOOTHING;
		self.WAV_WIN = DEFAULT_WAV_WIN;
	}
	
	pub fn switch_con_mode(&mut self) {
		// self.mode.next_con();

		self.mode = match self.mode {
			Mode::ConAlpha => {
				self.flusher = Program::print_block;
				Mode::ConBlock
			},
			Mode::ConBlock => {
				self.flusher = Program::print_brail;
				Mode::ConBrail
			},
			Mode::ConBrail => {
				self.flusher = Program::print_alpha;
				Mode::ConAlpha
			},

			Mode::Win       => Mode::Win,
			Mode::WinLegacy => Mode::WinLegacy,
		};

		self.refresh_con();
	}

	pub fn set_con_mode(&mut self, mode: Mode) {
		match mode {
			Mode::ConAlpha => self.flusher = Program::print_alpha,
			Mode::ConBlock => self.flusher = Program::print_block,
			Mode::ConBrail => self.flusher = Program::print_brail,
			_ => {},
		}
		self.mode = mode;
		self.refresh_con();
	}
	
	pub fn mode(&self) -> Mode {
		self.mode
	}
	
	pub fn scale(&self) -> u8 {
		self.SCALE
	}
}
