![Logo of Coffeevis](./assets/coffeevis_icon.svg)

# Coffee Music Visualizer
A personal GUI + TUI music visualizer written in Rust.

## What's new in v0.6.0

Terminal mode is back and enabled by default!

Minifb support has been removed!

Controls have been remapped. 
I'm currently preparing to introduce 
more complex visualizers that allow inputs.

A little post-processing effect has been added 
to window mode which should make the animations 
a little more smooth looking.

## Examples:

![Console](https://media1.giphy.com/media/v1.Y2lkPTc5MGI3NjExMWwydzQydHVhdG9nbTZwZnZyNHEyazduNmp0aGFib21xYjFtc2F3YSZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/flPyMMxHiSpo73kpEg/giphy.gif)
![Terminal](https://media2.giphy.com/media/v1.Y2lkPTc5MGI3NjExeThkNGxieWVxcGhhM3RibXZ5aGNzczN1YzF5aGRuNmVyYzBlYjc4NiZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/5FZtC9GbiOrXrhnMON/giphy.gif)

## Platform support

Coffeevis works on Linux.
Windows support is being planned.

## Installation

```
cargo install coffeevis
```

## Running

As of Cpal 0.17, only the ALSA host is supported, 
it is advised to install pipewire-alsa or pulseaudio-alsa in your system.

Upon launch coffeevis will grab your default audio source, use an audio
config tool to direct your desired source to coffeevis (e.g. `pavucontrol`),
usually to your default monitor sink, or by turning your output sink into 
a duplex and routing coffeevis to that.

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

| Option | Value(s) | Description |
| ------ | ------ | ------ |
| --ascii<br />--block<br />--braille | | run in the terminal |
| --no-auto-switch | | disable automatic visualizer switching |
| --size | 80x80 | set resolution in window mode |
| --scale | 2 | upscale in window mode |
| --fps | 60 | set refresh rate (by default coffeevis will try to query your monitor's refresh rate) |
| --resize | | allow resizing in window mode |
| --max-con-size | 50x50 | set maximum resolution in terminal mode |
| --vis | spectrum | launche coffeevis with the specified visualizer |
| --effect | crt | blank out every other horizontal line to simulate CRT effect |
| --effect | interlaced | (default) interlace fields together to make the visualizer appear smoother (the number of fields is the scale value) |
| --effect | none | rendering is scaled and presented as is |

## Notes

On Wayland, coffeevis cannot set itself on top so you will have to rely on an external tool. For example, on KDE Plasma, you can use the window rules feature.

When input is quiet, the visualizer will try to amplify the input so that the visualizers don't become boring.

Coffeevis prints text directly to stdout, rendering may be heavy depending on your terminal.

A terminal with GPU-accelerated support is recommended (i.e Alacritty, Kitty, Wezterm, ...)

A maximum resolution is built into the console mode (default: 50x50). Coffeevis will render in the center of the screen if terminal dimensions are larger than the limit.

It looks the smoothest when you're in a dark room with low monitor brightness. But don't do that lol

## Experimental features

"fast" is an aggressive feature that:
- disables most color blendings.
- sets the default rendering effect to none.
- disables some checks in drawing code.
- changes some util code.

May break visualizers.

## Keyboard shortcuts

### Global
|  Key | Description |
| ------ | ------ |
| <kbd>n</kbd> | iterate forward through visualizers (wraps around) |
| <kbd>b</kbd> | iterate backward (wraps around) |
| <kbd>q</kbd> | exit |
| <kbd>\\</bkd> | toggle auto switching (default: ON, 8 seconds) |

### Terminal
|  Key | Description |
| ------ | ------ |
| <kbd>.</kbd> | toggle between ascii rendering, block rendering and braille rendering |
| <kbd>9</kbd> / <kbd>0</kbd> | decrease/increase maximum resolution |
| <kbd>7</kbd> / <kbd>8</kbd> | decrease/increase fps by 5 (default: 60) |
| <kbd>1</kbd> .. <kbd>6</kbd> | change fps to 10 ... 60 respectively |

## Writing a visualizer (DRAFT)

See visualizers/misc/example.rs for an example.

<sup><sub>Please don't look at my code. No I'm not hiding anything in there it's all garbage code idk how to do gpu programming so it's all cpu code uh uhhh</sub></sup>
