pub type Mixer = fn(u32, u32) -> u32;

pub trait Blend {
	fn over(self, other: u32) -> u32;
	fn mix(self, other: u32) -> u32;
	fn add(self, other: u32) -> u32;
	fn sub(self, other: u32) -> u32;
	
	fn premultiply(self) -> u32;
	
	fn copy_alpha(self, other: u32) -> u32;
	
	fn mul_alpha(self, a: u8) -> u32;
	
	fn blend(self, other: u32) -> u32;
	
	fn sub_by_alpha(self, other: u8) -> u32;
	
	fn set_alpha(self, alpha: u8) -> u32;
	
	fn or(self, other: u32) -> u32;
	fn fade(self, alpha: u8) -> u32;
	fn decompose(self) -> [u8; 4];
	fn compose(array: [u8; 4]) -> Self;
	// fn mul(&mut self, other: u32) -> u32;
	// fn mul_alpha(&mut self, other: u32) -> u32;
}

pub fn u8_mul(a: u8, b: u8) -> u8 {
	let [out, _] = (a as u16 * b as u16).to_be_bytes();
	out
}

pub fn u32_fade(this: u32, other: u8) -> u32 {
	let [aa, r, g, b] = this.to_be_bytes();
	let a = u8_mul(aa, other);
	let r = u8_mul(r, other);
	let g = u8_mul(g, other);
	let b = u8_mul(b, other);
	u32::from_be_bytes([a, r, g, b])
}

pub fn alpha_mix(a: u8, b: u8) -> u8 {
	let a = a as i16;
	let b = b as i16;
	
	((a + (b*(256 - a))) >> 8) as u8
}

pub fn channel_mix(x: u8, y: u8, a: u8) -> u8 {
	/*if x < y {
		return x.saturating_add(u8_mul(y - x, a))
	}
	y.saturating_add(u8_mul(x - y, a))*/
	
	let x = x as i16;
	let y = y as i16;
	let a = a as i16;
	
	let [int, _] = ((y - x) * a).to_be_bytes();

	let o = x + int as i16;
	
	o as u8 + (x < y) as u8
	
	/*let x = x as u16;
	let y = y as u16;
	let a = a as u16;
	
	let [o, _] = (x*(256 - a) + y*a).to_be_bytes();
	
	o*/
}

pub fn channel_add(x: u8, y: u8, a: u8) -> u8 {
	x.saturating_add(u8_mul(y, a))
}

impl Blend for u32 {
	fn blend(self, other: u32) -> u32 {
		self.over(other)
	}
	
	fn copy_alpha(self, other: u32) -> u32 {
		(self & 0x00_FF_FF_FF) | (other & 0xFF_00_00_00)
	}
	
	fn premultiply(self) -> u32 {
		let [a, ar, ag, ab] = self.to_be_bytes();
		u32::from_be_bytes([
			a,
			u8_mul(ar, a),
			u8_mul(ag, a),
			u8_mul(ab, a),
		])
	}
	
	fn over(self, other: u32) -> u32 {
		other
	}
	
	fn mul_alpha(self, a: u8) -> u32 {
		let [aa, ar, ag, ab] = self.to_be_bytes();
		u32::from_be_bytes([
			u8_mul(aa, a),
			ar,
			ag,
			ab
			
		])
	}
	
	fn mix(self, other: u32) -> u32 {
		let [_aa, ar, ag, ab] = self.to_be_bytes();
		let [ba, br, bg, bb] = other.to_be_bytes();
		u32::from_be_bytes([
			ba, // alpha_mix(aa, ba),
			channel_mix(ar, br, ba),
			channel_mix(ag, bg, ba),
			channel_mix(ab, bb, ba)
		])
	}

	fn add(self, other: u32) -> u32 {
		let [aa, ar, ag, ab] = self.to_be_bytes();
		let [ba, br, bg, bb] = other.to_be_bytes();
		u32::from_be_bytes([
			aa,
			ar.saturating_add(u8_mul(br, ba)),
			ag.saturating_add(u8_mul(bg, ba)),
			ab.saturating_add(u8_mul(bb, ba))
		])
	}
	
	fn sub(self, other: u32) -> u32 {
		let [aa, ar, ag, ab] = self.to_be_bytes();
		let [ba, br, bg, bb] = other.to_be_bytes();
		u32::from_be_bytes([
			aa,
			ar.saturating_sub(u8_mul(br, ba)),
			ag.saturating_sub(u8_mul(bg, ba)),
			ab.saturating_sub(u8_mul(bb, ba))
		])
	}
	
	fn sub_by_alpha(self, other: u8) -> u32 {
		let [aa, ar, ag, ab] = self.to_be_bytes();
		u32::from_be_bytes([
			aa,
			ar.saturating_sub(other),
			ag.saturating_sub(other),
			ab.saturating_sub(other)
		])
	}

	fn set_alpha(self, alpha: u8) -> u32 {
		self & 0x00_FF_FF_FF | (alpha as u32) << 24
	}
	
	fn or(self, other: u32) -> u32 {
		self | other
	}

	fn fade(self, alpha: u8) -> u32 {
		u32_fade(self, alpha)
	}

	fn decompose(self) -> [u8; 4] {
		self.to_be_bytes()
	}

	fn compose(array: [u8; 4]) -> u32 {
		u32::from_be_bytes(array)
	}
}
