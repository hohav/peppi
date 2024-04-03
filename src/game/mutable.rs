//! Mutable (in-progress) game data.
//!
//! You’ll only encounter mutable frame data if you’re parsing live games.

use serde_json;

use crate::{frame::mutable::Frame, game};

pub struct Game {
	pub start: game::Start,
	pub end: Option<game::End>,
	pub frames: Frame,
	pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
	pub gecko_codes: Option<game::GeckoCodes>,
	pub hash: Option<String>,
	pub quirks: Option<game::Quirks>,
}
