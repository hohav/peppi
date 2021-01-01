use std::fmt;

pub type Logical = f32;

#[derive(Copy, Clone, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "hdf5", derive(hdf5::H5Type))]
#[repr(C)]
pub struct Physical {
	pub l: f32,
	pub r: f32,
}

impl fmt::Debug for Physical {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({}, {})", self.l, self.r)
	}
}

impl std::ops::Add for Physical {
	   type Output = Self;

	   fn add(self, other: Self) -> Self {
			   Self {
					   l: self.l + other.l,
					   r: self.r + other.r,
			   }
	   }
}

impl num_traits::identities::Zero for Physical {
	   fn zero() -> Self {
			   Self { l: 0.0, r: 0.0 }
	   }

	   fn is_zero(self: &Self) -> bool {
			   self.l == 0.0 && self.r == 0.0
	   }
}
