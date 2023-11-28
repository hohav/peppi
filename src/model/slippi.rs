use std::fmt;

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Version(pub u8, pub u8, pub u8);

impl Version {
	pub fn gte(&self, major: u8, minor: u8) -> bool {
		self.0 > major || (self.0 == major && self.1 >= minor)
	}

	pub fn lt(&self, major: u8, minor: u8) -> bool {
		!self.gte(major, minor)
	}
}

pub struct ParseVersionError(pub String);

impl From<std::num::ParseIntError> for ParseVersionError {
	fn from(err: std::num::ParseIntError) -> Self {
		ParseVersionError(format!("{}", err))
	}
}

impl TryFrom<&str> for Version {
	type Error = ParseVersionError;
	fn try_from(s: &str) -> Result<Self, Self::Error> {
		let v: Vec<u8> = s
			.split('.')
			.map(|s| s.parse::<u8>())
			.collect::<Result<Vec<u8>, std::num::ParseIntError>>()?;
		match v.len() {
			0 => unreachable!(),
			1 => Ok(Version(v[0], 0, 0)),
			2 => Ok(Version(v[0], v[1], 0)),
			3 => Ok(Version(v[0], v[1], v[2])),
			_ => Err(ParseVersionError("too many components".to_string())),
		}
	}
}

impl fmt::Display for Version {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}.{}.{}", self.0, self.1, self.2)
	}
}

/// Information about the Slippi mod.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Slippi {
	pub version: Version,
}
