use std::sync::Mutex;

use crate::{
	data::{Program, DEFAULT_SIZE_WIN},
	audio::SampleArr,
    graphics::{Canvas, P2, blend::Blend},
    math::{Cplx, rng::{random_int, random_float}}
};

#[derive(Copy, Clone)]
struct Bubble {
	pos: Cplx,
	radius: f64,
	color: u32,
	
}
