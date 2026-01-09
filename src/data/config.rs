use crate::{data::*, modes::Mode::*};

#[cfg(target_os = "linux")]
use desktop::create_tmp_desktop_file;

impl Program {
    pub fn eval_args(mut self, args: &mut dyn Iterator<Item = &String>) -> Self {
        let mut mode = Win;

        let mut flusher: fn(&Program, stdout: &mut std::io::Stdout) =
            |x, y| Program::print_ascii(x, y);

        let mut size = (DEFAULT_SIZE_WIN, DEFAULT_SIZE_WIN);
        let mut scale = 2;
        let mut milli_hz: Option<u32> = None;
        let mut vis = String::new();
        let mut effect = RenderEffect::Interlaced;
        let mut resize = false;
        let mut max_con_size = (50, 50);

        let mut args = args.peekable();
        args.next();

        while let Some(arg) = args.next() {
            let arg = arg.as_str();
            match arg {
                #[cfg(target_os = "linux")]
                "--desktop-file" => {
                    alert!("Attempting to create a desktop file!! This is experimental!!");
                    create_tmp_desktop_file();
                }

                "--quiet" => self.quiet = true,

                "--ascii" => mode = ConAscii,

                "--braille" => (mode, flusher) = (ConBrail, Program::print_brail),

                "--block" => (mode, flusher) = (ConBlock, Program::print_block),

                "--no-auto-switch" => self.vislist.auto_switch = false,

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
                    scale = args
                        .next()
                        .expect("Argument error: Expected u8 value for scale")
                        .parse::<u8>()
                        .expect("Argument error: Scale must be a positive integer");

                    if scale > MAX_SCALE_FACTOR {
                        panic!("Argument error: scale exceeds maximum allowed {MAX_SCALE_FACTOR}.");
                    }

                    if scale == 0 {
                        panic!("Argument error: scale needs to be larger than 0.");
                    }
                }

                "--fps" => {
                    let rate = args
                        .next()
                        .expect("Argument error: Expected value for refresh rate.");

                    let rate = rate.parse::<f32>().expect("Argument error: Invalid value.");

                    if rate < 0.0 {
                        panic!("...What?");
                    }

                    milli_hz = Some((rate * 1000.0) as u32);
                }

                "--resize" => {
                    resize = true;
                }

                "--max-con-size" => {
                    let s = args
                        .next()
                        .expect("Argument error: Expected value for size")
                        .split('x')
                        .map(|x| x.parse::<u16>().expect("Argument error: Invalid value"))
                        .collect::<Vec<_>>();

                    max_con_size = (s[0], s[1]);
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

                    vis = vis_name.clone();
                }

                ":3" => {
                    error!("\n:3");
                }

                "--effect" => {
                    let val = args
                        .next()
                        .expect("Expecting values of the following: crt, interlaced.");

                    effect = match val.as_str() {
                        "crt" => RenderEffect::Crt,
                        "interlaced" => RenderEffect::Interlaced,
                        "none" => RenderEffect::None,
                        _ => panic!("Invalid value for effect."),
                    };
                }

                &_ => error!("Argument error: Unknown option {}", arg),
            }
        }

        match std::env::var("WAYLAND_DISPLAY") {
            Ok(x) if x.is_empty() => self.wayland = false,
            Err(_) => self.wayland = false,
            _ => {}
        }

        self.update_size(size);

        if self.quiet || self.mode.is_con() {
            super::log::set_log_enabled(false);
        }

        self.mode = mode;
        self.console_props.flusher = flusher;
        self.console_props.set_max(max_con_size);
        self.update_size(size);
        self.scale = scale;
        self.resize = resize;
        self.vislist.select_by_name(&vis);
        self.win_render_effect = effect;

        if let Some(m) = milli_hz {
            if self.mode == Win && self.wayland() {
                alert!("Setting FPS on wayland is no longer supported.");
            }

            self.change_fps_frac(m);
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
                to run it at a large size.\
                "
                );
            }
        }
    }
}
