pub mod de;
pub mod ser;

use serde::{Deserialize, Serialize};
use std::{fmt, str};

use crate::{
	game::Quirks,
	io::{parse_u8, Error, Result},
};

pub use de::read;
pub use ser::write;

/// Current version of the Peppi format
pub const CURRENT_VERSION: Version = Version(2, 0, 0);

/// Minimum supported version of the Peppi format for reading
pub const MIN_VERSION: Version = Version(2, 0, 0);

/// Peppi files are TAR archives, guaranteed to start with `peppi.json`
pub const FILE_SIGNATURE: [u8; 10] = [0x70, 0x65, 0x70, 0x70, 0x69, 0x2e, 0x6a, 0x73, 0x6f, 0x6e];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Version(pub u8, pub u8, pub u8);

impl fmt::Display for Version {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}.{}.{}", self.0, self.1, self.2)
	}
}

impl str::FromStr for Version {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self> {
		let mut i = s.split('.');
		match (i.next(), i.next(), i.next(), i.next()) {
			(Some(major), Some(minor), Some(revision), None) => Ok(Version(
				parse_u8(major)?,
				parse_u8(minor)?,
				parse_u8(revision)?,
			)),
			_ => Err(err!("invalid Peppi version: {}", s.to_string())),
		}
	}
}

impl Default for Version {
	fn default() -> Self {
		CURRENT_VERSION
	}
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Compression {
	LZ4,
	ZSTD,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Peppi {
	pub version: Version,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub slp_hash: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub quirks: Option<Quirks>,
}

pub(crate) fn assert_current_version(version: Version) -> Result<()> {
	if version < MIN_VERSION {
		Err(err!("unsupported version ({} < {})", version, MIN_VERSION))
	} else {
		Ok(())
	}
}
