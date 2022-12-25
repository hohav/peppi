use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(
	Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, IntoPrimitive, TryFromPrimitive,
)]
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
