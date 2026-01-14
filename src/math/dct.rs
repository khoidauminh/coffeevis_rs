/*
 * Fast discrete cosine transform algorithms (Rust)
 *
 * Copyright (c) 2024 Project Nayuki. (MIT License)
 * https://www.nayuki.io/page/fast-discrete-cosine-transform-algorithms
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 * - The above copyright notice and this permission notice shall be included in
 *   all copies or substantial portions of the Software.
 * - The Software is provided "as is", without warranty of any kind, express or
 *   implied, including but not limited to the warranties of merchantability,
 *   fitness for a particular purpose and noninfringement. In no event shall the
 *   authors or copyright holders be liable for any claim, damages or other
 *   liability, whether in an action of contract, tort or otherwise, arising from,
 *   out of or in connection with the Software or the use or other dealings in the
 *   Software.
 */

use std::ops::{Add, Div, Sub};

pub fn dct<
    T: Add<Output = T> + Sub<Output = T> + std::ops::Div<f32, Output = T> + Copy + Default,
>(
    vector: &mut [T],
) {
    let n: usize = vector.len();
    assert_eq!(n.count_ones(), 1, "Length must be power of 2");

    __dct(vector, &mut vec![T::default(); n]);
}

fn __dct<T: Add<Output = T> + Sub<Output = T> + Div<f32, Output = T> + Copy>(
    vector: &mut [T],
    temp: &mut [T],
) {
    // Algorithm by Byeong Gi Lee, 1984. For details, see:
    // See: http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.118.3056&rep=rep1&type=pdf#page=34
    // Link may be dead. Visit the source article of this file for another copy.
    let len = vector.len();
    let lenf = len as f32;

    let halflen: usize = len / 2;

    let factor = std::f32::consts::PI / (lenf as f32);

    for i in 0..halflen {
        let x = vector[i];
        let y = vector[len - 1 - i];

        temp[i] = x + y;

        let twiddle = ((i as f32 + 0.5) * factor).cos() * 2.0;

        temp[i + halflen] = (x - y) / twiddle;
    }

    if len > 2 {
        __dct(&mut temp[..halflen], vector);
        __dct(&mut temp[halflen..len], vector);
    }

    for i in 0..halflen - 1 {
        vector[i * 2 + 0] = temp[i];
        vector[i * 2 + 1] = temp[i + halflen] + temp[i + halflen + 1];
    }

    vector[len - 2] = temp[halflen - 1];
    vector[len - 1] = temp[len - 1];
}
