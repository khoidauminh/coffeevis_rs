#![allow(dead_code, unused_variables)]

use std::time::Instant;

mod helpers;
mod misc;

use crate::{
    data::{DEFAULT_VIS_SWITCH_DURATION, log},
    visualizers::{
        classic::{
            bars::{Bars, BarsCircle},
            lazer::Lazer,
            scopes::{Oscilloscope, Vectorscope},
            shaky::Shaky,
            slice::Slice,
            spectrum::Spectrum,
            vol_sweeper::VolSweeper,
            wave::Wave,
        },
        milk::rain::Rain,
        misc::example::Example,
    },
};

pub struct VisualizerConfig {
    pub normalize: bool,
}

pub trait Visualizer {
    fn config(&self) -> VisualizerConfig {
        VisualizerConfig { normalize: true }
    }

    fn name(&self) -> &'static str {
        "ducky"
    }

    fn focus(&mut self) {}
    fn defocus(&mut self) {}
    fn perform(
        &mut self,
        pix: &mut crate::graphics::PixelBuffer,
        key: &crate::data::KeyInput,
        stream: &mut crate::audio::AudioBuffer,
    );
}

pub mod classic;
pub mod milk;

pub struct VisList {
    list: Vec<Box<dyn Visualizer>>,
    index: usize,
    next_update: Instant,
    pub auto_switch: bool,
}

impl VisList {
    pub fn new() -> Self {
        Self {
            // In Rust, trait objects must be
            // wrapped in `Box`es due to
            // their unknown sizes.
            list: vec![
                Box::new(Spectrum::default()),
                Box::new(Oscilloscope {}),
                Box::new(Vectorscope {}),
                Box::new(Bars::default()),
                Box::new(BarsCircle::default()),
                Box::new(Lazer::default()),
                Box::new(Shaky::default()),
                Box::new(Slice::default()),
                Box::new(VolSweeper::default()),
                Box::new(Wave {}),
                Box::new(Rain::default()),
                // your visualizers goes here.
                // they can be placed in any order.
                // This one is the last.
                //
                // Hitting `b` in the visualizer window
                // will take you to this one
                Box::new(Example::default()),
            ],
            index: 0,
            next_update: Instant::now() + DEFAULT_VIS_SWITCH_DURATION,
            auto_switch: true,
        }
    }

    fn reset_timer(&mut self) {
        self.next_update = Instant::now() + DEFAULT_VIS_SWITCH_DURATION;
    }

    fn log(&self) {
        crate::data::log::info!("Switching to {}", self.list[self.index].name());
    }

    pub fn next(&mut self) {
        self.list[self.index].defocus();

        self.index += 1;

        if self.index == self.list.len() {
            self.index = 0;
        }

        self.list[self.index].focus();

        self.reset_timer();
        self.log();
    }

    pub fn prev(&mut self) {
        self.list[self.index].defocus();

        if self.index == 0 {
            self.index = self.list.len();
        }

        self.index -= 1;

        self.list[self.index].focus();

        self.reset_timer();
        self.log();
    }

    pub fn update(&mut self) {
        if self.auto_switch {
            let now = Instant::now();
            if now >= self.next_update {
                self.next();
                self.next_update = now + DEFAULT_VIS_SWITCH_DURATION;
            }
        }
    }

    pub fn get(&mut self) -> &mut dyn Visualizer {
        self.list[self.index].as_mut()
    }

    pub fn select_by_name(&mut self, name: &str) {
        if let Some(i) = self
            .list
            .iter()
            .position(|v| v.name().eq_ignore_ascii_case(&name))
        {
            self.index = i;
            self.list[self.index].focus();
            return;
        }

        log::error!("Invalid name specified: \"{}\"", name);
        log::info!("Possible visualizer values include: ");

        for v in &self.list {
            log::info!("    {}", v.name())
        }

        log::info!("");
    }
}
