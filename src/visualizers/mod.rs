#![allow(dead_code)]

pub type VisFunc = fn(&mut crate::Program, &mut crate::AudioBuffer);

pub mod classic;
pub mod milk;
