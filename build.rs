const FFT_MAX_POWER: usize = 13;

use qoi::{Decoder, Header};

fn generate_twiddles() -> String {
    let length = 1 << FFT_MAX_POWER;

    let mut out = String::new();

    out += "use crate::math::Cplx;\n";
    out += "pub static TWIDDLE_MAP: &'static [Cplx] = &[\n";

    out += "Cplx(0.0, 0.0)";

    let mut k = 1;
    while k < length {
        let angle = -std::f32::consts::PI / k as f32;

        for j in 0..k {
            let (y, x) = (j as f32 * angle).sin_cos();
            out += &format!(",\nCplx({x:.9}, {y:.9})");
        }

        k *= 2;
    }

    out += "\n];\n";

    out
}

fn create_icon_const() -> String {
    let icon_file = include_bytes!("assets/coffeevis_icon_128x128.qoi");

    let mut icon = Decoder::new(icon_file)
        .map(|i| i.with_channels(qoi::Channels::Rgba))
        .ok()
        .unwrap();

    let &Header { width, height, .. } = icon.header();

    let buffer = icon.decode_to_vec().unwrap();

    let mut out = String::new();

    out += &format!("pub const ICON_WIDTH: u32 = {width};\n");
    out += &format!("pub const ICON_HEIGHT: u32 = {height};\n");

    out += &format!("pub static ICON_BUFFER: &'static [u8] = &[");

    for (i, c) in buffer.iter().enumerate() {
        if i != 0 {
            out += ", ";
        }
        out += &format!("0x{:x}", c);
    }

    out += "];\n";

    out
}

fn main() {
    let out = generate_twiddles() + &create_icon_const();

    std::fs::write("src/data/gen_const.rs", out).unwrap();
}
