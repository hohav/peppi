pub mod peppi;
pub mod slippi;
pub(crate) mod ubjson;

use std::{
	error::Error,
	fmt,
	io::{Read, Result},
};

use xxhash_rust::xxh3::Xxh3;

#[derive(Debug)]
pub struct PosError {
	pub error: std::io::Error,
	pub pos: u64,
}

impl fmt::Display for PosError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "parse error @{:#x}: {}", self.pos, self.error)
	}
}

impl Error for PosError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(&self.error)
	}
}

fn parse_u8(s: &str) -> std::io::Result<u8> {
	s.parse().map_err(|_| err!("couldn't parse integer: {}", s))
}

/// Reader that counts the number of bytes read (for error reporting).
pub struct TrackingReader<R> {
	reader: R,
	pos: u64,
}

impl<R> TrackingReader<R> {
	pub fn new(reader: R) -> Self {
		Self { reader, pos: 0 }
	}

	pub fn pos(&self) -> u64 {
		self.pos
	}

	pub fn into_inner(self) -> R {
		self.reader
	}
}

impl<R: Read> Read for TrackingReader<R> {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let result = self.reader.read(buf);
		if let Ok(read) = result {
			self.pos += read as u64;
		}
		result
	}
}

/// Reader that hashes the bytes it reads. Wrap with a `BufReader` for performance.
pub struct HashingReader<R> {
	reader: R,
	hasher: Box<Xxh3>,
}

impl<R> HashingReader<R> {
	pub fn new(reader: R) -> Self {
		Self {
			reader,
			hasher: Box::new(Xxh3::new()),
		}
	}

	pub fn into_digest(self) -> String {
		format_hash(&self.hasher)
	}

	pub fn into_inner(self) -> R {
		self.reader
	}
}

impl<R: Read> Read for HashingReader<R> {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
		let n = self.reader.read(buf)?;
		self.hasher.update(&buf[..n]);
		Ok(n)
	}
}

pub fn format_hash(hasher: &Xxh3) -> String {
	format!("xxh3:{:016x}", &hasher.digest())
}

pub fn expect_bytes<R: Read>(r: &mut R, expected: &[u8]) -> Result<()> {
	let mut actual = vec![0; expected.len()];
	r.read_exact(&mut actual)?;
	if expected == actual.as_slice() {
		Ok(())
	} else {
		Err(err!("expected: {:?}, got: {:?}", expected, actual))
	}
}
