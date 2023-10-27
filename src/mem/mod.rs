#[derive(Debug)]
pub struct StackVec<T, const N: usize> {
	buffer: [T; N],
	capacity: usize,
	length: usize,
	mask: usize,
}

macro_rules! stackvec {
	($depth: expr, $len: expr, $val: expr) => {
		{
			use crate::mem::StackVec;
			const cap: usize = 1 << $depth;
			if $len > cap {
				panic!(
					"Capacity overflow.\nStackvec allocated with depth {} ({} elements) but passed length is {}", 
					$depth,
					1 << $depth,
					$len
				);
			}
			
			StackVec::<_, cap>::init($val, $len)
		}
	}
}

pub(crate) use stackvec;

impl<T: Copy + Clone, const N: usize> StackVec<T, N> {
	pub fn panic_overflow() {
		panic!("Capacity reached");
	}
	
	#[doc(hidden)]
	pub const fn init(v: T, len: usize) -> Self  {
		// if len > N {panic!("Length exceeds maximum capacity")}
		//if len > N {Self::panic_init(depth, len)}
		
		Self {
			buffer: [v; N],
			capacity: N,
			length: len,
			mask: {N-1}
		}
	}
	
	pub fn len(&self) -> usize {
		self.length
	}
	
	pub fn cap(&self) -> usize {
		self.capacity
	}
	
	pub fn resize(&mut self, newlen: usize) {
		if newlen > self.capacity {
			Self::panic_overflow();
		}
	}
	
	pub fn append(&mut self, v: T) {
		if self.length >= self.capacity {
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
	
	pub fn fill(&mut self, v: T) {
		self.buffer[0..self.length].fill(v)
	}
} 

impl<T: Copy + Clone, const N: usize> std::ops::Index<usize> for StackVec<T, N>
where T: Copy + Clone
{
	type Output = T;
	fn index(&self, i: usize) -> &Self::Output {
		unsafe {self.buffer.get_unchecked(i & self.mask)}
	}
}

impl<T: Copy + Clone, const N: usize> std::ops::IndexMut<usize> for StackVec<T, N>
where T: Copy + Clone
{
	fn index_mut(&mut self, i: usize) -> &mut Self::Output {
		unsafe {self.buffer.get_unchecked_mut(i & self.mask)}
	}
}


