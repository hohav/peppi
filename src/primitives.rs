use std::fmt::{Debug, Display, Formatter, Result};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, num_enum::TryFromPrimitive)]
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

frame_data!(Position {
	x: f32,
	y: f32,
}, { });

frame_data!(Velocity {
	x: f32,
	y: f32,
}, { });
