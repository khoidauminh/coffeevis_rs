const FFT_MAX_POWER: usize = 13;

fn main() {
    let length = 1 << FFT_MAX_POWER;

    let mut out = String::new();

    out += "use crate::math::Cplx;\n";
    out += "pub const TWIDDLE_MAP: &'static [Cplx] = &[\n";

    out += "Cplx { x: 0.0, y: 0.0 }";

    let mut k = 1;
    while k < length {
        let angle = -std::f32::consts::PI / k as f32;

        for j in 0..k {
            let (y, x) = (j as f32 * angle).sin_cos();
            out += &format!(",\nCplx {{ x: {x:.9}, y: {y:.9} }}");
        }

        k *= 2;
    }

    out += "\n];\n";

    std::fs::write("src/math/twiddle_map.rs", out).unwrap();
}
