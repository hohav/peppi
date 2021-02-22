use std::fmt;

pub type Logical = f32;

#[derive(Copy, Clone, PartialEq, serde::Serialize)]
pub struct Physical {
	pub l: f32,
	pub r: f32,
}

impl fmt::Debug for Physical {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({}, {})", self.l, self.r)
	}
}
