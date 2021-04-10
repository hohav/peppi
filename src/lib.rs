#[macro_export]
macro_rules! err {
	($( $arg: expr ),*) => {
		std::io::Error::new(std::io::ErrorKind::InvalidData, format!($( $arg ),*))
	}
}

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

#[macro_use] pub mod frame_data;
#[macro_use] pub mod pseudo_bitmask;
#[macro_use] pub mod pseudo_enum;

pub mod action_state;
pub mod attack;
pub mod buttons;
pub mod character;
pub mod frame;
pub mod game;
pub mod game_parser;
pub mod item;
pub mod metadata;
pub mod parse;
pub mod primitives;
pub mod stage;
pub mod triggers;
pub mod ubjson;

#[cfg(test)] mod test;

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
			write!(f, "error parsing game ({}): {}", pos, self.error)
		} else {
			write!(f, "error parsing game: {}", self.error)
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
pub struct TrackingReader<R> {
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

/// Parses a Slippi replay from `r`, passing events to the callbacks in `handlers` as they occur.
pub fn parse<R: Read, H: parse::Handlers>(r: &mut R, handlers: &mut H, opts: Option<parse::Opts>) -> std::result::Result<(), ParseError> {
	let mut r = TrackingReader {
		pos: 0,
		reader: r,
	};
	parse::parse(&mut r, handlers, opts)
		.map_err(|e| ParseError { error: e, pos: Some(r.pos) })
}

/// Parses a Slippi replay file from `r`, returning a `game::Game` object.
pub fn game<R: Read>(r: &mut R, opts: Option<parse::Opts>) -> Result<game::Game, ParseError> {
	let mut game_parser: game_parser::GameParser = Default::default();
	parse(r, &mut game_parser, opts)
		.and_then(|_| game_parser.into_game().map_err(|e| ParseError { error: e, pos: None }))
}
