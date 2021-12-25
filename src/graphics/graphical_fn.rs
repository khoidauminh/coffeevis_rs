use crate::constants::{WIN_H, WIN_W, pi2, pih};

pub fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}

pub fn rgba_to_u32(r: u8, g: u8, b: u8, a : u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}

pub fn u32_to_rgb(p : u32) -> (u8, u8, u8) {
    ((p >> 16) as u8, (p >> 8) as u8, p as u8)
}

pub fn u32_to_rgba(p : u32) -> (u8, u8, u8, u8) {
    ((p >> 16) as u8, (p >> 8) as u8, p as u8, (p >> 24) as u8)
}

pub unsafe fn coord_to_1d(x: i32, y: i32) -> usize {
    ((x as usize).min(WIN_W) + (y as usize).min(WIN_H)*WIN_W).min(20735)
}

pub mod color_blending {
    use crate::graphical_fn::{u32_to_rgba, rgba_to_u32};

    pub fn mix(c1 : u32, c2 : u32) -> u32 {
        let c_1 = u32_to_rgba(c1);
        let c_2 = u32_to_rgba(c2);

        let c_1 = (c_1.0 as i32, c_1.1 as i32, c_1.2 as i32, c_1.3 as i32);
        let c_2 = (c_2.0 as i32, c_2.1 as i32, c_2.2 as i32, c_2.3 as i32);

        rgba_to_u32(
            (c_1.0 + (c_2.0-c_1.0)*c_2.3/255) as u8,
            (c_1.1 + (c_2.1-c_1.1)*c_2.3/255) as u8,
            (c_1.2 + (c_2.2-c_1.2)*c_2.3/255) as u8,
            c_1.3 as u8
        )
    }

    pub fn set_alpha(c : u32, a : u8) -> u32 {
        (c & 0b0000_0000_1111_1111_1111_1111_1111_1111) | ((a as u32) << 24)
    }

    pub fn mul_alpha(c : u32, a : u8) -> u32 {
        let a1 = c >> 24;
        (c & 0b0000_0000_1111_1111_1111_1111_1111_1111) | ((a as u32 * a1 as u32) << 24)
    }
}

pub fn win_clear(buf: &mut Vec<u32>) {
    *buf = vec![0u32; buf.len()];
}

pub fn apply_alpha(color : u32, a : u8) -> u32 {
    let (r, g, b) = u32_to_rgb(color);
    rgb_to_u32(
        (r as u16 * a as u16 / 255) as u8,
        (g as u16 * a as u16 / 255) as u8,
        (b as u16 * a as u16 / 255) as u8
    )
}

pub fn win_clear_alpha(buf: &mut Vec<u32>, alpha : f32) {
    for i in 0..buf.len() {
        let (r, g, b) = u32_to_rgb(buf[i]);
        buf[i] = rgb_to_u32(
            (r as f32 * alpha) as u8,
            (g as f32 * alpha) as u8,
            (b as f32 * alpha) as u8
        );
    }
}

pub unsafe fn draw_point(buf : &mut Vec<u32>, p1 : P2, color : u32) {
    buf[coord_to_1d(p1.0, p1.1)] = color_blending::mix(buf[coord_to_1d(p1.0, p1.1)], color);
}

pub unsafe fn draw_line(buf : &mut Vec<u32>, p1 : P2, p2 : P2, color : u32, thickness : f32) {
    let (box0, box1) = get_bounding_box2(p1, p2);

    for i in box0.0..box1.0 {
        for j in box0.1..box1.1 {
            if linear_func(p1, p2, P2(i, j), thickness) {
                buf[coord_to_1d(i, j)] = color;
            }
        }
    }
}

pub unsafe fn draw_line_direct(buf : &mut Vec<u32>, p1 : P2, p2 : P2, color : u32) {
    let (box0, box1) = get_bounding_box2(p1, p2);
    let d = (box1.0 - box0.0 + box1.1 - box0.1) * 2;
    
    for i in 0..d {
        let t = i as f32 / d as f32;
        
        let x = linear_interp(p1.0 as f32, p2.0 as f32, t);
        let y = linear_interp(p1.1 as f32, p2.1 as f32, t);
        
        buf[coord_to_1d(x as i32, y as i32)] = color;
    }
}

