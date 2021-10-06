use std::io::Result;

use serde_json::{Map, Value};

use crate::{
	model::{
		frame::{self, Frame, PortData},
		game::{self, Frames, Game, GeckoCodes, NUM_PORTS},
		item,
		metadata,
		primitives::Port,
	},
	serde::de::{self, FrameEvent, FrameId, Indexed, PortId},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct Opts {
	pub rollbacks: bool,
}

#[derive(Debug, Default)]
pub struct FrameEvents {
	pub pre: [Vec<frame::Pre>; NUM_PORTS],
	pub post: [Vec<frame::Post>; NUM_PORTS],
}

#[derive(Debug, Default)]
pub struct Collector {
	pub opts: Opts,
	pub first_port: Option<Port>,

	pub gecko_codes: Option<GeckoCodes>,
	pub start: Option<game::Start>,
	pub end: Option<game::End>,
	pub frames_index: Vec<i32>,
	pub frames_start: Vec<frame::Start>,
	pub frames_end: Vec<frame::End>,
	pub frames_leaders: FrameEvents,
	pub frames_followers: FrameEvents,
	pub items: Vec<Vec<item::Item>>,
	pub metadata: Option<Map<String, Value>>,
}

macro_rules! into_game {
	($gp: expr, $frames_type: ident => $( $idx: expr ),* $(,)? ) => {{
		let start = $gp.start.ok_or_else(|| err!("missing start event"))?;
		let end = $gp.end.ok_or_else(|| err!("missing end event"))?;
		let ports: Vec<_> = start.players.iter().map(|p| p.port as usize).collect();

		let metadata_raw = $gp.metadata.unwrap_or_default();
		let metadata = metadata::parse(&metadata_raw)?;
		if let Some(ref players) = metadata.players {
			let meta_ports: Vec<_> = players.iter().map(|p| p.port as usize).collect();
			if meta_ports != ports {
				return Err(err!("game-start ports ({:?}) != metadata ports ({:?})", ports, meta_ports));
			}
		}

		let frame_count = $gp.frames_leaders.pre[ports[0]].len();

		for p in &ports {
			match $gp.frames_leaders.pre[*p].len() {
				n if n == frame_count => (),
				n => return Err(err!("mismatched pre-frame counts: {}: {}, {}", p, frame_count, n)),
			}
		}

		for p in &ports {
			match $gp.frames_leaders.post[*p].len() {
				n if n == frame_count => (),
				n => return Err(err!("mismatched post-frame counts: {}, {}", frame_count, n)),
			}
		}

		let mut frames = Vec::with_capacity(frame_count);
		for n in 0 .. frame_count {
			frames.push(Frame {
				index: match $gp.opts.rollbacks {
					true => $gp.frames_index[n],
					_ => n as i32 + game::FIRST_FRAME_INDEX,
				},
				start: $gp.frames_start.get(n).copied(),
				end: $gp.frames_end.get(n).copied(),
				ports: [ $(
					PortData {
						leader: frame::Data {
							pre: $gp.frames_leaders.pre[ports[$idx]][n],
							post: $gp.frames_leaders.post[ports[$idx]][n],
						},
						follower: {
							let pre = &$gp.frames_followers.pre[ports[$idx]];
							let post = &$gp.frames_followers.post[ports[$idx]];
							match (pre.is_empty(), post.is_empty()) {
								(true, true) => None,
								(false, false) => Some(Box::new(frame::Data {
									pre: pre[n],
									post: post[n],
								})),
								_ => return Err(err!("inconsistent follower data (frame: {})", n)),
							}
						},
					},
				)* ],
				items: $gp.items.get(n).cloned(),
			});
		}

		Game {
			gecko_codes: $gp.gecko_codes,
			start: start,
			end: end,
			frames: Frames::$frames_type(frames),
			metadata: metadata,
			metadata_raw: metadata_raw,
		}
	}}
}

impl Collector {
	pub fn into_game(self) -> Result<Game> {
		match self.start {
			None => Err(err!("missing start event")),
			Some(ref start) => match start.players.len() {
				1 => Ok(into_game!(self, P1 => 0)),
				2 => Ok(into_game!(self, P2 => 0, 1)),
				3 => Ok(into_game!(self, P3 => 0, 1, 2)),
				4 => Ok(into_game!(self, P4 => 0, 1, 2, 3)),
				n => Err(err!("unsupported number of ports: {}", n)),
			},
		}
	}
}

fn append_frame_event<Id, Event>(v: &mut Vec<Event>, evt: FrameEvent<Id, Event>, opts: Opts) -> Result<usize> where Id: Indexed, Event: Copy {
	let idx = match opts.rollbacks {
		true => v.len(),
		_ => evt.id.array_index(),
	};

	while v.len() < idx {
		// fill in missing values by duplicating the last value
		// FIXME: determine whether this is appropriate for Frame Start & Frame End
		v.push(*v.last().ok_or_else(|| err!("missing initial frame data: {:?}", evt.id.index()))?);
	}

	if idx < v.len() {
		v[idx] = evt.event; // rollback
	} else {
		v.push(evt.event);
	}

	Ok(idx)
}

/// fills in missing frame data for eliminated players by duplicating their last-seen data
macro_rules! append_missing_frame_data {
	( $arr: expr, $count: expr ) => {
		for f in $arr.iter_mut() {
			if let Some(&last) = f.last() {
				while f.len() < $count {
					f.push(last);
				}
			}
		}
	}
}

impl de::Handlers for Collector {
	fn gecko_codes(&mut self, codes: &[u8], actual_size: u16) -> Result<()> {
		self.gecko_codes = Some(GeckoCodes {
			bytes: codes.to_vec(),
			actual_size: actual_size,
		});
		Ok(())
	}

	fn game_start(&mut self, s: game::Start) -> Result<()> {
		self.start = Some(s);
		Ok(())
	}

	fn game_end(&mut self, s: game::End) -> Result<()> {
		self.end = Some(s);
		Ok(())
	}

	fn frame_start(&mut self, evt: FrameEvent<FrameId, frame::Start>) -> Result<()> {
		let idx = append_frame_event(&mut self.frames_start, evt, self.opts)?;
		// reset items list in case of rollback
		while self.items.len() <= idx {
			self.items.push(Vec::new());
		}
		self.items[idx] = Vec::new();
		Ok(())
	}

	fn frame_pre(&mut self, evt: FrameEvent<PortId, frame::Pre>) -> Result<()> {
		if self.first_port.is_none() {
			self.first_port = Some(evt.id.port);
		}
		if Some(evt.id.port) == self.first_port {
			self.frames_index.push(evt.id.index);
		}
		match evt.id.is_follower {
			true => append_frame_event(&mut self.frames_followers.pre[evt.id.port as usize], evt, self.opts)?,
			_ => append_frame_event(&mut self.frames_leaders.pre[evt.id.port as usize], evt, self.opts)?,
		};
		Ok(())
	}

	fn frame_post(&mut self, evt: FrameEvent<PortId, frame::Post>) -> Result<()> {
		match evt.id.is_follower {
			true => append_frame_event(&mut self.frames_followers.post[evt.id.port as usize], evt, self.opts)?,
			_ => append_frame_event(&mut self.frames_leaders.post[evt.id.port as usize], evt, self.opts)?,
		};
		Ok(())
	}

	fn frame_end(&mut self, evt: FrameEvent<FrameId, frame::End>) -> Result<()> {
		append_frame_event(&mut self.frames_end, evt, self.opts)?;
		Ok(())
	}

	fn item(&mut self, evt: FrameEvent<FrameId, item::Item>) -> Result<()> {
		assert!(!self.items.is_empty());
		let idx = match self.opts.rollbacks {
			true => self.items.len() - 1,
			_ => evt.id.array_index(),
		};
		self.items[idx].push(evt.event);
		Ok(())
	}

	fn metadata(&mut self, metadata: Map<String, Value>) -> Result<()> {
		self.metadata = Some(metadata);
		Ok(())
	}

	fn finalize(&mut self) -> Result<()> {
		let frame_count = self.frames_leaders.pre.iter().map(Vec::len).max().unwrap_or(0);

		append_missing_frame_data!(self.frames_leaders.pre, frame_count);
		append_missing_frame_data!(self.frames_leaders.post, frame_count);
		append_missing_frame_data!(self.frames_followers.pre, frame_count);
		append_missing_frame_data!(self.frames_followers.post, frame_count);

		while self.items.len() < frame_count {
			self.items.push(Vec::new());
		}

		Ok(())
	}
}
