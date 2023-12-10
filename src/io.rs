pub mod slippi {
	pub mod de;
	pub mod ser;
}
pub mod peppi {
	pub mod de;
	pub mod ser;
}
pub(crate) mod ubjson {
	pub(crate) mod de;
	pub(crate) mod ser;
}

use std::io::{Read, Result};

use crate::model::game::{self, immutable::Game};

pub(crate) const MAX_PLAYERS: usize = 6;
pub(crate) const ICE_CLIMBERS: u8 = 14;

pub(crate) fn expect_bytes<R: Read>(r: &mut R, expected: &[u8]) -> Result<()> {
	let mut actual = vec![0; expected.len()];
	r.read_exact(&mut actual)?;
	if expected == actual.as_slice() {
		Ok(())
	} else {
		Err(err!("expected: {:?}, got: {:?}", expected, actual))
	}
}

pub(crate) fn assert_max_version(game: &Game) -> Result<()> {
	if game.start.slippi.version <= game::MAX_SUPPORTED_VERSION {
		Ok(())
	} else {
		Err(err!(
			"Unsupported Slippi version ({} > {})",
			game.start.slippi.version,
			game::MAX_SUPPORTED_VERSION
		))
	}
}
