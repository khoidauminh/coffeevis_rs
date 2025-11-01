use crate::{
    audio::AudioBuffer,
    data::{DEFAULT_SIZE_WIN, Program},
    graphics::{Canvas, P2, blend::Blend},
    math::{
        Cplx,
        rng::{random_float, random_int},
    },
};

#[derive(Copy, Clone)]
struct Bubble {
    pos: Cplx,
    radius: f32,
    color: u32,
}
