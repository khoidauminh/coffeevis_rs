use crate::visualizers::{VisFunc, classic::*, milk};
use crate::math::{increment_index, decrement};

#[derive(Copy, Clone)]
pub struct Visualizer {
	func: VisFunc,
	name: &'static str,
	request_auto_gain: bool,
}

macro_rules! define_visualizer {
	($visfunc:expr, $name:literal, $request:expr) => {
		Visualizer {
			func: $visfunc,
			name: $name,
			request_auto_gain: $request
		}
	}
}

impl Visualizer {
	/*pub const fn new(f: VisFunc, name: &'static str, request: bool) -> Self {
		Self {
			func: f,
			name: name,
			request_auto_gain: request,
		}
	}*/
	
	pub fn func(&self) -> VisFunc {
		self.func
	}
	
	pub fn name(&self) -> &'static str {
		self.name
	}
	
	pub fn request(&self) -> bool {
		self.request_auto_gain
	}
}

pub struct VisList {
	list: &'static [Visualizer],
	name: &'static str,
}

impl VisList {
	pub const fn new(list: &'static [Visualizer], name: &'static str) -> Self {
		Self {
			list: list,
			name: name
		}
	}
}

pub struct VisNavigator {
	structure: &'static [VisList],
	index_vis: usize,
	index_list: usize,
}

impl VisNavigator {
	pub fn new() -> Self {
		Self {
			structure: VIS_MENU,
			index_vis: 0,
			index_list: 0,
		}
	}
	
	pub fn current_vis(&self) -> Visualizer {
		self.structure[self.index_list].list[self.index_vis]
	}
	
	pub fn current_list_len(&self) -> usize {
		self.structure[self.index_list].list.len()
	}
	
	pub fn num_of_lists(&self) -> usize {
		self.structure.len()
	}
	
	pub fn current_list_name(&self) -> &'static str {
		self.structure[self.index_list].name
	}
	
	pub fn current_vis_name(&self) -> &'static str {
		self.structure[self.index_list].list[self.index_vis].name
	}
	
	pub fn next_vis(&mut self) -> Visualizer {
		let list_size = self.current_list_len();
		
		self.index_vis = increment_index(self.index_vis, list_size);
		
		self.current_vis()
	}
	
	pub fn prev_vis(&mut self) -> Visualizer {
		let list_size = self.current_list_len();
		
		self.index_vis = decrement(self.index_vis, list_size);
		
		self.current_vis()	
	}
	
	pub fn next_list(&mut self) {
		let size = self.num_of_lists();
		self.index_list = increment_index(self.index_list, size);
		self.index_vis = 0;
	}
	
	pub fn prev_list(&mut self) {
		let size = self.num_of_lists();
		self.index_list = increment_index(self.index_list, size);
		self.index_vis = 0;
	}
}

pub const VIS_MENU: &[VisList] = 
&[
	VisList::new(VIS_CLASSIC, "Classic"),
	VisList::new(VIS_MILK, "Milk"),
];


pub const VIS_CLASSIC: &[Visualizer] = 
&[
	// scopes::draw_phase,
	define_visualizer!(scopes::draw_vectorscope, "Vectorscope", true),
	define_visualizer!(scopes::draw_oscilloscope, "Oscilloscope", true),
	define_visualizer!(ring::draw_ring, "Ring", true),
	define_visualizer!(spectrum::draw_spectrum, "Spectrum", true),
	define_visualizer!(bars::draw_bars, "Bars", false),
	define_visualizer!(bars::draw_bars_circle, "Bars (Circle)", false),
	define_visualizer!(wavelet::draw_wavelet, "Wavelet", true),
	define_visualizer!(shaky::draw_shaky, "Shaky", false),
	define_visualizer!(lazer::draw_lazer, "Lazer", true),
	define_visualizer!(wave::draw_wave, "Wave", true),
	define_visualizer!(vol_sweeper::draw_vol_sweeper, "Volume sweep", false),
/*	[vol_sweeper::draw_vol_sweeper, "Vol sweeper"],
	[// experiment1::draw_exp1,
	[// experiment1::draw_f32,
	[wave::draw_wave,  */
];

pub const VIS_MILK: &[Visualizer] = 
&[
	define_visualizer!(milk::rain::draw, "Rain", false)
];
