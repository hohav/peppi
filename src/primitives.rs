use std::fmt::{Debug, Formatter, Result};

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum Direction { Left, Right }

#[derive(Clone, Copy, PartialEq, Serialize)]
pub struct Position {
	pub x: f32,
	pub y: f32,
}

impl Debug for Position {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}

#[derive(Clone, Copy, PartialEq, Serialize)]
pub struct Velocity {
	pub x: f32,
	pub y: f32,
}

impl Debug for Velocity {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}
