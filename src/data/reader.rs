use crate::{data::*, modes::Mode::*};

impl Program {
    pub fn eval_args(mut self, args: &mut dyn Iterator<Item = &String>) -> Self {
        #[allow(unused_imports)]
        let mut size = (DEFAULT_SIZE_WIN, DEFAULT_SIZE_WIN);

        let mut args = args.peekable();
        args.next();

        while let Some(arg) = args.next() {
            let arg = arg.as_str();
            match arg {
                "--foreign" => {
                    crate::audio::get_buf().init_audio_communicator();
                    self.pix.init_commands_communicator();
                    self.init_program_communicator();
                }

                "--quiet" => self.quiet = true,

                #[cfg(not(feature = "window_only"))]
                "--braille" => (self.mode, self.flusher) = (ConBrail, Program::print_brail),

                #[cfg(not(feature = "window_only"))]
                "--ascii" => (self.mode, self.flusher) = (ConAscii, Program::print_ascii),

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

                    if self.scale > MAX_SCALE_FACTOR {
                        panic!("Argument error: scale exceeds maximum allowed {MAX_SCALE_FACTOR}.");
                    }

                    assert_ne!(self.scale, 0);
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

                #[cfg(not(feature = "window_only"))]
                "--max-con-size" => {
                    let s = args
                        .next()
                        .expect("Argument error: Expected value for size")
                        .split('x')
                        .map(|x| x.parse::<u16>().expect("Argument error: Invalid value"))
                        .collect::<Vec<_>>();

                    (self.console_max_width, self.console_max_height) = (s[0], s[1]);
                }

                "--x11" => {
                    self.print_message(format_red!(
                        "This option no longer works as Rust 2024 Edition now marks \
                        set_var and remove_var as unsafe. Unset WAYLAND_DISPLAY to \
                        force running in Xwayland."
                    ));
                }

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
                    eprintln_red!(
                        "You have told coffeevis to not present the buffer. Expect a black window (or no window on Wayland)."
                    );
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

        if !self.mode.is_con() {
            self.mode = match std::env::var("WAYLAND_DISPLAY") {
                Ok(s) if s.len() != 0 => WinWayland,
                _ => WinX11,
            }
        }

        self
    }

    pub fn print_startup_info(&self) {
        self.print_message(
            "\nWelcome to Coffeevis!\n\
            Audio visualizer by khoidauminh (Cas Pascal on github)\n",
        );

        self.print_message("Startup configurations (may change):\n");

        self.print_message(format!(
            "Refresh rate: {}hz\n",
            self.milli_hz as f32 / 1000.0
        ));

        self.print_message(format!(
            "Auto switch: {}\n",
            if self.auto_switch { "on" } else { "off" }
        ));

        if self.mode.is_con() {
            self.print_message(format!(
                "Running in a terminal: {} rendering\n",
                self.mode.get_name()
            ));
        } else {
            self.print_message(format!(
                "Running with: Winit, {}\n",
                if self.mode == WinWayland {
                    "Wayland"
                } else {
                    "X11"
                }
            ))
        }

        if self.resize {
            self.print_message(format_red!(
                "Note: resizing is not thoroughly tested and can crash the program or \
                result in artifacts.\n"
            ));
        }

        #[cfg(not(feature = "console_only"))]
        {
            let w = self.window_width as u32 * self.scale as u32;
            let h = self.window_height as u32 * self.scale as u32;

            if self.resize || w * h > 70000 {
                self.print_message(format_red!(
                    "\
                Coffeevis is a CPU program, it is not advised \
                to run it at large a size.\
                "
                ));
            }
        }

        if self.milli_hz / 1000 >= 300 {
            self.print_message(format_red!("\nHave fun cooking your CPU"));
        }

        println!();
    }
}
