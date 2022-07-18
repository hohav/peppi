#![allow(clippy::zero_prefixed_literal)]
#![allow(clippy::redundant_field_names)]

macro_rules! err {
	($( $arg: expr ),*) => {
		std::io::Error::new(std::io::ErrorKind::InvalidData, format!($( $arg ),*))
	}
}

/// Every .slp file will start with a UBJSON opening brace, `raw` key & type: "{U\x03raw[$U#l"
pub const SLIPPI_FILE_SIGNATURE: [u8; 11] =
	[0x7b, 0x55, 0x03, 0x72, 0x61, 0x77, 0x5b, 0x24, 0x55, 0x23, 0x6c];

#[derive(Clone, Copy, Debug)]
pub struct SerializationConfig {
	/// Print enum names with numeric values (e.g. `14:WAIT`).
	pub enum_names: bool,
}

// TODO: replace with `serde_state`?
/// Global singleton, a hack to smuggle config into serializers.
/// You probably don't care about this unless you're serializing with Serde.
pub static mut SERIALIZATION_CONFIG: SerializationConfig = SerializationConfig {
	enum_names: false,
};

pub(crate) mod ubjson {
	pub(crate) mod de;
	pub(crate) mod ser;
}

pub mod model {
	#[macro_use] #[doc(hidden)] pub(crate) mod pseudo_bitmask;
	#[macro_use] #[doc(hidden)] pub(crate) mod pseudo_enum;

	pub mod buttons;
	pub mod frame;
	pub mod game;
	pub mod item;
	pub mod metadata;
	pub mod primitives;
	pub mod slippi;
	pub mod triggers;
	pub mod enums {
		pub mod action_state;
		pub mod attack;
		pub mod character;
		pub mod costume;
		pub mod ground;
		pub mod item;
		pub mod stage;
	}
}

pub mod serde {
	pub mod arrow;
	pub mod collect;
	pub mod de;
	pub mod ser;
}

use std::{
	error,
	fmt,
	io::{self, Read},
};

#[derive(Debug)]
pub struct ParseError {
	pub pos: Option<u64>,
	pub error: io::Error,
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
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let result = self.reader.read(buf);
		if let Ok(read) = result {
			self.pos += read as u64;
		}
		result
	}
}

/// Parse a Slippi replay from `r`, passing events to the callbacks in `handlers` as they occur.
pub fn parse<R: Read, H: serde::de::Handlers>(r: &mut R, handlers: &mut H, opts: Option<&serde::de::Opts>) -> std::result::Result<(), ParseError> {
	let mut r = TrackingReader {
		pos: 0,
		reader: r,
	};
	serde::de::deserialize(&mut r, handlers, opts)
		.map_err(|e| ParseError { error: e, pos: Some(r.pos) })
}

/// Parse a Slippi replay from `r`, returning a `game::Game` object.
pub fn game<R: Read>(r: &mut R, parse_opts: Option<&serde::de::Opts>, collect_opts: Option<&serde::collect::Opts>) -> Result<model::game::Game, ParseError> {
	let mut game_parser = serde::collect::Collector {
		opts: collect_opts.copied().unwrap_or_default(),
		..Default::default()
	};
	parse(r, &mut game_parser, parse_opts)
		.and_then(|_| game_parser.into_game().map_err(|e| ParseError { error: e, pos: None }))
}
