use std::io::Result;
use std::collections::HashMap;

use super::{frame, game, metadata, parse, ubjson};
use super::frame::{Frame, Port};
use super::game::{Frames, Game, NUM_PORTS};

#[derive(Debug)]
pub struct GameParser {
	pub start: Option<game::Start>,
	pub end: Option<game::End>,
	pub frames_start: Vec<frame::Start>,
	pub frames_end: Vec<frame::End>,
	pub frames_pre: [Vec<frame::Pre>; NUM_PORTS],
	pub frames_post: [Vec<frame::Post>; NUM_PORTS],
	pub metadata: Option<HashMap<String, ubjson::Object>>,
}

pub fn port(pre: frame::Pre, post: frame::Post) -> Port {
	Port {
		leader: frame::Data {
			pre: pre,
			post: post,
		},
		follower: None,
	}
}

// TODO: eliminate duplication
pub fn frames1(start: Vec<frame::Start>, end: Vec<frame::End>, pre: Vec<&Vec<frame::Pre>>, post: Vec<&Vec<frame::Post>>) -> Result<Frames> {
	let mut frames = Vec::new();
	for n in 0 .. pre[0].len() {
		frames.push(Frame {
			start: start.get(n).copied(),
			end: end.get(n).copied(),
			ports: [
				port(pre[0][n], post[0][n])
			]
		});
	}
	Ok(Frames::P1(frames))
}

pub fn frames2(start: Vec<frame::Start>, end: Vec<frame::End>, pre: Vec<&Vec<frame::Pre>>, post: Vec<&Vec<frame::Post>>) -> Result<Frames> {
	let mut frames = Vec::new();
	for n in 0 .. pre[0].len() {
		frames.push(Frame {
			start: start.get(n).copied(),
			end: end.get(n).copied(),
			ports: [
				port(pre[0][n], post[0][n]),
				port(pre[1][n], post[1][n]),
			]
		});
	}
	Ok(Frames::P2(frames))
}

pub fn frames3(start: Vec<frame::Start>, end: Vec<frame::End>, pre: Vec<&Vec<frame::Pre>>, post: Vec<&Vec<frame::Post>>) -> Result<Frames> {
	let mut frames = Vec::new();
	for n in 0 .. pre[0].len() {
		frames.push(Frame {
			start: start.get(n).copied(),
			end: end.get(n).copied(),
			ports: [
				port(pre[0][n], post[0][n]),
				port(pre[1][n], post[1][n]),
				port(pre[2][n], post[2][n]),
			]
		});
	}
	Ok(Frames::P3(frames))
}

pub fn frames4(start: Vec<frame::Start>, end: Vec<frame::End>, pre: Vec<&Vec<frame::Pre>>, post: Vec<&Vec<frame::Post>>) -> Result<Frames> {
	let mut frames = Vec::new();
	for n in 0 .. pre[0].len() {
		frames.push(Frame {
			start: start.get(n).copied(),
			end: end.get(n).copied(),
			ports: [
				port(pre[0][n], post[0][n]),
				port(pre[1][n], post[1][n]),
				port(pre[2][n], post[2][n]),
				port(pre[3][n], post[3][n]),
			]
		});
	}
	Ok(Frames::P4(frames))
}

