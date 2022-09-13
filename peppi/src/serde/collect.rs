use std::io::Result;

use serde_json::{Map, Value};

use crate::{
	model::{
		frame::Frame,
		game::{self, Frames, Game, GeckoCodes},
		metadata::Metadata,
	},
	serde::handlers::HandlersAbs,
};

#[derive(Debug, Default)]
pub struct Collector {
	pub gecko_codes: Option<GeckoCodes>,
	pub start: Option<game::Start>,
	pub end: Option<game::End>,
	pub frames: Option<Frames>,
	pub metadata: Option<Map<String, Value>>,
}

impl Collector {
	pub fn into_game(self) -> Result<Game> {
		let start = self.start.ok_or(err!("missing start event"))?;
		let end = self.end.ok_or(err!("missing end event"))?;
		let frames = self.frames.unwrap_or((match start.players.len() {
			1 => Ok(Frames::P1(Vec::new())),
			2 => Ok(Frames::P2(Vec::new())),
			3 => Ok(Frames::P3(Vec::new())),
			4 => Ok(Frames::P4(Vec::new())),
			_ => Err(err!("invalid number of players")),
		})?);
		let metadata_raw = self.metadata.ok_or(err!("missing metadata event"))?;
		let metadata = Metadata::parse(&metadata_raw)?;
		Ok(Game {
			start,
			end,
			frames,
			metadata,
			metadata_raw,
			gecko_codes: self.gecko_codes,
		})
	}
}

impl HandlersAbs for Collector {
	fn gecko_codes(&mut self, codes: GeckoCodes) {
					self.gecko_codes = Some(codes);
	}

	fn game_start(&mut self, s: game::Start) {
		self.start = Some(s);
	}

	fn frame<const N: usize>(&mut self, f: Frame<N>) {
		if matches!(self.frames, None) {
			self.frames = match N {
				1 => Some(Frames::P1(Vec::new())),
				2 => Some(Frames::P2(Vec::new())),
				3 => Some(Frames::P3(Vec::new())),
				4 => Some(Frames::P4(Vec::new())),
				_ => panic!(),
			}
		}
		if let Some(frames) = &mut self.frames {
			frames.downcast_mut::<N>().unwrap().push(f);
		}
	}

	fn game_end(&mut self, e: game::End) {
		self.end = Some(e);
	}

	fn metadata(&mut self, metadata: Map<String, Value>) {
		self.metadata = Some(metadata);
	}
}
