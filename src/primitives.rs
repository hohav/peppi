use std::fmt::{Debug, Formatter, Result};
use std::ops::Add;

use num_traits::identities::Zero;
use serde::Serialize;

pseudo_enum!(Direction: i8 {
	-1 => LEFT,
	1 => RIGHT,
});

#[derive(Copy, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "hdf5", derive(hdf5::H5Type))]
#[repr(C)]
pub struct Position {
	pub x: f32,
	pub y: f32,
}

impl Debug for Position {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}

impl Add for Position {
	   type Output = Self;

	   fn add(self, other: Self) -> Self {
			   Self {
					   x: self.x + other.x,
					   y: self.y + other.y,
			   }
	   }
}

impl Zero for Position {
	   fn zero() -> Self {
			   Self { x: 0.0, y: 0.0 }
	   }

	   fn is_zero(self: &Self) -> bool {
			   self.x == 0.0 && self.y == 0.0
	   }
}

#[derive(Copy, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "hdf5", derive(hdf5::H5Type))]
#[repr(C)]
pub struct Velocity {
	pub x: f32,
	pub y: f32,
}

impl Debug for Velocity {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}

impl Add for Velocity {
	   type Output = Self;

	   fn add(self, other: Self) -> Self {
			   Self {
					   x: self.x + other.x,
					   y: self.y + other.y,
			   }
	   }
}

impl Zero for Velocity {
	   fn zero() -> Self {
			   Self { x: 0.0, y: 0.0 }
	   }

	   fn is_zero(self: &Self) -> bool {
			   self.x == 0.0 && self.y == 0.0
	   }
}
