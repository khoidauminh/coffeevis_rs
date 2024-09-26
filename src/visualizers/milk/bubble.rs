use std::sync::Mutex;

use crate::{
    audio::SampleArr,
    data::{Program, DEFAULT_SIZE_WIN},
    graphics::{blend::Blend, Canvas, P2},
    math::{
        rng::{random_float, random_int},
        Cplx,
    },
};

#[derive(Copy, Clone)]
struct Bubble {
    pos: Cplx,
    radius: f32,
    color: u32,
}
