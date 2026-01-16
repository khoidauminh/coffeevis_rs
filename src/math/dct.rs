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

use std::ops::{Add, Sub};

const MAX_DEPTH: usize = 13;
const MAX_SIZE: usize = 1 << 13;

pub struct Dct<T> {
    twiddles: Vec<f32>,
    temp: Vec<T>,
}

impl<T: Add<Output = T> + Sub<Output = T> + std::ops::Mul<f32, Output = T> + Copy + Default>
    Dct<T>
{
    pub fn new(n: usize) -> Self {
        assert_eq!(n.count_ones(), 1, "Length must be power of 2");
        assert!(n <= MAX_SIZE, "Length is greater than {MAX_SIZE}");

        Self {
            twiddles: {
                let mut o = vec![0.0f32; n];

                let mut k = 2;
                while k <= n {
                    let factor = std::f32::consts::PI / (k as f32);

                    let kh = k / 2;

                    for i in 0..kh {
                        o[kh + i] = 1.0 / (((i as f32 + 0.5) * factor).cos() * 2.0);
                    }

                    k *= 2;
                }

                o
            },

            temp: vec![T::default(); n],
        }
    }

    pub fn exec(&mut self, vector: &mut [T]) {
        assert_eq!(vector.len(), self.temp.len(), "Length mismatch");
        Self::__dct(vector, &mut self.temp, &self.twiddles);
    }

    fn __dct(vector: &mut [T], temp: &mut [T], twiddles: &[f32]) {
        let len = vector.len();

        let halflen: usize = len / 2;

        let factors = &twiddles[halflen..];

        for i in 0..halflen {
            let x = vector[i];
            let y = vector[len - 1 - i];
            temp[i] = x + y;
            temp[i + halflen] = (x - y) * factors[i];
        }

        if len > 2 {
            Self::__dct(&mut temp[..halflen], vector, &twiddles);
            Self::__dct(&mut temp[halflen..], vector, &twiddles);
        }

        for i in 0..halflen - 1 {
            vector[i * 2 + 0] = temp[i];
            vector[i * 2 + 1] = temp[i + halflen] + temp[i + halflen + 1];
        }

        vector[len - 2] = temp[halflen - 1];
        vector[len - 1] = temp[len - 1];
    }
}
