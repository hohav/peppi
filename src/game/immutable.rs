use serde_json::{Map, Value};

use crate::{
	frame::{immutable::Frame, transpose},
	game::{self, End, GeckoCodes, Quirks, Start},
};

#[derive(Debug)]
pub struct Game {
	pub start: Start,
	pub end: Option<End>,
	pub frames: Frame,
	pub metadata: Option<Map<String, Value>>,
	pub gecko_codes: Option<GeckoCodes>,
	pub hash: Option<String>,
	pub quirks: Option<Quirks>,
}

impl game::Game for Game {
	fn start(&self) -> &Start {
		&self.start
	}

	fn end(&self) -> &Option<End> {
		&self.end
	}

	fn metadata(&self) -> &Option<Map<String, Value>> {
		&self.metadata
	}

	fn gecko_codes(&self) -> &Option<GeckoCodes> {
		&self.gecko_codes
	}

	fn len(&self) -> usize {
		self.frames.id.len()
	}

	fn frame(&self, idx: usize) -> transpose::Frame {
		self.frames.transpose_one(idx, self.start.slippi.version)
	}
}
