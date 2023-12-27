macro_rules! err {
	($( $arg: expr ),*) => {
		crate::io::Error::InvalidData(format!($( $arg ),*))
	}
}

pub(crate) use err;

pub mod peppi;
pub mod slippi;
pub(crate) mod ubjson;

use std::io::{Read, Seek, SeekFrom};

use thiserror::Error as ThisError;
use xxhash_rust::xxh3::Xxh3;

#[derive(ThisError, Debug)]
pub enum Error {
	#[error("invalid data: {0}")]
	InvalidData(String),

	#[error("I/O error: {0}")]
	Io(#[from] std::io::Error),

	#[error("invalid Arrow: {0}")]
	Arrow(#[from] arrow2::error::Error),

	#[error("invalid JSON: {0}")]
	Json(#[from] serde_json::Error),

	#[error("invalid UTF8: {0}")]
	Utf8(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Reader that hashes the bytes it reads.
struct HashingReader<R: Read> {
	reader: R,
	hasher: Option<Box<Xxh3>>,
}

impl<R: Read> HashingReader<R> {
	pub fn new(reader: R, hash: bool) -> Self {
		Self {
			reader,
			hasher: hash.then(|| Box::new(Xxh3::new())),
		}
	}

	pub fn into_digest(self) -> Option<String> {
		self.hasher.as_deref().map(format_hash)
	}
}

impl<R: Read> Read for HashingReader<R> {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		let n = self.reader.read(buf)?;
		self.hasher.as_mut().map(|h| h.update(&buf[..n]));
		Ok(n)
	}
}

impl<R: Read + Seek> Seek for HashingReader<R> {
	fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
		let n = self.reader.seek(pos)?;
		// disable hashing, since we'll no longer get a meaningful result
		self.hasher = None;
		Ok(n)
	}
}

fn parse_u8(s: &str) -> Result<u8> {
	s.parse().map_err(|_| err!("couldn't parse integer: {}", s))
}

fn expect_bytes<R: Read>(r: &mut R, expected: &[u8]) -> Result<()> {
	let mut actual = vec![0; expected.len()];
	r.read_exact(&mut actual)?;
	if expected == actual.as_slice() {
		Ok(())
	} else {
		Err(err!("expected: {:?}, got: {:?}", expected, actual))
	}
}

/// Format an XXH3 hash the way Peppi does (e.g. "xxh3:580fec7a32ec691a").
pub fn format_hash(hasher: &Xxh3) -> String {
	format!("xxh3:{:016x}", &hasher.digest())
}