impl GameParser {
	pub fn into_game(self) -> Result<Game> {
		let start = self.start.ok_or_else(|| err!("missing start event"))?;
		let end = self.end.ok_or_else(|| err!("missing end event"))?;
		let num_ports = start.players.len();

		let metadata = metadata::parse(&self.metadata.unwrap_or_default())?;
		if let Some(players) = metadata.players.as_ref() {
			if num_ports != players.len() {
				Err(err!("unexpected number of metadata players: {}", players.len()))?;
			}
		}

		let frames_pre: Vec<_> = self.frames_pre.iter().filter(|x| !x.is_empty()).collect();
		if num_ports != frames_pre.len() {
			Err(err!("unexpected number of pre-frame ports: {}", frames_pre.len()))?;
		}

		let frames_post: Vec<_> = self.frames_post.iter().filter(|x| !x.is_empty()).collect();
		if num_ports != frames_post.len() {
			Err(err!("unexpected number of post-frame ports: {}", frames_post.len()))?;
		}

		match num_ports {
			1 => Ok(Game {
				start: start,
				end: end,
				frames: frames1(self.frames_start, self.frames_end, frames_pre, frames_post)?,
				metadata: metadata,
			}),
			2 => Ok(Game {
				start: start,
				end: end,
				frames: frames2(self.frames_start, self.frames_end, frames_pre, frames_post)?,
				metadata: metadata,
			}),
			3 => Ok(Game {
				start: start,
				end: end,
				frames: frames3(self.frames_start, self.frames_end, frames_pre, frames_post)?,
				metadata: metadata,
			}),
			4 => Ok(Game {
				start: start,
				end: end,
				frames: frames4(self.frames_start, self.frames_end, frames_pre, frames_post)?,
				metadata: metadata,
			}),
			n => Err(err!("unsupported number of ports: {}", n))?,
		}
	}
}

fn append_frame_event<Id, Event>(v: &mut Vec<Event>, evt: parse::FrameEvent<Id, Event>) -> Result<()> where Id: parse::Indexed, Event: Copy {
	let idx = evt.id.array_index();
	if idx == v.len() {
		v.push(evt.event);
	} else if idx < v.len() { // rollback
		v[idx] = evt.event;
	} else {
		// is this appropriate for Frame Start & Frame End?
		if let Some(&last) = v.last() {
			while v.len() < idx {
				v.push(last);
			}
		} else {
			Err(err!("missing initial frame data: {:?}", evt.id.index()))?;
		}
	}
	Ok(())
}

impl parse::Handlers for GameParser {
	fn game_start(&mut self, s: game::Start) -> Result<()> {
		self.start = Some(s);
		Ok(())
	}

	fn game_end(&mut self, s: game::End) -> Result<()> {
		self.end = Some(s);
		Ok(())
	}

	fn frame_start(&mut self, evt: parse::FrameEvent<parse::FrameId, frame::Start>) -> Result<()> {
		append_frame_event(&mut self.frames_start, evt)?;
		Ok(())
	}

	fn frame_pre(&mut self, evt: parse::FrameEvent<parse::PortId, frame::Pre>) -> Result<()> {
		match evt.id.port as usize {
			p if p < NUM_PORTS => Ok(append_frame_event(&mut self.frames_pre[p], evt)?),
			p => Err(err!("invalid port: {}", p)),
		}
	}

	fn frame_post(&mut self, evt: parse::FrameEvent<parse::PortId, frame::Post>) -> Result<()> {
		match evt.id.port as usize {
			p if p < NUM_PORTS => Ok(append_frame_event(&mut self.frames_post[p], evt)?),
			p => Err(err!("invalid port: {}", p)),
		}
	}

	fn frame_end(&mut self, evt: parse::FrameEvent<parse::FrameId, frame::End>) -> Result<()> {
		append_frame_event(&mut self.frames_end, evt)?;
		Ok(())
	}

	fn metadata(&mut self, metadata:HashMap<String, ubjson::Object>) -> Result<()> {
		self.metadata = Some(metadata);
		Ok(())
	}

	fn finalize(&mut self) -> Result<()> {
		let frame_count = self.frames_pre.iter().map(|f| f.len()).max().unwrap_or(0);

		// fill in missing frames for eliminated players by duplicating the last frame
		for f in self.frames_pre.iter_mut() {
			if let Some(&last) = f.last() {
				while f.len() < frame_count {
					f.push(last);
				}
			}
		}

		for f in self.frames_post.iter_mut() {
			if let Some(&last) = f.last() {
				while f.len() < frame_count {
					f.push(last);
				}
			}
		}

		Ok(())
	}
}
