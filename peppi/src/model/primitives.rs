use std::fmt::{self, Debug, Display, Formatter};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use peppi_derive::Arrow;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Port {
	P1 = 0,
	P2 = 1,
	P3 = 2,
	P4 = 3,
}

impl Display for Port {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		use Port::*;
		match *self {
			P1 => write!(f, "P1"),
			P2 => write!(f, "P2"),
			P3 => write!(f, "P3"),
			P4 => write!(f, "P4"),
		}
	}
}

impl Default for Port {
	fn default() -> Self {
		Self::P1
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Direction { Left, Right }

impl Default for Direction {
	fn default() -> Self {
		Self::Left
	}
}

/// Melee represents direction as f32 for some reason
impl TryFrom<f32> for Direction {
	type Error = std::io::ErrorKind;

	fn try_from(x: f32) -> Result<Self, Self::Error> {
		if x < 0.0 {
			Ok(Direction::Left)
		} else if x > 0.0 {
			Ok(Direction::Right)
		} else {
			Err(Self::Error::InvalidData)
		}
	}
}

impl From<Direction> for f32 {
	fn from(d: Direction) -> Self {
		match d {
			Direction::Left => -1.0,
			Direction::Right => 1.0,
		}
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Arrow)]
pub struct Position {
	pub x: f32,
	pub y: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Arrow)]
pub struct Velocity {
	pub x: f32,
	pub y: f32,
}
