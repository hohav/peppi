mod parse;
mod game;
mod ubjson;

use std::{error, fmt, fs, io};
use std::io::{Seek};

#[derive(Debug)]
pub struct ParseError {
	pub line:u64,
	pub error:io::Error,
}

impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "error parsing game: line {}: {}", self.line, self.error)
	}
}

impl error::Error for ParseError {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		Some(&self.error)
	}
}

pub fn parse(path:&str) -> Result<game::Game, ParseError> {
	let f = fs::File::open(path).map_err(|e| ParseError {line:0, error:e})?;
	let mut r = io::BufReader::new(f);
	parse::parse(&mut r)
		.map_err(|e| ParseError {line:r.seek(io::SeekFrom::Current(0)).unwrap_or(0), error:e})
}
