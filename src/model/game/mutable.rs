use serde_json;

use crate::model::{
	frame::mutable::Frame,
	game,
};

pub struct Game {
	pub start: game::Start,
	pub end: Option<game::End>,
	pub frames: Frame,
	pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
	pub gecko_codes: Option<game::GeckoCodes>,
}
