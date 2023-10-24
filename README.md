# Coffee Music Visualizer
A small GUI + TUI music visualizer written in Rust.

## Examples (may be old):

![console mode](https://media.giphy.com/media/EjtGZZXlqdKeZ5ctBW/giphy.gif)
![windowed mode](https://media.giphy.com/media/ahIRySAELqiI7kIUI2/giphy.gif)

## Platform support
Coffeevis is built for and runs on Linux.
Windows, MacOS and BSD support is not available.

## Installation
Run `cargo install coffeevis`

## Usage
Coffeevis supports temporary options at launch

| Option | Value (example) | Description |
| ------ | ------ | ------ |
| --win-legacy |  | opens window with minifb (coffeevis now runs with winit by default) |
| --ascii<br />--block<br />--braille | | runs in the terminal |
| --transparent | 192 | sets transparency, no value indicates full transparency (value currently ignored for now) |
| --auto-switch | true<br />false | toggles auto visualizer switching |
| --size | 80x80 | sets resolution in window mode |
| --scale | 2 | upscales in window mode |
| --fps | 60 | sets refresh rate |
| --resizable | | allows resizing in window mode (not recommended) |
| --max-con-size | 50x50 | sets maximum resolution in terminal mode |

Currently reading from a file is not supported. It is recommended to launch coffeevis in a script.

## Notes
Coffeevis prints text directly to stdout, rendering may be heavy depending on your terminal.

A terminal with GPU-accelerated support is recommended (i.e Alacritty, Kitty, Wezterm, ...)

A maximum resolution is built into the console mode (default: 50x50). Coffeevis will render in the center of the screen if terminal dimensions are larger than the limit.

## Keyboard shortcuts

### Global
|  Key | Description |
| ------ | ------ |
| <kbd>Space</kbd> | iterates through visualizers |
| <kbd>q</kbd> | exits |
| <kbd>/</kbd> | resets all settings |
| <kbd>-</kbd> / <kbd>+</kbd> | decreases/increases input volume |
| <kbd>\[</kbd> / <kbd>\]</kbd> | decreases/increases spectrum roughness |
| <kbd>;</kbd> / <kbd>'</kbd> | decreases/increases amount of samples into input (works for wave-based visualizers only) |
| <kbd>\\</bkd> | toggles auto switching (default: ON, 8 seconds) |

### Terminal mode
|  Key | Description |
| ------ | ------ |
| <kbd>.</kbd> | toggles between ascii rendering, block rendering and braille rendering |
| <kbd>9</kbd> / <kbd>0</kbd> | decreases/increases maximum resolution |
| <kbd>7</kbd> / <kbd>8</kbd> | decreases/increases fps by 5 (default: 60) |
| <kbd>1</kbd> ... <kbd>6</kbd> | changes fps to 10 ... 60 respectively |

