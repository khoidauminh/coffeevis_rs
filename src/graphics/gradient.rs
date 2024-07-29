use crate::graphics::P2;

enum GradientType {
    Raidal(P2, f64),
    Linear(P2, f64),
}

struct Gradient {
    gradient_type: GradientType,
    color_a: u32,
    color_b: u32,
}
