# Coffee Music Visualizer
A small music visualizer written in Rust using Cpal and Minifb.

Built-in visualizers: 
vectorscope, shaky coffee, vol sweeper, spectrum, oscilloscope, lazer

### Installation 
In the source directory, run `cargo run --release` 

OR 

Run `cargo install --path <path to coffeevis_rs>`

Cargo should take care of the dependencies. 

This program has been run and tested OK on Linux Mint 20.2.

### How to use
|  Key | Descripttion |
| ------ | ------ |
| Space | iterate through visualizers |
| Esc | exit | 
| - / + | decrease/increase input volume |
| \[ / \] | decrease/increase spectrum smoothing |
| ; / ' | decrease/increase waveform coverage |
| / | reset all settings | 
