use crate::math::{decrement, increment};
use crate::visualizers::{classic::*, milk, VisFunc};

#[derive(Copy, Clone)]
pub struct Visualizer {
    func: VisFunc,
    name: &'static str,
    request_normalizer: bool,
}

macro_rules! define_visualizer_struct {
    ($visfunc:expr, $name:literal, $request:expr) => {
        Visualizer {
            func: $visfunc,
            name: $name,
            request_normalizer: $request,
        }
    };
}

#[macro_export]
macro_rules! define_visualizer {
    ($func_name:ident, $func_body:block) => {
        pub fn $func_name(prog: &mut $crate::data::Program, stream: &mut $crate::audio::SampleArr) {
            $func_body
        }
    };
}

#[macro_export]
macro_rules! vis_para {
    () => { prog: &mut $crate::data::Program, stream: &mut $crate::audio::SampleArr };
}

impl Visualizer {
    /*pub const fn new(f: VisFunc, name: &'static str, request: bool) -> Self {
        Self {
            func: f,
            name: name,
            request_normalizer: request,
        }
    }*/

    pub fn func(&self) -> VisFunc {
        self.func
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn request(&self) -> bool {
        self.request_normalizer
    }
}

pub struct VisList {
    list: &'static [Visualizer],
    name: &'static str,
    pub current_index: usize,
}

impl VisList {
    pub const fn new(list: &'static [Visualizer], name: &'static str) -> Self {
        Self {
            list,
            name,
            current_index: 0,
        }
    }
}

pub struct VisNavigator {
    structure: [VisList; 2],
    index_vis: usize,
    index_list: usize,
}

impl VisNavigator {
    pub fn new() -> Self {
        Self {
            structure: [
                VisList::new(VIS_CLASSIC, "Classic"),
                VisList::new(VIS_MILKY, "Milky"),
            ],
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

        self.index_vis = increment(self.index_vis, list_size);

        let current = self.current_vis();

        crate::audio::set_normalizer(current.request());

        current
    }

    pub fn prev_vis(&mut self) -> Visualizer {
        let list_size = self.current_list_len();

        self.index_vis = decrement(self.index_vis, list_size);

        let current = self.current_vis();

        crate::audio::set_normalizer(current.request());

        current
    }

    pub fn save_index(&mut self) {
        self.structure[self.index_list].current_index = self.index_vis;
    }

    pub fn load_index(&mut self) {
        self.index_vis = self.structure[self.index_list].current_index;
    }

    pub fn next_list(&mut self) {
        let size = self.num_of_lists();
        self.save_index();
        self.index_list = increment(self.index_list, size);
        self.load_index();
    }

    pub fn prev_list(&mut self) {
        let size = self.num_of_lists();
        self.save_index();
        self.index_list = increment(self.index_list, size);
        self.load_index();
    }

    pub fn switch_by_index(&mut self, index: usize) -> Visualizer {
        let current_list = self.structure[self.index_list].list;

        if let Some(_vis) = current_list.get(index) {
            self.index_vis = index;

            return self.current_vis();
        }

        self.current_vis()
    }

    pub fn switch_by_name(&mut self, name: &str) -> Visualizer {
        let name = name.to_lowercase();
        let mut found = false;
        let mut selected_vis = VIS_CLASSIC[0];
        let mut selected_vis_index = 0;

        let iter = VIS_CLASSIC.iter().chain(VIS_MILKY.iter());

        for (i, vis) in iter.clone().enumerate() {
            if vis.name().to_lowercase() == name {
                found = true;
                selected_vis = *vis;
                selected_vis_index = i;
                break;
            }
        }

        if !found {
            eprintln!(
                "\nCan't find the specified visualizer.\n\
            Available visualizers (case insensitive):\n\
            "
            );

            for vis in iter {
                eprintln!("{}", vis.name());
            }

            eprintln!();

            return selected_vis;
        }

        if selected_vis_index < VIS_CLASSIC.len() {
            self.index_vis = selected_vis_index;
            self.index_list = 0;
        } else {
            self.index_vis = selected_vis_index - VIS_CLASSIC.len();
            self.index_list = 1;
        }

        selected_vis
    }
}

pub const VIS_CLASSIC: &[Visualizer] = &[
    define_visualizer_struct!(scopes::draw_vectorscope, "Vectorscope", true),
    define_visualizer_struct!(scopes::draw_oscilloscope3, "Oscilloscope", true),
    define_visualizer_struct!(ring::draw_ring, "Ring", true),
    define_visualizer_struct!(spectrum::draw_spectrum, "Spectrum", true),
    define_visualizer_struct!(bars::draw_bars, "Bars", true),
    define_visualizer_struct!(bars::draw_bars_circle, "Bars (Circle)", true),
    define_visualizer_struct!(wavelet::draw_wavelet, "Wavelet", true),
    define_visualizer_struct!(shaky::draw_shaky, "Shaky", false),
    define_visualizer_struct!(lazer::draw_lazer, "Lazer", true),
    define_visualizer_struct!(wave::draw_wave, "Wave", true),
    define_visualizer_struct!(vol_sweeper::draw_vol_sweeper, "Volume sweep", false),
    define_visualizer_struct!(slice::draw_slice, "Slice", false),
    // define_visualizer_struct!(tests::draw_quick_sort_iter, "Test", true),
    /*  [vol_sweeper::draw_vol_sweeper, "Vol sweeper"],
    [// experiment1::draw_exp1,
    [// experiment1::draw_f32,
    [wave::draw_wave,  */
];

pub const VIS_MILKY: &[Visualizer] = &[define_visualizer_struct!(milk::rain::draw, "Rain", false)];
