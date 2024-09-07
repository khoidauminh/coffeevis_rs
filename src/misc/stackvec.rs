#[derive(Debug)]
pub struct StackVec<T, const N: usize> {
    buffer: [T; N],
    length: usize,
}

impl<T: Copy + Clone, const N: usize> StackVec<T, N> {
    pub fn panic_overflow() {
        panic!("Capacity reached");
    }

    pub fn panic_out_of_bounds() {
        panic!("Out of bound");
    }

    #[doc(hidden)]
    pub const fn init(v: T, len: usize) -> Self {
        if len > N {
            panic!("Init size larger than capacity");
        }

        Self {
            buffer: [v; N],
            length: len,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn cap(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_full(&self) -> bool {
        self.length == N
    }

    pub fn resize(&mut self, newlen: usize) {
        if newlen > self.buffer.len() {
            Self::panic_overflow();
        }
    }

    pub fn slice<'a>(&'a self) -> &'a [T] {
        &self.buffer
    }

    pub fn push(&mut self, v: T) {
        if self.length >= self.buffer.len() {
            Self::panic_overflow();
        }
        self.buffer[self.length] = v;
        self.length += 1;
    }

    pub fn pop(&mut self) -> T {
        if self.length == 0 {
            panic!("Trying to pop from an empty StackVec");
        }
        self.length -= 1;
        self.buffer[self.length]
    }

    pub fn first<'a>(&self) -> Option<&T> {
        if self.length > 0 {
            Some(&self.buffer[0])
        } else {
            None
        }
    }

    pub fn last<'a>(&self) -> Option<&T> {
        if self.length > 0 {
            Some(&self.buffer[self.length - 1])
        } else {
            None
        }
    }

    pub fn fill(&mut self, v: T) {
        self.buffer[0..self.length].fill(v)
    }
}

impl<T: Copy + Clone, const N: usize> std::ops::Index<usize> for StackVec<T, N>
where
    T: Copy + Clone,
{
    type Output = T;
    fn index(&self, i: usize) -> &Self::Output {
        &self.buffer[i]
    }
}

impl<T: Copy + Clone, const N: usize> std::ops::IndexMut<usize> for StackVec<T, N>
where
    T: Copy + Clone,
{
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.buffer[i]
    }
}
