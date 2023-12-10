macro_rules! err {
	($( $arg: expr ),*) => {
		std::io::Error::new(std::io::ErrorKind::InvalidData, format!($( $arg ),*))
	}
}

/// Every .slp file will start with a UBJSON opening brace, `raw` key & type: "{U\x03raw[$U#l"
pub const SLIPPI_FILE_SIGNATURE: [u8; 11] = [
	0x7b, 0x55, 0x03, 0x72, 0x61, 0x77, 0x5b, 0x24, 0x55, 0x23, 0x6c,
];

pub mod model {
	pub mod frame;
	pub mod game;
	pub mod shift_jis;
	pub mod slippi;
}

pub mod io {
	pub mod slippi {
		pub mod de;
		pub mod ser;
	}
	pub(crate) mod ubjson {
		pub(crate) mod de;
		pub(crate) mod ser;
	}
}

use std::{error, fmt, io::Read};

#[derive(Debug)]
pub struct ParseError {
	pub pos: Option<u64>,
	pub error: std::io::Error,
}

impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if let Some(pos) = self.pos {
			write!(f, "parse error @{:#x}: {}", pos, self.error)
		} else {
			write!(f, "parse error: {}", self.error)
		}
	}
}

impl error::Error for ParseError {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		Some(&self.error)
	}
}

/// Since we support non-seekable readers, we use this wrapper to track
/// position for better error reporting.
struct TrackingReader<R> {
	reader: R,
	pos: u64,
}

impl<R: Read> Read for TrackingReader<R> {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		let result = self.reader.read(buf);
		if let Ok(read) = result {
			self.pos += read as u64;
		}
		result
	}
}

/// Parse a Slippi replay from `r`, returning a `peppi::model::game::immutable::Game`.
pub fn game<R: Read>(
	r: &mut R,
	opts: Option<&io::slippi::de::Opts>,
) -> std::result::Result<model::game::immutable::Game, ParseError> {
	let mut r = TrackingReader { pos: 0, reader: r };
	io::slippi::de::deserialize(&mut r, opts).map_err(|e| ParseError {
		error: e,
		pos: Some(r.pos),
	})
}
