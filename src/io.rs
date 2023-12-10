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

