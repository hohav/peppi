use std::fmt::{Debug, Display, Formatter, Result};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use peppi_derive::Peppi;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, IntoPrimitive, TryFromPrimitive)]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Peppi)]
pub struct Position {
	pub x: f32,
	pub y: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Peppi)]
pub struct Velocity {
	pub x: f32,
	pub y: f32,
}
