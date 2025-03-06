#![allow(unused_imports)]

/// The foreign communicator module.
///
/// This allows for coffeevis to communicate with other
/// programs via tmp files without the complications of
/// Shared memory or IPC.
///
/// When invoked (via the `--foreign` flag), coffeevis opens
/// 3 files: the audio binary file, the commands text file,
/// and the program text file.
///
/// External programs can read from the audio file and
/// send back either commands data, or an uncompressed
/// image data (hex or binary). Information about the
/// canvas can be retrieved via the prorgam test file.
///
/// This allows writing visualizers in other languages.
/// See an example in src/visualizers/milk/impostor.py
///
use std::fmt::Arguments;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, ErrorKind, Seek, Write};
use std::io::{Error, Read};

use std::slice::Split;
use std::time::Duration;

use crate::graphics::blend::Mixer;
use crate::graphics::draw_raw::*;
use crate::graphics::{DrawCommand, DrawCommandBuffer, P2, Pixel};
use crate::math::Cplx;

use super::Program;

const AUDIO_PATH: &str = "/tmp/coffeevis_audio.bin";
const COMMAND_PATH: &str = "/tmp/coffeevis_command.txt";
const PROGRAM_PATH: &str = "/tmp/coffeevis_program.txt";
const DEFAULT_INTERVAL: Duration = Duration::from_millis(1000 / 60);
const NUM_SAMPLES_TO_WRITE: usize = 1024;

fn is_newline(x: &u8) -> bool {
    *x == b'\n'
}

pub fn identify_line<'a, T: Pixel>(
    inp: &mut Split<'a, u8, impl Fn(&u8) -> bool>,
) -> Option<DrawCommand<T>> {
    let head = inp.next()?;

    if head.starts_with(b"COMMAND") {
        return parse_command(head);
    } else if head.starts_with(b"IMAGEHEX") {
        return parse_image(inp, head);
    } else {
        return None;
    }
}

pub fn parse_command<T: Pixel>(inp: &[u8]) -> Option<DrawCommand<T>> {
    let mut iter = inp.split(|&x| x == b' ');

    iter.next()?; // Skips "COMMAND"

    let mut color = [0u8; 4];

    for c in color.iter_mut() {
        let Some(i) = iter.next() else {
            break;
        };
        *c = u8::from_str_radix(std::str::from_utf8(i).ok()?, 16).ok()?;
    }

    let blending = match iter.next()? {
        b"over" => T::over,
        b"add" => <T as Pixel>::add,
        b"sub" => <T as Pixel>::sub,
        b"mix" => T::mix,
        other => {
            eprintln!("Invalid token {:?}", other);
            return None;
        }
    };

    let ident = iter.next()?;

    let mut num_array = [0i32; 4];

    for (i, n) in iter.zip(num_array.iter_mut()) {
        *n = i32::from_str_radix(std::str::from_utf8(i).ok()?, 16).ok()?;
    }

    let (param, func): (DrawParam, DrawFunction<T>) = match ident {
        b"fill" => (DrawParam::Fill {}, fill),

        b"plot" => (
            DrawParam::Plot {
                p: P2::new(num_array[0], num_array[1]),
            },
            set_pixel_xy_by,
        ),

        b"line" => (
            DrawParam::Line {
                ps: P2::new(num_array[0], num_array[1]),
                pe: P2::new(num_array[0], num_array[1]),
            },
            draw_line_by,
        ),

        b"rect" => (
            DrawParam::RectWh {
                ps: P2::new(num_array[0], num_array[1]),
                w: num_array[2].try_into().ok()?,
                h: num_array[3].try_into().ok()?,
            },
            draw_rect_wh_by,
        ),

        other => {
            eprintln!("Invalid token {:?}", other);
            return None;
        }
    };

    Some(DrawCommand::<T> {
        func,
        param,
        color: T::compose(color),
        blending,
    })
}

pub fn parse_image<'a, T: Pixel>(
    inp: &mut Split<'a, u8, impl Fn(&u8) -> bool>,
    header: &[u8],
) -> Option<DrawCommand<T>> {
    let mut tokens = header.split(|&x| x == b' ');

    tokens.next(); // Skips IMAGEHEX

    let pos_x = i32::from_str_radix(std::str::from_utf8(tokens.next()?).ok()?, 16).ok()?;
    let pos_y = i32::from_str_radix(std::str::from_utf8(tokens.next()?).ok()?, 16).ok()?;
    let width = usize::from_str_radix(std::str::from_utf8(tokens.next()?).ok()?, 16).ok()?;
    let mut vec = Vec::with_capacity(width * width);

    while let Some(line) = inp.next() {
        if line.starts_with(b"END") {
            break;
        }

        for pixel in line.chunks_exact(8) {
            let mut color = [0u8; 4];
            for (num, c) in pixel.chunks_exact(2).zip(color.iter_mut()) {
                *c = num[0] * 16 + num[1];
            }
            vec.push(u32::compose(color));
        }
    }

    Some(DrawCommand::<T> {
        func: draw_pix_by,
        color: T::trans(),
        blending: T::mix,
        param: DrawParam::Pix {
            p: P2::new(pos_x, pos_y),
            w: width,
            v: std::sync::Arc::from(vec),
        },
    })
}

