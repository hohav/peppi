#![allow(incomplete_features)]
#![feature(const_generics)]

#[macro_export]
macro_rules! err {
	($( $arg: expr ),*) => {
		std::io::Error::new(std::io::ErrorKind::InvalidData, format!($( $arg ),*))
	}
}

#[derive(Clone, Debug)]
pub struct Config {
	pub json: bool,
	pub frames: bool,
	pub enum_names: bool,
	pub query: Option<String>,
}

// TODO: use serde_state to pass this config to the serializers?
pub static mut CONFIG: Config = Config {
	json: false,
	frames: false,
	enum_names: false,
	query: None,
};

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
pub mod stage;
pub mod triggers;
pub mod ubjson;

#[cfg(test)] mod test;

use std::{error, fmt, fs, io, path};

#[derive(Debug)]
pub struct ParseError {
	pub pos: Option<usize>,
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

pub struct TrackingReader<R> {
	reader: R,
	bytes_read: usize,
}

impl<R: io::Read> io::Read for TrackingReader<R> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let result = self.reader.read(buf);
		if let Ok(bytes) = result {
			self.bytes_read += bytes;
		}
		result
	}
}

/// Parses a Slippi replay from `r`, passing events to the callbacks in `handlers` as they occur.
pub fn parse<R: io::Read, H: parse::Handlers>(r: &mut R, handlers: &mut H) -> std::result::Result<(), ParseError> {
	let mut r = TrackingReader {
		bytes_read: 0,
		reader: r,
	};
	parse::parse(&mut r, handlers)
		.map_err(|e| ParseError { error: e, pos: Some(r.bytes_read) })
}

/// Parses the Slippi replay file at `path`, returning a `game::Game` object.
pub fn game(path: &path::Path) -> std::result::Result<game::Game, ParseError> {
	let f = fs::File::open(path).map_err(|e| ParseError { error: e, pos: None })?;
	let mut r = io::BufReader::new(f);
	let mut game_parser: game_parser::GameParser = Default::default();
	parse(&mut r, &mut game_parser)
		.and_then(|_| game_parser.into_game().map_err(|e| ParseError { pos: None, error: e }))
}
