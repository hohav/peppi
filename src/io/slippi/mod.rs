pub mod de;
pub mod ser;

use serde::{Deserialize, Serialize};
use std::{fmt, str};

use crate::io::{parse_u8, Error, Result};

pub use de::read;
pub use ser::write;

/// We can read replays with higher versions than this, but that discards information.
/// We don't support writing these replays as a result, though this restriction may be
/// relaxed in the future.
pub const MAX_SUPPORTED_VERSION: Version = Version(3, 16, 0);

/// Every .slp file will start with a UBJSON opening brace, `raw` key & type: "{U\x03raw[$U#l"
pub const FILE_SIGNATURE: [u8; 11] = [
	0x7b, 0x55, 0x03, 0x72, 0x61, 0x77, 0x5b, 0x24, 0x55, 0x23, 0x6c,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Version(pub u8, pub u8, pub u8);

impl Version {
	pub fn gte(&self, major: u8, minor: u8) -> bool {
		self.0 > major || (self.0 == major && self.1 >= minor)
	}

	pub fn lt(&self, major: u8, minor: u8) -> bool {
		!self.gte(major, minor)
	}
}

impl str::FromStr for Version {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self> {
		let mut i = s.split('.');
		match (i.next(), i.next(), i.next(), i.next()) {
			(Some(major), Some(minor), Some(patch), None) => Ok(Version(
				parse_u8(major)?,
				parse_u8(minor)?,
				parse_u8(patch)?,
			)),
			_ => Err(err!("invalid Slippi version: {}", s.to_string())),
		}
	}
}

impl fmt::Display for Version {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}.{}.{}", self.0, self.1, self.2)
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Slippi {
	pub version: Version,
}

pub(crate) fn assert_max_version(version: Version) -> Result<()> {
	if version <= MAX_SUPPORTED_VERSION {
		Ok(())
	} else {
		Err(err!(
			"unsupported version ({} > {})",
			version,
			MAX_SUPPORTED_VERSION
		))
	}
}
