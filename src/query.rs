use std::io::{Write, Result};

pub trait Query {
	fn query(&self, f:&mut dyn Write, config:&super::Config, query:&[&str]) -> Result<()>;
}

macro_rules! query_impl {
	// TODO: eliminate duplication
	($n:ident : $nt:ty, $type:ty, $self:ident, $f:ident, $config:ident, $query:ident $body:block) => {
		impl<const $n:$nt> super::query::Query for $type {
			fn query(&$self, $f:&mut dyn std::io::Write, $config:&super::Config, $query:&[&str]) -> std::io::Result<()> {
				match $query.is_empty() {
					true => match $config.json {
						true => serde_json::to_writer($f, $self).map_err(|e| err!("JSON serialization error: {:?}", e)),
						_ => write!($f, "{:#?}", $self),
					},
					_ => $body,
				}
			}
		}
	};

	($type:ty, $self:ident, $f:ident, $config:ident, $query:ident $body:block) => {
		impl super::query::Query for $type {
			fn query(&$self, $f:&mut dyn std::io::Write, $config:&super::Config, $query:&[&str]) -> std::io::Result<()> {
				match $query.is_empty() {
					true => match $config.json {
						true => serde_json::to_writer($f, $self).map_err(|e| err!("JSON serialization error: {:?}", e)),
						_ => write!($f, "{:#?}", $self),
					},
					_ => $body,
				}
			}
		}
	};

	($type:ty) => {
		query_impl!($type, self, f, config, query {
			Err(err!("can't query an atom ({:?})", query))
		});
	};
}

impl<T> Query for Option<T> where T:Query {
	fn query(&self, f:&mut dyn Write, config:&super::Config, query:&[&str]) -> Result<()> {
		match self {
			Some(x) => x.query(f, config, query),
			_ => match config.json {
				true => write!(f, "null"),
				_ => write!(f, "None"),
			},
		}
	}
}

impl<T> Query for &T where T:Query {
	fn query(&self, f:&mut dyn Write, config:&super::Config, query:&[&str]) -> Result<()> {
		(*self).query(f, config, query)
	}
}

macro_rules! collection_query {
	() => {
		fn query(&self, f:&mut dyn Write, config:&super::Config, query:&[&str]) -> Result<()> {
			match query.is_empty() {
				true => match config.json {
					true => serde_json::to_writer(f, self).map_err(|e| err!("JSON serialization error: {:?}", e)),
					_ => write!(f, "{:#?}", self),
				},
				_ => match &*query[0] {
					"" => {
						for x in self {
							x.query(f, config, &query[1..])?;
							write!(f, "\n")?;
						}
						Ok(())
					},
					s if s.starts_with("-") => match s[1..].parse::<usize>() {
						Ok(idx) if idx < self.len() => self[self.len() - idx].query(f, config, &query[1..]),
						Ok(idx) => Err(err!("index out of bounds: {}", idx)),
						_ => Err(err!("invalid index: {}", s)),
					},
					s => match s.parse::<usize>() {
						Ok(idx) if idx < self.len() => self[idx].query(f, config, &query[1..]),
						Ok(idx) => Err(err!("index out of bounds: {}", idx)),
						_ => Err(err!("invalid index: {}", s)),
					},
				},
			}
		}
	}
}

impl<T> Query for Vec<T> where T:Query, T:std::fmt::Debug, T:serde::Serialize {
	collection_query!();
}

impl<T> Query for [T] where T:Query, T:std::fmt::Debug, T:serde::Serialize {
	collection_query!();
}

impl<T> Query for [T; 1] where T:Query, T:std::fmt::Debug, T:serde::Serialize {
	collection_query!();
}

impl<T> Query for [T; 2] where T:Query, T:std::fmt::Debug, T:serde::Serialize {
	collection_query!();
}

impl<T> Query for [T; 3] where T:Query, T:std::fmt::Debug, T:serde::Serialize {
	collection_query!();
}

impl<T> Query for [T; 4] where T:Query, T:std::fmt::Debug, T:serde::Serialize {
	collection_query!();
}

impl<T> Query for std::collections::HashMap<String, T> where T:Query, T:std::fmt::Debug, T:serde::Serialize {
	fn query(&self, f:&mut dyn std::io::Write, config:&super::Config, query:&[&str]) -> std::io::Result<()> {
		match query.is_empty() {
			true => match config.json {
				true => serde_json::to_writer(f, self).map_err(|e| err!("JSON serialization error: {:?}", e)),
				_ => write!(f, "{:#?}", self),
			},
			_ => self.get(query[0]).query(f, config, &query[1..]),
		}
	}
}

query_impl!(u8);
query_impl!(i8);
query_impl!(u16);
query_impl!(i16);
query_impl!(u32);
query_impl!(i32);
query_impl!(u64);
query_impl!(i64);
query_impl!(f32);
query_impl!(usize);

query_impl!(bool);
query_impl!(String);
