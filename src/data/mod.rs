use crossterm::{
	queue, QueueableCommand,
	terminal::{self, Clear, ClearType}
};

pub mod reader;
pub mod vislist;
pub mod log;

use std::time::{Duration, Instant};
use crate::modes::{Mode, console_mode::Flusher};
use crate::graphics::Image;
use crate::VisFunc;
use vislist::VIS_MENU;

pub const SAMPLE_RATE_MAX: usize = 384000;
pub const SAMPLE_RATE: usize = 44100;

pub const POWER: usize = 10;
pub const FFT_POWER: usize = POWER;
pub const SAMPLE_SIZE: usize = (3 << POWER) - 1;
pub const FFT_SIZE: usize = 1 << (FFT_POWER-1);

pub const DEFAULT_FPS: u8 = 144;
pub const DEFAULT_WAV_WIN: usize = { let t = (SAMPLE_RATE / 15000); if t > 5 {t} else {5} };
pub const ROTATE_SIZE: usize = SAMPLE_SIZE * (DEFAULT_WAV_WIN) / 100; // 3539; 
pub const PHASE_OFFSET: usize = 2048 * SAMPLE_RATE / SAMPLE_RATE_MAX;
pub const INCREMENT: usize = (DEFAULT_WAV_WIN / 12) | 1;
pub const DEFAULT_VOL_SCL: f32   = 0.86;
pub const DEFAULT_SMOOTHING: f32 = 0.65;

/// How long silence has happened to trigger render slow down. 
pub const SILENCE_LIMIT: u8 = 7;
#[doc(hidden)]
pub const IDLE_REFRESH_RATE: Duration = Duration::from_millis(1000/24);

pub const DEFAULT_SIZE_WIN: u16 = 144;
#[doc(hidden)]
pub const ERR_MSG: &str = "Configration error at line";

pub const WIN_SCALE: usize = 2;

pub static IMAGE: &[u8; 1442] = include_bytes!("coffee_pixart_2x.png");

/// Status of the audio system.
/// The transitional stages only exist for one iteration in the main loop.
#[doc(hidden)]
#[derive(Copy, Clone)]
pub enum State {
    /// Transitional stage, program just received audio infomation.
    Waken = 0, 
    Active = 1, 
    /// Transitional stage, program prepares for slowdown.
    Waiting = 2, 
    /// Slowdown.
    Idle = 3,
}

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
	pub DISPLAY: bool,
	pub SCALE: u8,

	/// Allow for resizing. Windowed mode only.
	pub RESIZE: bool,

	pub pix: crate::graphics::Canvas,

	pub mode: Mode,

	pub FPS: u64,
	pub REFRESH_RATE: std::time::Duration,
	pub WAV_WIN: usize,
	pub VOL_SCL: f32,
	pub SMOOTHING: f32,

	pub WIN_W: u16,
	pub WIN_H: u16,
	
	pub CON_W: u16, 
	pub CON_H: u16,
	pub CON_MAX_W: u16,
	pub CON_MAX_H: u16,

	pub VIS: vislist::VisNavigator,

	pub visualizer: VisFunc,
	pub flusher: Flusher,

	pub SWITCH: Instant,
    pub AUTO_SWITCH: bool,
    pub AUTO_SWITCH_ITVL: Duration,

    pub IMG: Image,

    pub exp: u8,

    /// Triggers render when is 0.
    ///
    /// When there is no new audio information, the program triggers
    /// slowdown to reduce processor consumption. 
    /// Over the `audio` module, there is an u8 global variable called 
    /// `NO_SAMPLE`. Everytime audio input returns silence, `NO_SAMPLE`
    /// is incremented, saturating at 255. When there's new audio, 
    /// it's immediately dropped back to 0.
    ///
    /// `render_trigger` is incremented in every iteration of the main 
    /// loop and wraps around if it exceeds NO_SAMPLE. On active state, 
    /// NO_SAMPLE is 0 and therefore program renders at every loop 
    /// iteration.
    ///
    /// This is to reduce processor power when the program is idle, 
    /// while keeping the main loop and input evaluation running at 
    /// low latency.
    render_trigger: u8,
    pub state: State,

    pub msg: Result<(), String>,
    pub msg_timeout: u64,
}

