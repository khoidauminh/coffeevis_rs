use crate::{data::*, graphics::Pixel, modes::Mode::*};

#[cfg(target_os = "linux")]
use desktop::create_tmp_desktop_file;

impl Program {
    pub fn eval_args(mut self, args: &mut dyn Iterator<Item = &String>) -> Self {
        #[allow(unused_imports)]
        let mut size = (DEFAULT_SIZE_WIN, DEFAULT_SIZE_WIN);

        let mut args = args.peekable();
        args.next();

        while let Some(arg) = args.next() {
            let arg = arg.as_str();
            match arg {
                #[cfg(not(target_os = "windows"))]
                "--desktop-file" => {
                    alert!("Attempting to create a desktop file!! This is experimental!!");
                    create_tmp_desktop_file();
                }

                "--quiet" => self.quiet = true,

                "--braille" => {
                    (self.mode, self.console_props.flusher) = (ConBrail, Program::print_brail)
                }

                "--ascii" => {
                    (self.mode, self.console_props.flusher) = (ConAscii, Program::print_ascii)
                }

                "--block" => {
                    (self.mode, self.console_props.flusher) = (ConBlock, Program::print_block)
                }

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
                        .expect("Argument error: Scale must be a positive integer");

                    if self.scale > MAX_SCALE_FACTOR {
                        panic!("Argument error: scale exceeds maximum allowed {MAX_SCALE_FACTOR}.");
                    }

                    assert_ne!(self.scale, 0);
                }

                "--fps" => {
                    let rate = args
                        .next()
                        .expect("Argument error: Expected value for refresh rate.");

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

                "--max-con-size" => {
                    let s = args
                        .next()
                        .expect("Argument error: Expected value for size")
                        .split('x')
                        .map(|x| x.parse::<u16>().expect("Argument error: Invalid value"))
                        .collect::<Vec<_>>();

                    self.console_props.set_max((s[0], s[1]));
                }

                "--x11" => {
                    alert!(
                        "This option no longer works as Rust 2024 Edition now marks \
                        set_var and remove_var as unsafe. Unset WAYLAND_DISPLAY to \
                        force running in Xwayland."
                    );
                }

                "--vis" => {
                    let vis_name = args
                        .next()
                        .expect("Argument error: Expected name of visualizer");

                    self.visualizer = self.vis_navigator.switch_by_name(vis_name).func();
                }

                "--background" => {
                    let bg = u32::from_str_radix(
                        args.next().expect(&format_red!(
                            "Argument error: Expected hex value for background color. E.g 0022FF"
                        )),
                        16,
                    )
                    .expect(&format_red!(
                        "Argument error: Invalid value for background color"
                    ));

                    self.pix.set_background(bg);

                    self.transparent = bg.alpha() != 255;
                }

                ":3" => {
                    error!("\n:3");
                }

                "--effect" => {
                    let val = args
                        .next()
                        .expect("Expecting values of the following: crt, interlaced.");

                    match val.as_str() {
                        "crt" => self.set_win_render_effect(RenderEffect::Crt),
                        "interlaced" => self.set_win_render_effect(RenderEffect::Interlaced),
                        "none" => self.set_win_render_effect(RenderEffect::None),
                        _ => panic!("Invalid value for effect."),
                    }
                }

                "--no-display" => {
                    alert!(
                        "You have told coffeevis to not present the buffer. Expect a black window (or no window on Wayland)."
                    );
                    self.display = false;
                }

                &_ => error!("Argument error: Unknown option {}", arg),
            }
        }

        self.update_size(size);

        if self.quiet || self.mode.is_con() {
            super::log::set_log_enabled(false);
        }

        if !self.mode.is_con() {
            self.mode = Win;
        }

        self
    }

    pub fn print_startup_info(&self) {
        let mut string_out = String::new();

        string_out += "Welcome to Coffeevis!\n\
        Audio visualizer by khoidauminh (Cas Pascal on github)\n";

        string_out += "Startup configurations (may change):\n";

        string_out += &format!("Refresh rate: {}hz\n", self.milli_hz as f32 / 1000.0);

        string_out += &format!(
            "Auto-switch: {}\n",
            if self.auto_switch { "on" } else { "off" }
        );

        if self.mode.is_con() {
            string_out += &format!("Running in terminal, renderer: {}\n", self.mode.get_name());
        } else {
            string_out += "Running graphically\n";
        }

        if self.is_transparent() {
            string_out += "Transparency is on";
        }

        info!("{}", string_out);

        if self.resize {
            alert!(
                "Note: resizing is not thoroughly tested and \
                can crash the program or result in artifacts."
            );
        }

        {
            let w = self.window_props.width as u32 * self.scale as u32;
            let h = self.window_props.height as u32 * self.scale as u32;

            if self.resize || w * h > 70000 {
                alert!(
                    "\
                Coffeevis is a CPU program, it is not advised \
                to run it at large a size.\
                "
                );
            }
        }

        if self.milli_hz / 1000 >= 300 {
            alert!("Have fun cooking your CPU");
        }

        println!();
    }
}
