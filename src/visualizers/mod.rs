#![allow(dead_code)]

pub type VisFunc = fn(&mut crate::data::Program, &mut crate::audio::SampleArr);

pub mod classic;
pub mod milk;
