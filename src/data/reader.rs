use crate::data::Program;

macro_rules! eprintln_red {
    () => {
        eprintln!()
    };
    ($arg:tt) => {
        eprintln!("\x1B[31;1m{}\x1B[0m", $arg)
    };
}

pub(crate) use eprintln_red;

#[allow(unused_macros)]
macro_rules! format_red {
    () => {
        format!()
    };
    ($arg:tt) => {
        format!("\x1B[31;1m{}\x1B[0m", $arg)
    };
}

#[allow(unused_macros)]
macro_rules! panic_red {
    () => {
        format!()
    };
    ($arg:tt) => {
        panic!("\x1B[31;1m{}\x1B[0m", $arg)
    };
}

const RESO_WARNING: &str = "\
    Coffeevis is a CPU program, it is not advised \
    to run it at large a size.\
    ";

impl Program {
    pub fn eval_args(mut self, args: &mut dyn Iterator<Item = &String>) -> Self {
        #[allow(unused_imports)]
        use crate::{data::*, modes::Mode::*};

        let mut size = (DEFAULT_SIZE_WIN, DEFAULT_SIZE_WIN);

        let mut args = args.peekable();
        args.next();

        while let Some(arg) = args.next() {
            let arg = arg.as_str();
            match arg {
                #[cfg(not(feature = "console_only"))]
                "--win" => self.mode = Win,

                #[cfg(not(feature = "window_only"))]
                "--braille" => (self.mode, self.flusher) = (ConBrail, Program::print_brail),

                #[cfg(not(feature = "window_only"))]
                "--ascii" => (self.mode, self.flusher) = (ConAlpha, Program::print_alpha),

                #[cfg(not(feature = "window_only"))]
                "--block" => (self.mode, self.flusher) = (ConBlock, Program::print_block),

                "--no-auto-switch" => self.auto_switch = false,

                "--size" => {
                    let s = args
                        .next()
                        .expect("Argument error: Expected value for size.")
                        .split('x')
                        .map(|x| x.parse::<u16>().expect("Argument error: Invalid value"))
                        .collect::<Vec<_>>();

                    size = (s[0], s[1]);
                }

                "--scale" => {
                    self.scale = args
                        .next()
                        .expect("Argument error: Expected u8 value for scale")
                        .parse::<u8>()
                        .expect("Argument error: Invalid value");

                    if self.scale == 0 {
                        panic!("Argument error: scale is 0");
                    }
                }

                "--fps" => {
                    let rate = args.next().expect(
                        "\
                            Argument error: Expected value for refresh rate.\n\
                            Available values: {float} or inf\
                        ",
                    );

                    let rate = rate.parse::<f32>().expect("Argument error: Invalid value.");

                    if rate < 0.0 {
                        panic!("...What?");
                    }

                    self.change_fps_frac((rate * 1000.0) as u32);

                    self.refresh_rate_mode = RefreshRateMode::Specified;
                }

                "--resizable" => {
                    self.resize = true;
                }

                "--transparent" => {
                    eprintln_red!(
                        "Transparency isn't supported by Softbuffer. \
                        See https://github.com/rust-windowing/softbuffer/issues/215\n"
                    );
                }

                "--max-con-size" => {
                    let s = args
                        .next()
                        .expect("Argument error: Expected value for size")
                        .split('x')
                        .map(|x| x.parse::<u16>().expect("Argument error: Invalid value"))
                        .collect::<Vec<_>>();

                    (self.console_max_width, self.console_max_height) = (s[0], s[1]);
                }

                "--x11" => self.wayland = false,

                "--vis" => {
                    let vis_name = args
                        .next()
                        .expect("Argument error: Expected name of visualizer");

                    self.visualizer = self.vis_navigator.switch_by_name(vis_name).func();
                }

                "--background" => {
                    self.pix.set_background(u32::from_str_radix(
                        args.next().expect(
                            &format_red!("Argument error: Expected hex value for background color. E.g 0022FF")
                        ),
                        16,
                    ).expect(&format_red!("Argument error: Invalid value for background color"))
                    );
                }

                ":3" => {
                    eprintln_red!("\n:3");
                }

                "--crt" => {
                    self.crt = true;
                }

                "--no-display" => {
                    eprintln_red!("You have told coffeevis to not present the buffer. Expect a black window (or no window on Wayland).");
                    self.display = false;
                }

                &_ => match arg {
                    #[cfg(feature = "window_only")]
                    "--braille" | "--block" | "--ascii" => {
                        panic!(
                            "\x1B[31;1m\
                                Feature terminal is turned off in \
                                this build of coffeevis.\n\
                                Did you happen to commpile coffeevis \
                                with features \"window_only\"?\
                            \x1B[0m"
                        )
                    }

                    #[cfg(feature = "console_only")]
                    "--win" => {
                        panic!(
                            "\x1B[31;1m\
                                Feature winit is turned off in \
                                this build of coffeevis.\n\
                                Did you happen to commpile coffeevis \
                                with feature \"console_only\"?\
                            \x1B[0m"
                        )
                    }

                    &_ => {
                        let msg = format!("Argument error: Unknown option {arg}");
                        eprintln_red!(msg);
                    }
                },
            }
        }

        self.update_size(size);

        if std::env::var("WAYLAND_DISPLAY").is_err() {
            self.wayland = false;
        } else if !self.wayland {
            std::env::set_var("WAYLAND_DISPLAY", "");
        }

        self
    }

    pub fn print_startup_info(&self) {
        use crate::modes::Mode::{ConAlpha, ConBlock, ConBrail, Win};

        println!(
            "\nWelcome to Coffeevis!\n\
            Audio visualizer by khoidauminh (Cas Pascal on github)"
        );

        eprintln!("Startup configurations (may change): ");

        println!("Refresh rate: {}hz", self.milli_hz as f32 / 1000.0);

        println!(
            "Auto switch: {}",
            if self.auto_switch { "on" } else { "off" }
        );

        match self.mode {
            Win => println!(
                "Running with: Winit, {}",
                if self.wayland { "Wayland" } else { "X11" }
            ),

            _ => {
                println!(
                    "Running in a terminal: {} rendering",
                    match self.mode {
                        ConBrail => "braille",
                        ConAlpha => "ascii",
                        ConBlock => "block",
                        _ => "",
                    }
                );
            }
        }

        if self.resize {
            eprint!(
                "\n\
                Note: resizing is not thoroughly tested and can crash the program or \
                result in artifacts. "
            );
        }

        let w = self.window_width as u32 * self.scale as u32;
        let h = self.window_height as u32 * self.scale as u32;

        if self.resize || w * h > 70000 {
            eprintln_red!(RESO_WARNING);
        }

        if self.milli_hz / 1000 >= 300 {
            eprintln_red!("\nHave fun cooking your CPU");
        }

        println!();
    }
}
