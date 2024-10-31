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
        use crate::{data::*, modes::Mode::*};

        let mut size = (DEFAULT_SIZE_WIN, DEFAULT_SIZE_WIN);

        let mut args = args.peekable();
        args.next();

        while let Some(arg) = args.next() {
            let arg = arg.as_str();
            match arg {
                "--win" => self.mode = Win,

                #[cfg(feature = "minifb")]
                "--minifb" => self.mode = WinLegacy,

                #[cfg(feature = "terminal")]
                "--braille" => (self.mode, self.flusher) = (ConBrail, Program::print_brail),

                #[cfg(feature = "terminal")]
                "--ascii" => (self.mode, self.flusher) = (ConAlpha, Program::print_alpha),

                #[cfg(feature = "terminal")]
                "--block" => (self.mode, self.flusher) = (ConBlock, Program::print_block),

                "--no-auto-switch" => self.AUTO_SWITCH = false,

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
                    self.SCALE = args
                        .next()
                        .expect("Argument error: Expected u8 value for scale")
                        .parse::<u8>()
                        .expect("Argument error: Invalid value");

                    if self.SCALE == 0 {
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

                    let rate = match rate.as_str() {
                        "inf" => f32::INFINITY,

                        &_ => {
                            let rate = rate.parse::<f32>().expect("Argument error: Invalid value.");

                            if rate < 0.0 {
                                panic!("...What?");
                            }

                            rate
                        }
                    };

                    if rate == f32::INFINITY {
                        self.MILLI_HZ = u32::MAX;
                        self.REFRESH_RATE = Duration::from_micros(1);
                    } else {
                        let rate = (rate * 1000.0) as u32;

                        self.change_fps_frac(rate);
                    }

                    self.REFRESH_RATE_MODE = RefreshRateMode::Specified;
                }

                "--resizable" => {
                    self.RESIZE = true;
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

                    (self.CON_MAX_W, self.CON_MAX_H) = (s[0], s[1]);
                }

                "--x11" => self.WAYLAND = false,

                "--vis" => {
                    let vis_name = args
                        .next()
                        .expect("Argument error: Expected name of visualizer");

                    self.visualizer = self.VIS.switch_by_name(vis_name).func();
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

                &_ => match arg {
                    #[cfg(not(feature = "terminal"))]
                    "--braille" | "--block" | "--ascii" => {
                        panic!(
                            "\x1B[31;1m\
                                Feature terminal is turned off in \
                                this build of coffeevis.\n\
                                This was to make dependencies of coffeevis \
                                optional and excluded when not needed.\n\
                                Recompile coffeevis with `--features terminal`\
                                to use these flags.\
                            \x1B[0m"
                        )
                    }

                    #[cfg(not(feature = "minifb"))]
                    "--minifb" => {
                        panic!(
                            "\x1B[31;1m\
                                Feature minifb is turned off in \
                                this build of coffeevis.\n\
                                Recompile coffeevis with `--features minifb` \
                                to use this flag.\
                            \x1B[0m"
                        )
                    }

                    &_ => eprintln_red!("Argument error: Unknown option {arg}"),
                },
            }
        }

        self.update_size(size);

        if std::env::var("WAYLAND_DISPLAY").is_err() {
            self.WAYLAND = false;
        } else if !self.WAYLAND {
            std::env::set_var("WAYLAND_DISPLAY", "");
        }

        self
    }

    pub fn print_startup_info(&self) {
        use crate::modes::Mode::{ConAlpha, ConBlock, ConBrail, Win, WinLegacy};

        println!(
            "\nWelcome to Coffeevis!\n\
            Audio visualizer by khoidauminh (Cas Pascal on github)"
        );

        eprintln!("Startup configurations (may change): ");

        println!("Refresh rate: {}hz", self.MILLI_HZ as f32 / 1000.0);

        println!(
            "Auto switch: {}",
            if self.AUTO_SWITCH { "on" } else { "off" }
        );

        match self.mode {
            Win => println!(
                "Running with: Winit, {}",
                if self.WAYLAND { "Wayland" } else { "X11" }
            ),

            WinLegacy => println!("Running with minifb"),

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

        if self.RESIZE {
            eprint!(
                "\n\
                Note: resizing is not thoroughly tested and can crash the program or \
                result in artifacts. "
            );
        }

        let w = self.WIN_W as u32 * self.SCALE as u32;
        let h = self.WIN_H as u32 * self.SCALE as u32;

        if self.RESIZE || w * h > 70000 {
            eprintln_red!(RESO_WARNING);
        }

        if self.MILLI_HZ / 1000 >= 300 {
            eprintln_red!("\nHave fun cooking your CPU");
        }

        println!();
    }
}
