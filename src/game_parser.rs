use std::io::Result;
use std::collections::HashMap;

use super::{frame, game, metadata, parse, ubjson};
use super::frame::Indexed;

#[derive(Debug)]
pub struct GameParser {
	pub start: Option<game::Start>,
	pub end: Option<game::End>,
	pub ports: [Option<game::Port>; game::NUM_PORTS],
	pub metadata: Option<HashMap<String, ubjson::Object>>,
}

impl GameParser {
	pub fn into_game(self) -> Result<game::Game> {
		Ok(game::Game {
			start: self.start.ok_or_else(|| err!("missing start event"))?,
			end: self.end.ok_or_else(|| err!("missing end event"))?,
			ports: self.ports,
			metadata: metadata::parse(&self.metadata.unwrap_or_default()),
		})
	}
}

impl parse::Handlers for GameParser {
	fn game_start(&mut self, s:game::Start) -> Result<()> {
		self.start = Some(s);
		Ok(())
	}

	fn game_end(&mut self, s:game::End) -> Result<()> {
		self.end = Some(s);
		Ok(())
	}

	fn frame_pre(&mut self, e:parse::FrameEvent<frame::Pre>) -> Result<()> {
		let id = e.id;

		if self.ports[id.port as usize].is_none() {
			self.ports[id.port as usize] = Some(game::Port {
				leader: game::Frames { pre: Vec::new(), post: Vec::new() },
				follower: None,
			});
		}

		let port = self.ports[id.port as usize].as_mut().unwrap();

		let frames = if id.is_follower {
			if port.follower.is_none() {
				port.follower = Some(game::Frames { pre: Vec::new(), post: Vec::new() });
			}
			&mut port.follower.as_mut().unwrap().pre
		} else {
			&mut port.leader.pre
		};

		let idx = e.event.array_index();
		if idx == frames.len() {
			frames.push(e.event)
		} else if idx > frames.len() {
			Err(err!("missing frames: {:?} -> {:?}", frames.last().map(|f| f.index), e.event.index))?
		} else { // rollback
			frames[idx] = e.event
		};

		Ok(())
	}

	fn frame_post(&mut self, e:parse::FrameEvent<frame::Post>) -> Result<()> {
		let id = e.id;

		if self.ports[id.port as usize].is_none() {
			self.ports[id.port as usize] = Some(
				game::Port {
					leader: game::Frames { pre: Vec::new(), post: Vec::new() },
					follower: None
				}
			);
		}

		let port = self.ports[id.port as usize].as_mut().unwrap();

		let frames = if id.is_follower {
			if port.follower.is_none() {
				port.follower = Some(game::Frames { pre: Vec::new(), post: Vec::new() });
			}
			&mut port.follower.as_mut().unwrap().post
		} else {
			&mut port.leader.post
		};

		let idx = e.event.array_index();
		if idx == frames.len() {
			frames.push(e.event)
		} else if idx > frames.len() {
			Err(err!("out-of-order frame: {:?} -> {:?}", frames.last().map(|f| f.index), e.event.index))?
		} else { // rollback
			frames[idx] = e.event
		};

		Ok(())
	}

	fn metadata(&mut self, metadata:HashMap<String, ubjson::Object>) -> Result<()> {
		self.metadata = Some(metadata);
		Ok(())
	}
}
