pub trait Blend
{
	fn mix(self, other: u32) -> u32;
	fn add(self, other: u32) -> u32;
	fn or(self, other: u32) -> u32;
	fn fade(self, alpha: u8) -> u32;
	fn decompose(self) -> [u8; 4];
	fn compose(array: [u8; 4]) -> Self;
	// fn mul(&mut self, other: u32) -> u32;
	// fn mul_alpha(&mut self, other: u32) -> u32;
}

pub fn u8_mul(a: u8, b: u8) -> u8 { ((a as u16 * b as u16) >> 8) as u8 }

pub fn u32_fade(this: u32, other: u8) -> u32
{
	let [aa, r, g, b] = this.to_be_bytes();
	let r = u8_mul(r, other);
	let g = u8_mul(g, other);
	let b = u8_mul(b, other);
	u32::from_be_bytes([aa, r, g, b])
}

pub fn channel_mix(x: u8, y: u8, a: u8) -> u8
{
	if x > y {return y.saturating_add(u8_mul(x - y, a))}
	x.saturating_add(u8_mul(y - x, a))
}

pub fn channel_add(x: u8, y: u8, a: u8) -> u8
{
	x.saturating_add(u8_mul(y, a))
}

impl Blend for u32
{
	fn mix(self, other: u32) -> u32
	{
		let [aa, ar, ag, ab] = self.to_be_bytes();
		let [ba, br, bg, bb] = other.to_be_bytes();
		u32::from_be_bytes([
			aa,
			channel_mix(ar, br, ba),
			channel_mix(ag, bg, ba),
			channel_mix(ab, bb, ba)
		])
	}

	fn add(self, other: u32) -> u32
	{
		let [aa, ar, ag, ab] = self.to_be_bytes();
		let [ba, br, bg, bb] = other.to_be_bytes();
		u32::from_be_bytes([
			aa,
			ar.saturating_add(u8_mul(br, ba)),
			ag.saturating_add(u8_mul(bg, ba)),
			ab.saturating_add(u8_mul(bb, ba))
		])
	}

	fn or(self, other: u32) -> u32
	{
		self | other
	}

	fn fade(self, alpha: u8) -> u32
	{
		u32_fade(self, alpha)
	}

	fn decompose(self) -> [u8; 4]
	{
		self.to_be_bytes()
	}

	fn compose(array: [u8; 4]) -> u32
	{
		u32::from_be_bytes(array)
	}
}
