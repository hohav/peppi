use std::io::{Write, Result};

pub type Logical = f32;

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize)]
pub struct Physical {
	pub l: f32,
	pub r: f32,
}

impl super::query::Query for Physical {
	fn query(&self, f: &mut dyn Write, config: &super::Config, _query: &[&str]) -> Result<()> {
		match config.json {
			true => serde_json::to_writer(f, self).map_err(|e| err!("JSON serialization error: {:?}", e)),
			_ => write!(f, "{:?}", self),
		}
	}
}
