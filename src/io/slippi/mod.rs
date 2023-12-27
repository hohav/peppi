pub mod de;
pub mod ser;

use serde::{Deserialize, Serialize};
use std::{
	fmt,
	io::{self, Error, Read, Seek},
	str,
};

use crate::{
	game::immutable::Game,
	io::{parse_u8, PosError, TrackingReader},
};

pub use ser::write;

/// We can parse files with higher versions than this, but we won't expose all information.
/// When converting a replay with a higher version number to another format like Arrow,
/// the conversion will be lossy.
pub const MAX_SUPPORTED_VERSION: Version = Version(3, 15, 0);

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
	fn from_str(s: &str) -> Result<Self, Self::Err> {
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

pub fn assert_max_version(version: Version) -> io::Result<()> {
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

/// Parses a Slippi replay from `r`.
pub fn read<R: Read + Seek>(r: R, opts: Option<&de::Opts>) -> Result<Game, PosError> {
	let mut r = TrackingReader::new(r);
	de::read(&mut r, opts).map_err(|e| PosError {
		error: e,
		pos: r.pos,
	})
}
