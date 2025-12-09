#![allow(dead_code)]

use std::time::Instant;

use crate::{
    data::log,
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
    fn perform(&mut self, prog: &mut crate::data::Program, stream: &mut crate::audio::AudioBuffer);
}

pub mod classic;
pub mod milk;

pub struct VisList {
    list: Vec<Box<dyn Visualizer>>,
    index: usize,
    last_updated: Instant,
    pub auto_switch: bool,
}

impl VisList {
    pub fn new() -> Self {
        Self {
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
            ],
            index: 0,
            last_updated: Instant::now(),
            auto_switch: true,
        }
    }

    pub fn next(&mut self) {
        self.list[self.index].defocus();

        self.index += 1;

        if self.index == self.list.len() {
            self.index = 0;
        }

        self.last_updated = Instant::now();
        self.list[self.index].focus();
    }

    pub fn prev(&mut self) {
        self.list[self.index].defocus();
        if self.index == 0 {
            self.index = self.list.len();
        }

        self.index -= 1;

        self.last_updated = Instant::now();
        self.list[self.index].focus();
    }

    pub fn update(&mut self) {
        if self.auto_switch {
            let now = Instant::now();
            if now - self.last_updated >= crate::data::DEFAULT_VIS_SWITCH_DURATION {
                self.next();
                self.last_updated = now;
            }
        }
    }

    pub fn get(&mut self) -> &mut dyn Visualizer {
        self.list[self.index].as_mut()
    }

    pub fn select_by_name(&mut self, name: &str) {
        let name = name.trim().to_lowercase();
        if let Some(i) = self.list.iter().position(|v| v.name() == name) {
            self.index = i;
            self.list[self.index].focus();
            return;
        }

        log::error!("Invalid name specified");
        log::info!("Possible visualizer values include: ");

        for v in &self.list {
            log::info!("    {}", v.name())
        }

        log::info!("");
    }
}