/// Sends audio data to a tmp file
///
/// Layout: continuous array of 32bit float data.
/// Left/Right interleaved.
pub struct ForeignAudioCommunicator {
    audio_file: File,
}

/// Receives command code from external programs
///
/// Accepts these follwing formats
/// ```
/// COMMAND AA RR GG BB BLEND DRAW PARAMETERS
///
/// IMAGEHEX XXXX YYYY WWWW
/// PIXELS
/// END
/// ```
/// `RR`, `GG`, `BB`, `XXXX`, `YYYY`, `WWWW` must all be hex values.
///
/// `BLEND` is one of the following: over, add, sub, mix
/// `DRAW` is one of the following: plot, line, rect, fill,
///
/// `IMAGEHEX` instructs coffeevis to interpret the following
/// stream as image data with `WWWW` width until `END`, then
/// draw it onto the screen at the `XXXX` `YYYY` coordinates.
///
/// `PIXELS` is of format `AARRGGBB`.
///
pub struct ForeignCommandsCommunicator {
    command_file: File,
    input_cache: Vec<u8>,
}

/// Signals external program about coffeevis running.
///
/// Layout:
/// ```
/// X.X.X
/// WWWW HHHH
/// RRRRRR
/// ```
///
/// All numbers are in decimal base.
pub struct ForeignProgramCommunicator {
    program_file: File,
}

impl ForeignAudioCommunicator {
    pub fn new() -> Option<Self> {
        let audio_file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(AUDIO_PATH)
            .inspect_err(|_| eprintln!("Can't open audio file for writing."))
            .ok()?;

        Some(Self { audio_file })
    }

    pub fn send_audio<'a>(&mut self, data: &[Cplx], offset: usize) -> Result<(), Error> {
        self.audio_file.rewind()?;

        let mut writer = std::io::BufWriter::new(&self.audio_file);

        for i in 0..NUM_SAMPLES_TO_WRITE {
            let index = (offset + i) % data.len();
            for c in data[index].as_slice() {
                writer.write(&c.to_ne_bytes())?;
            }
        }

        self.audio_file.sync_all()?;

        Ok(())
    }
}

impl ForeignCommandsCommunicator {
    pub fn new() -> Option<Self> {
        let command_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(COMMAND_PATH)
            .inspect_err(|_| eprintln!("Can't open command file for reading."))
            .ok()?;

        Some(Self {
            command_file,
            input_cache: Vec::new(),
        })
    }

    pub fn read_commands<T: Pixel>(&mut self) -> Result<DrawCommandBuffer<T>, Error> {
        self.command_file.rewind()?;

        let mut out_buffer = Vec::<DrawCommand<T>>::new();

        self.input_cache.clear();
        self.command_file.read_to_end(&mut self.input_cache)?;

        if self.input_cache.is_empty() {
            return Err(Error::new(ErrorKind::Other, "Empty file"));
        }

        let mut lines = self.input_cache.split(is_newline);

        while let Some(cmd) = identify_line(&mut lines) {
            out_buffer.push(cmd);
        }

        if out_buffer.is_empty() {
            return Err(Error::new(ErrorKind::Other, "No parsing has been done"));
        }

        self.command_file.set_len(0)?;

        Ok(DrawCommandBuffer::from(out_buffer))
    }
}

impl ForeignProgramCommunicator {
    pub fn new() -> Option<Self> {
        let program_file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(PROGRAM_PATH)
            .inspect_err(|_| eprintln!("Can't open program file for writing."));

        Some(Self {
            program_file: program_file.ok()?,
        })
    }

    pub fn write(&mut self, a: Arguments) -> Result<(), Error> {
        self.program_file.rewind()?;
        self.program_file.set_len(0)?;
        self.program_file.write_fmt(a)?;
        self.program_file.sync_all()?;
        Ok(())
    }
}

impl Drop for ForeignProgramCommunicator {
    fn drop(&mut self) {
        let _ = self.program_file.set_len(0);
        let _ = self.program_file.sync_all();
    }
}
