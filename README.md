# Coffee Music Visualizer
A small music visualizer written in Rust using Cpal and Minifb.

Built-in visualizers: 

![vectorscope](https://media2.giphy.com/media/LU7E8uu9g8zv6oBtdL/giphy.gif?cid=790b76115b652f5ae6329eb78eef396c970ed45a240a00bc&rid=giphy.gif&ct=g)
![shaky coffee](https://i.giphy.com/media/T99UxYb9ZbW0SR6OBD/giphy.webp)
![vol sweeper](https://media.giphy.com/media/Tuy6v2OgRl6e9DeYmc/giphy.gif)
![spectrum](https://media.giphy.com/media/QlrsTRVBv2kBscsTQC/giphy.gif)
![oscilloscope](https://media.giphy.com/media/WSnsugN74Qk3WJuoX3/giphy.gif)
![lazer](https://media.giphy.com/media/V1toUVISK2PQBqMbBs/giphy.gif)

## Installation 
In the source directory, run `cargo run --release` 

OR 

Run `cargo install --path <path to coffeevis_rs>`

Cargo should take care of the dependencies. 

This program has been run and tested OK on Linux Mint 20.2.

## How to use
|  Key | Descripttion |
| ------ | ------ |
| <kbd>Space</kbd> | iterate through visualizers |
| <kbd>Esc</kbd> | exit | 
| <kbd>-</kbd> / <kbd>+</kbd> | decrease/increase input volume |
| <kbd>\[</kbd> / <kbd>\]</kbd> | decrease/increase spectrum roughness |
| <kbd>;</kbd> / <kbd>'</kbd> | decrease/increase waveform coverage |
| <kbd>/</kbd> | reset all settings | 

