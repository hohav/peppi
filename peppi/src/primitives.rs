use std::fmt::{Debug, Display, Formatter, Result};
use serde::{Deserialize, Serialize};
use peppi_derive::Peppi;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, num_enum::IntoPrimitive, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum Port {
	P1 = 0,
	P2 = 1,
	P3 = 2,
	P4 = 3,
}

impl Display for Port {
	fn fmt(&self, f: &mut Formatter) -> Result {
		use Port::*;
		match *self {
			P1 => write!(f, "P1"),
			P2 => write!(f, "P2"),
			P3 => write!(f, "P3"),
			P4 => write!(f, "P4"),
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum Direction { Left, Right }

impl From<Direction> for u8 {
	fn from(d: Direction) -> Self {
		match d {
			Direction::Left => 0,
			_ => 1,
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Peppi)]
pub struct Position {
	pub x: f32,
	pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Peppi)]
pub struct Velocity {
	pub x: f32,
	pub y: f32,
}
