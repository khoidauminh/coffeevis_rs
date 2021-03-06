pub type VisFunc = fn(&mut [u32], &[(f32, f32)], &mut crate::config::Parameters) -> ();

mod dash_line;
mod cross;
pub mod oscilloscope;
pub mod spectrum;
pub mod lazer;
pub mod shaky_coffee;
pub mod vol_sweeper;
pub mod ring;
pub mod bars;
pub mod experiment1;
pub mod wave;
//pub mod proc;
