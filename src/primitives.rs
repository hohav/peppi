use std::fmt::{Debug, Formatter, Result};

use serde::Serialize;

pseudo_enum!(Direction: i8 {
	-1 => LEFT,
	1 => RIGHT,
});

#[derive(Copy, Clone, PartialEq, Serialize)]
pub struct Position {
	pub x: f32,
	pub y: f32,
}

impl Debug for Position {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}

#[derive(Copy, Clone, PartialEq, Serialize)]
pub struct Velocity {
	pub x: f32,
	pub y: f32,
}

impl Debug for Velocity {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}
