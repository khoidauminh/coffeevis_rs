![Logo of Coffeevis](./assets/coffeevis_icon.svg)

# Coffee Music Visualizer
A personal GUI + TUI music visualizer written in Rust.

## About this project
I decided to write the visualizer because existing projects didn't suit me.
Some too slow, some didn't have the visualizer I like, some take up too much cpu usage, etc.

This project also serves as my playground, so you'll see a lot of weird implementations in the source files.

## What's new in v0.6.0

#### YOU CAN NOW WRITE COFFEEVIS VISUALIZERS *OUTSIDE* OF COFFEEVIS

See [README.md](src/data/README.md)

### Other changes

Terminal mode is back and enabled by default!

Minifb support has been removed!

Two new features have been added: 'window_only' and 'console_only` that
respectively disables terminal mode and winit mode if you don't want it.

A little post-processing effect has been added to window mode which should
make the animations a little more smooth looking.

New visualizers: TODO!

Configuration changes: TODO!

## Examples:

![Console](https://media1.giphy.com/media/v1.Y2lkPTc5MGI3NjExMWwydzQydHVhdG9nbTZwZnZyNHEyazduNmp0aGFib21xYjFtc2F3YSZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/flPyMMxHiSpo73kpEg/giphy.gif)
![Terminal](https://media2.giphy.com/media/v1.Y2lkPTc5MGI3NjExeThkNGxieWVxcGhhM3RibXZ5aGNzczN1YzF5aGRuNmVyYzBlYjc4NiZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/5FZtC9GbiOrXrhnMON/giphy.gif)

## Platform support
Coffeevis so far only runs on Linux.
Windows, MacOS and BSD support is not available.

## Installation

```
cargo install coffeevis
```

To disable winit, use:

```
cargo install coffeevis --features console_only
```

To disable console mode, use:

```
cargo install coffeevis --features window_only
```

## Configuration

Coffeevis does not remember settings and does not generate config files
(feature won't be implemented unless requested).

To get around this, make a user script that runs coffeevis with flags

E.g:
```
#!/bin/bash

/path/to/coffeevis --fps 60 --no-auto-switch --size 40x40

```

To force Coffeevis to run in Xwayland, unset `WAYLAND_DISPLAY`

```
WAYLAND_DISPLAY= coffeevis
```

## Flags
Coffeevis supports temporary options at launch

| Option | Value (example) | Description |
| ------ | ------ | ------ |
| --ascii<br />--block<br />--braille | | runs in the terminal |
| --no-auto-switch | | disables automatic visualizer switching |
| --size | 80x80 | sets resolution in window mode |
| --scale | 2 | upscales in window mode |
| --fps | 60 | sets refresh rate (by default coffeevis will try to query your monitor's refresh rate) |
| --resizable | | allows resizing in window mode |
| --max-con-size | 50x50 | sets maximum resolution in terminal mode |
| --vis | spectrum | launches coffeevis with the specified visualizer |

## Experimental

| Option | Value (example) | Description |
| ------ | ------ | ------ |
| --foreign | | instructs coffeevis to send out audio and listens to input commands see [Foreign Communicator](src/data/README.md) |
| --desktop-file | | (Specifically for GNOME) creates a desktop file so that it gets an icon |

## Notes

Upon launch coffeevis will grab your default audio source, use an audio
config tool to direct your desired source to coffeevis (e.g. `pavucontrol`).

On Wayland, coffeevis cannot set itself on top so you will have to rely on an external tool. For example, on KDE Plasma, you can use the window rules feature.

When input is quiet, the visualizer will try to amplify the input so that the visualizers don't become boring.

Coffeevis prints text directly to stdout, rendering may be heavy depending on your terminal.

A terminal with GPU-accelerated support is recommended (i.e Alacritty, Kitty, Wezterm, ...)

A maximum resolution is built into the console mode (default: 50x50). Coffeevis will render in the center of the screen if terminal dimensions are larger than the limit.

It looks the smoothest when you're in a dark room with low monitor brightness. But don't do that lol

## Keyboard shortcuts

### Global
|  Key | Description |
| ------ | ------ |
| <kbd>Space</kbd> | iterates forward through visualizers (wraps around) |
| <kbd>b</kbd> | iterates backward (wraps around) |
| <kbd>q</kbd> | exits |
| <kbd>/</kbd> | resets all settings |
| <kbd>-</kbd> / <kbd>+</kbd> | decreases/increases input volume |
| <kbd>\[</kbd> / <kbd>\]</kbd> | decreases/increases spectrum roughness |
| <kbd>;</kbd> / <kbd>'</kbd> | decreases/increases amount of samples into input (works for some wave-based visualizers only) |
| <kbd>\\</bkd> | toggles auto switching (default: ON, 8 seconds) |
| <kbd>n</kbd> | switches through sets of visualizers (wraps around) |
| <kbd>f</kbd> | (when launched with foreign communicator) switches between foreign visualizer and built-in. |

### Terminal
|  Key | Description |
| ------ | ------ |
| <kbd>.</kbd> | toggles between ascii rendering, block rendering and braille rendering |
| <kbd>9</kbd> / <kbd>0</kbd> | decreases/increases maximum resolution |
| <kbd>7</kbd> / <kbd>8</kbd> | decreases/increases fps by 5 (default: 60) |
| <kbd>1</kbd> .. <kbd>6</kbd> | changes fps to 10 ... 60 respectively |

<sup><sub>Please don't look at my code. No I'm not hiding anything in there it's all garbage code idk how to do gpu programming so it's all cpu code uh uhhh</sub></sup>