impl Program {
	pub fn new() -> Self {
		let vislist_ = vislist::VisNavigator::new();
		let vis = vislist_.current_vis();
		
		Self {
			DISPLAY: true,
			SCALE: 1,
			RESIZE: false,

			mode: Mode::Win,

			pix: crate::graphics::Canvas::new(DEFAULT_SIZE_WIN as usize, DEFAULT_SIZE_WIN as usize), 

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

			WIN_W: DEFAULT_SIZE_WIN,
			WIN_H: DEFAULT_SIZE_WIN,
			CON_W: 50,
			CON_H: 25, 
		    CON_MAX_W: 50,
            CON_MAX_H: 25,

			IMG: crate::data::reader::prepare_image(IMAGE),

			render_trigger: 0u8,
			state: State::Waiting,

			msg: Ok(()),
			msg_timeout: 0,

			exp: 0,
		}
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
		self.pix.width = DEFAULT_SIZE_WIN as usize;
		self.pix.height = DEFAULT_SIZE_WIN as usize;
		self.mode = Mode::Win;
		self.refresh();
		self
	}

	#[cfg(feature = "winit")]
	pub fn as_winit(mut self) -> Self {
		self.pix.width = DEFAULT_SIZE_WIN as usize;
		self.pix.height = DEFAULT_SIZE_WIN as usize;
		self.mode = Mode::Winit;
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

		use std::io::Write;
		use crossterm::{self, 
			terminal::{
				EnterAlternateScreen, 
				LeaveAlternateScreen
			}, 
			style::Print
		};
		
		if
			self.DISPLAY && 
			self.mode != Mode::Win
		{
			crossterm::queue!(
				std::io::stdout(),
				LeaveAlternateScreen,
				Print(format!("Switching to {} in list {}\r\n", vis_name, vis_list)),
				EnterAlternateScreen
			);
		} else {
			println!("Switching to {} in list {}", vis_name, vis_list);
		}

		//println!("Switching to {}\r", self.VIS[self.VIS_IDX].1);
		//std::io::stdout().flush().unwrap();
	}
	
	pub fn update_fps(&mut self, new_fps: u64) {
	    self.FPS = new_fps;
	    self.REFRESH_RATE = std::time::Duration::from_micros(1_000_000 / new_fps as u64);
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
			Mode::Win => (self.WIN_W, self.WIN_H) = size,

			#[cfg(feature = "winit")]
			Mode::Winit => (self.WIN_W, self.WIN_H) = size,

			_ => {
				(self.CON_W, self.CON_H) = size;
				size = crate::modes::console_mode::rescale(size, self);
			}
		}
		
		self.pix.resize(size.0 as usize, size.1 as usize);
	}

	pub fn refresh(&mut self) {
		match &self.mode {
			Mode::Win	=> self.pix.resize(self.WIN_W as usize, self.WIN_H as usize),

			#[cfg(feature = "winit")]
			Mode::Winit	=> self.pix.resize(self.WIN_W as usize, self.WIN_H as usize),

			_ 			=> self.pix.resize(self.CON_W as usize, self.CON_H as usize),
		}

		// self.pix.update();
	}

	pub fn clear_pix(&mut self) {
		self.pix.clear();
        //self.pix.fade(96);
		//self.timed_clear();
	}

	pub fn clear_pix_alpha(&mut self, alpha: u8) {
		self.pix.fade(alpha);
	}

	pub fn timed_clear(&mut self) {
	    if self.exp == 0 { self.pix.clear() }
	    else { self.pix.fade(216) } 
	    self.exp = crate::math::increment(self.exp, 4);
	}
    /*
	pub fn advance_timer(&mut self) {
	}*/

    /// Automatically renders on trigger.
	pub fn render(&mut self) {
        self.update_timer();
        if self.render_trigger() {
            self.force_render();
        }
	}
    
    pub fn render_trigger(&self) -> bool {
        // crate::audio::get_no_sample() >
        self.render_trigger == 0
    }

    pub fn update_timer(&mut self) {
        let sample = crate::audio::get_no_sample();
        /*
        if sample == 255 {
            self.render_trigger = 255;
            return;
        }*/

        self.render_trigger =
			crate::math::increment(
				self.render_trigger,
                sample >> 3
			);
    }

	pub fn force_render(&mut self) {
		let mut buf = crate::audio::get_buf();

	    // if self.render_trigger == 0 {
			(self.visualizer)(self, &mut buf);
		//}
	}

	pub fn get_state(&self) -> State {
        self.state
	}

	pub fn update_state(&mut self) {
        use State::*;
        let silence = crate::audio::get_no_sample() > SILENCE_LIMIT;
        self.state = match (silence, &self.state) {
            (true, Active)   => Waiting,
            
            (true, Waiting)  => Idle,
            
            (true, Waken) | (true, Idle) 
                             => Idle,
                             
            (false, Waken)   => Waken,
            
            (false,  Active) => self.state,
            
            (false, Waiting) | (false, Idle)
                             => Waken,
        }
	}
}