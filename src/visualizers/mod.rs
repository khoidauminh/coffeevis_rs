#![allow(dead_code)]

pub type VisFunc = fn(&mut crate::Program, &mut crate::audio::SampleArr);

pub mod classic;
pub mod milk;