pub unsafe fn draw_rect(buf: &mut Vec<u32>, x: usize, y: usize, w: usize, h: usize, color : u32) {
    let x1 = x.min(WIN_W);
    let y1 = y.min(WIN_H);

    let x2 = (x1 + w).min(WIN_W);
    let y2 = (y1 + h).min(WIN_H);
    
    for dx in x1..x2 {
        for dy in y1..y2 {
            let di = dx + dy*WIN_W;

            buf[di] = color;
        }
    }
}

//fn swap<T>(&mut a : T, &mut b : T) {
//	let z = a;
//	a = b;
//	b = z;
//}
//
pub fn copy_buf(buf : Vec<u32>, new_buf : &mut Vec<u32>) {
    let l = buf.len().min(new_buf.len());
    for i in 0..l {
        new_buf[i] = buf[i];
    }
}

#[derive(Clone, Copy)]
pub struct P2(pub i32, pub i32);

pub fn p2_add(p1 : P2, p2 : P2) -> P2 {
    P2(p1.0 + p2.0, p1.1 + p2.1)
}

fn p2_sub(p1 : P2, p2 : P2) -> P2 {
    P2(p1.0 - p2.0, p1.1 - p2.1)
}

fn p2_mul_num(p : P2, n : f32) -> P2 {
    P2((p.0 as f32 * n) as i32, (p.1 as f32 * n) as i32)
}

pub unsafe fn quad_bezier(buf: &mut Vec<u32>, p1 : P2, p2 : P2, p3 : P2, color : u32) {
    let (box0, box1) = get_bounding_box3(p1, p2, p3);
    let d = 2.0 * (box1.0 - box0.0 + box1.1 - box0.1).abs() as f32;

    let mut i = 0.0;
    while i < d {
        let p = quad_interp(p1, p2, p3, i/d);
        buf[coord_to_1d(p.0, p.1)] = color;

        i += 1.0;
    }
}

fn quad_interp_(p1: P2, p2: P2, p3: P2, t : f32) -> P2 {
    let a = p2_mul_num(p1, (1.0-t).powi(2));
    let b = p2_mul_num(p2, 2.0*(1.0-t));
    let c = p2_mul_num(p3, t.powi(2));

    p2_add(a, p2_add(b, c))
}

fn quad_interp(p1: P2, p2: P2, p3: P2, t : f32) -> P2 {
    let a = p2_mul_num(p2_sub(p1, p2), (1.0-t).powi(2));
    let b = p2_mul_num(p2_sub(p3, p2), t*t);

    p2_add(p1, p2_add(a, b))
}

// this will check if a pixel is "close enough" to the quadratic bezier
// not finished
/*
fn quad_function(a : f32, b : f32, c : f32) -> bool {
    let (box0, box1) = get_bounding_box(p1, p2, p3);

    let a = p2_add(p1, p3);
    let b = p2_add(p1, p2);
    let c = p2_add(p2_mul_int(p1, 2.0), p2_sub(p1));

    let dx = (b.0*b.0 - 4*a.0*c.0).sqrt();
    let dy = (b.1*b.1 - 4*a.1*c.1).sqrt();

    if (dx < 0.0 || dy < 0.0) { return false; }

     let tx =
}*/

// Check if a pixel is closer to the linear function of p1 & p2 than the parameter 'thickness' (usually <= 2);
fn linear_func(p1 : P2, p2 : P2, p : P2, thickness : f32) -> bool {
        // vector (a b) perpendicular to the line.
        let a = - p2.1 + p1.1;
        let b = p2.0 - p1.0;
        let c = -p1.0*a -p1.1*b;
        // linear function ax + by + c = 0

        let d = ((a*p.0 + b*p.1 + c) as f32).abs()/((a*a + b*b) as f32).sqrt();

        if (d <= thickness) {
            return true;
        }

        return false;
}

fn get_bounding_box2(p1 : P2, p2 : P2) -> (P2, P2) {
    (
        P2(p1.0.min(p2.0), p1.1.min(p2.1)),
        P2(p1.0.max(p2.0), p1.1.max(p2.1))
    )
}

fn sine_interpolation(a : f32, b : f32, t : f32) -> f32 {
    let lerp = (t*pih).sin();
    (a + (b-a)*lerp)
}

fn get_bounding_box3(p1 : P2, p2 : P2, p3 : P2) -> (P2, P2) {
    (
        P2(p1.0.min(p2.0.min(p3.0)), p1.1.min(p2.1.min(p3.1))),
        P2(p1.0.max(p2.0.max(p3.0)), p1.1.max(p2.1.max(p3.1)))
    )
}

pub fn linear_interp(a : f32, b : f32, t : f32) -> f32 {
    (a + (b-a)*t) 
}
