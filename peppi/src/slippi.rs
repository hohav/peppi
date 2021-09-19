use std::fmt;
use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Version(pub u8, pub u8, pub u8);

pub const fn version(major: u8, minor: u8) -> Version {
	Version(major, minor, 0)
}

pub struct ParseVersionError(pub String);

impl From<std::num::ParseIntError> for ParseVersionError {
	fn from(err: std::num::ParseIntError) -> ParseVersionError {
		ParseVersionError(format!("{}", err))
	}
}

impl TryFrom<&str> for Version {
	type Error = ParseVersionError;
	fn try_from(s: &str) -> Result<Version, Self::Error> {
		let v: Vec<u8> = s.split('.').map(|s| s.parse::<u8>()).collect::<Result<Vec<u8>, std::num::ParseIntError>>()?;
		match v.len() {
			0 => unreachable!(),
			1 => Ok(Version(v[0], 0, 0)),
			2 => Ok(Version(v[0], v[1], 0)),
			3 => Ok(Version(v[0], v[1], v[3])),
			_ => Err(ParseVersionError("too many components".to_string())),
		}
	}
}

impl fmt::Display for Version {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}.{}.{}", self.0, self.1, self.2)
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Slippi {
	pub version: Version,
}
