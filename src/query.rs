use std::io::{Write, Result};

pub trait Query {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()>;
}

impl<T> Query for Option<T> where T:Query {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()> {
		match self {
			Some(x) => x.query(f, query),
			_ => write!(f, "null"),
		}
	}
}

impl<T> Query for Vec<T> where T:Query {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()> {
		match &*query[0] {
			"" => {
				for x in self {
					x.query(f, &query[1..])?;
					write!(f, "\n")?;
				}
				Ok(())
			},
			s if s.starts_with("-") => match s[1..].parse::<usize>() {
				Ok(idx) if idx < self.len() => self[self.len() - idx].query(f, &query[1..]),
				Ok(idx) => Err(err!("index out of bounds: {}", idx)),
				_ => Err(err!("invalid index: {}", s)),
			},
			s => match s.parse::<usize>() {
				Ok(idx) if idx < self.len() => self[idx].query(f, &query[1..]),
				Ok(idx) => Err(err!("index out of bounds: {}", idx)),
				_ => Err(err!("invalid index: {}", s)),
			},
		}
	}
}

impl<T> Query for [T] where T:Query {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()> {
		match &*query[0] {
			"" => {
				for x in self {
					x.query(f, &query[1..])?;
					write!(f, "\n")?;
				}
				Ok(())
			},
			s if s.starts_with("-") => match s[1..].parse::<usize>() {
				Ok(idx) if idx < self.len() => self[self.len() - idx].query(f, &query[1..]),
				Ok(idx) => Err(err!("index out of bounds: {}", idx)),
				_ => Err(err!("invalid index: {}", s)),
			},
			s => match s.parse::<usize>() {
				Ok(idx) if idx < self.len() => self[idx].query(f, &query[1..]),
				Ok(idx) => Err(err!("index out of bounds: {}", idx)),
				_ => Err(err!("invalid index: {}", s)),
			},
		}
	}
}
