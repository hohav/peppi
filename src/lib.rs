#[macro_export]
macro_rules! err {
	($( $arg: expr ),*) => {
		std::io::Error::new(std::io::ErrorKind::InvalidData, format!($( $arg ),*))
	}
}

#[derive(Clone, Debug)]
pub struct Config {
	pub enum_names: bool,
}

// TODO: use serde_state to pass this config to the serializers?
pub static mut CONFIG: Config = Config {
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

use std::{error, fmt, io};
use std::io::{Read, Seek, SeekFrom};

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

pub struct SkippingReader<R> {
	reader: R,
	pos: u64,
}

impl<R: Read> Read for SkippingReader<R> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let result = self.reader.read(buf);
		if let Ok(read) = result {
			self.pos += read as u64;
		}
		result
	}
}

impl<R: Read> Seek for SkippingReader<R> {
	fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
		match pos {
			SeekFrom::Current(skip) if skip >= 0 => {
				io::copy(&mut self.reader.by_ref().take(skip as u64), &mut io::sink())?;
				self.pos += skip as u64;
				Ok(self.pos)
			},
			_ => unimplemented!(),
		}
	}
}

/// Parses a Slippi replay from `r`, passing events to the callbacks in `handlers` as they occur.
pub fn parse<R: Read, H: parse::Handlers>(r: &mut R, handlers: &mut H, skip_frames: bool) -> std::result::Result<(), ParseError> {
	let mut r = SkippingReader {
		pos: 0,
		reader: r,
	};
	parse::parse(&mut r, handlers, skip_frames)
		.map_err(|e| ParseError { error: e, pos: r.stream_position().ok() })
}

/// Parses a Slippi replay file from `r`, returning a `game::Game` object.
pub fn game<R: Read>(r: &mut R, skip_frames: bool) -> Result<game::Game, ParseError> {
	let mut game_parser: game_parser::GameParser = Default::default();
	parse(r, &mut game_parser, skip_frames)
		.and_then(|_| game_parser.into_game().map_err(|e| ParseError { error: e, pos: None }))
}
