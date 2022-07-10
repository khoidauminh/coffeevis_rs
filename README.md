# Coffee Music Visualizer
A small GUI + TUI music visualizer written in Rust.

## Examples:

![console mode](https://media.giphy.com/media/EjtGZZXlqdKeZ5ctBW/giphy.gif)
![windowed mode](https://media.giphy.com/media/ahIRySAELqiI7kIUI2/giphy.gif)

## Dependencies
cpal = "0.13.5"

minifb = "0.23.0"

crossterm = "0.24.0"

drawille = "0.3.0"

## Platform support
Coffeevis currently runs well on Debian 11 i3 and is expected to run on other Linux distributions.

Windows, MacOS and BSD support is not tested.

## Installation
Run `cargo install coffeevis`

## Usage
Run in windowed mode: `coffeevis`

Run in console mode: `coffeevis --con`

## Notes
Coffeevis prints text directly to stdout, rendering may be heavy depending on your terminal.

A terminal with GPU-accelerated support is recommended (i.e Alacritty, Kitty, Wezterm, ...)

A maximum resolution is built into the console mode (default: 50x50). Coffeevis will render in the center of the screen if terminal dimensions are larger than the limit.

## Keyboard shortcuts

### Global
|  Key | Descripttion |
| ------ | ------ |
| <kbd>Space</kbd> | iterates through visualizers |
| <kbd>q</kbd> | exits |
| <kbd>/</kbd> | resets all settings (fps is unaffected by this)|
| <kbd>-</kbd> / <kbd>+</kbd> | decreases/increases input volume |
| <kbd>\[</kbd> / <kbd>\]</kbd> | decreases/increases spectrum roughness |
| <kbd>;</kbd> / <kbd>'</kbd> | decreases/increases amount of samples into input (works for wave-based visualizers only) |
| <kbd>\\</bkd> | toggles auto switching (default: ON, 8 seconds) |
| <kbd>7</kbd> / <kbd>8</kbd> | decreases/increases fps by 5 (default: 60) |
| <kbd>1</kbd> ... <kbd>6</kbd> | changes fps to 10 ... 60 respectively |

### Console mode
|  Key | Descripttion |
| ------ | ------ |
| <kbd>.</kbd> | toggles between text rendering and braille rendering |
| <kbd>,</kbd> | in text rendering, toggles between ascii character set and block character set |
| <kbd>9</kbd> / <kbd>0</kbd> | decreases/increases maximum resolution |
