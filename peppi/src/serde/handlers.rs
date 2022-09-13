use std::io::Result;
use std::collections::VecDeque;
use serde_json::{Map, Value};
use crate::{
	model::{
		frame::{self, Frame, Pre, Post},
		game::{self, GeckoCodes},
		item::Item,
		slippi::Version,
	},
	serde::de::{FrameEvent, FrameId, PortId,},
};

/// The largest possible rollback size in frames. Used to determine when a
/// frame is finalized for replays before v3.7.0
pub const LARGEST_ROLLBACK: u8 = 7;

/// When the major scene number in the game start event is set to this value
/// it indicates an online game is played and will have rollbacks
pub const NETPLAY_MAJOR_SCENE: u8 = 8;

/// Callbacks for events encountered while parsing a replay.
///
/// For frame events, there will be one event per frame per character
/// (Ice Climbers are two characters).
pub trait Handlers {
	// Descriptions below partially copied from the Slippi spec:
	// https://github.com/project-slippi/slippi-wiki/blob/master/SPEC.md

	/// List of enabled Gecko codes. Currently unparsed.
	fn gecko_codes(&mut self, _codes: &[u8], _actual_size: u16) -> Result<()> { Ok(()) }

	/// How the game is set up; also includes the version of the extraction code.
	fn game_start(&mut self, _: game::Start) -> Result<()> { Ok(()) }
	/// The end of the game.
	fn game_end(&mut self, _: game::End) -> Result<()> { Ok(()) }
	/// Miscellaneous data not directly provided by Melee.
	fn metadata(&mut self, _: serde_json::Map<String, serde_json::Value>) -> Result<()> { Ok(()) }

	/// RNG seed and frame number at the start of a frame's processing.
	fn frame_start(&mut self, _: FrameEvent<FrameId, frame::Start>) -> Result<()> { Ok(()) }
	/// Pre-frame update, collected right before controller inputs are used to figure out the character's next action. Used to reconstruct a replay.
	fn frame_pre(&mut self, _: FrameEvent<PortId, Pre>) -> Result<()> { Ok(()) }
	/// Post-frame update, collected at the end of the Collision detection which is the last consideration of the game engine. Useful for making decisions about game states, such as computing stats.
	fn frame_post(&mut self, _: FrameEvent<PortId, Post>) -> Result<()> { Ok(()) }
	/// Indicates an entire frame's worth of data has been transferred/processed.
	fn frame_end(&mut self, _: FrameEvent<FrameId, frame::End>) -> Result<()> { Ok(()) }

	/// One event per frame per item, with a maximum of 15 updates per frame. Can be used for stats, training AIs, or visualization engines to handle items. Items include projectiles like lasers or needles.
	fn item(&mut self, _: FrameEvent<FrameId, Item>) -> Result<()> { Ok(()) }

	/// Called after all parse events have been handled.
	fn finalize(&mut self) -> Result<()> { Ok(()) }
}

pub trait HandlersAbs {
	/// How the game is set up; also includes the version of the extraction code.
	fn game_start(&mut self, _: game::Start) { }
	/// Single frame of the game
	fn frame<const N: usize>(&mut self, _: Frame<N>) { }
	/// The end of the game.
	fn game_end(&mut self, _: game::End) { }
	/// Miscellaneous data not directly provided by Melee.
	fn metadata(&mut self, _: Map<String, Value>) { }
	/// List of enabled Gecko codes. Currently unparsed.
	fn gecko_codes(&mut self, _: GeckoCodes) { }
	/// Called after all parse events have been handled.
	fn finalize(&mut self) { }
}

/// Defines rollback behavior of `Hook`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rollback {
	/// last ocurrence of a frame overwrites the previous ocurrences
	Overwrite,
	/// all ocurrences of every frame preserved in order they appear
	Preserve,
	/// ignore all rollbacks providing only the first ocurrence of each frame
	Ignore,
	/// consider rollbacks to be an error
	Error,
}

#[derive(Clone, Debug, Default)]
struct FrameEvents {
	pre: [Option<frame::Pre>; game::NUM_PORTS],
	post: [Option<frame::Post>; game::NUM_PORTS],
}

#[derive(Debug, Default)]
struct FrameState {
	index: i32,
	start: Option<frame::Start>,
	leader: FrameEvents,
	follow: FrameEvents,
	end: Option<frame::End>,
	items: Option<Vec<Item>>,
}

impl FrameState {
	fn new(index: i32) -> FrameState {
		FrameState {
			index,
			..FrameState::default()
		}
	}
}

pub struct Hook<H> where H: HandlersAbs {
	hook: H,
	rollback: Rollback,
	frames: VecDeque<FrameState>,
	highest_frame: i32,
	ignore_frame: i32,
	ports: Option<Vec<usize>>,
	version: Version,
}

macro_rules! publish_frame {
	($hook: ident, $fs: expr, $const_generic: expr => $( $idx: expr ),* $(,)?) => {{
		let ports = $hook.ports.as_ref().unwrap();
		let frame = Frame {
			index: $fs.index,
			start: $fs.start,
			end: $fs.end,
			items: $fs.items,
			ports: [$( frame::PortData {
				leader: frame::Data {
					pre: $fs.leader.pre[ports[$idx]].ok_or_else(||
						err!("missing pre event: index {}, port {}", $fs.index, ports[$idx]))?,
					post: $fs.leader.post[ports[$idx]].ok_or_else(||
						err!("missing post event: index {}, port {}", $fs.index, ports[$idx]))?,
				},
				follower: {
					let pre = $fs.follow.pre[ports[$idx]];
					let post = $fs.follow.post[ports[$idx]];
					match (pre, post) {
						(Some(pre), Some(post)) => Some(Box::new(frame::Data {
							pre: pre,
							post: post,
						})),
						(None, None) => None,
						_ => return Err(err!("inconsistent follower data (frame: {})", $fs.index)),
					}
				},
			}),*],
		};

		$hook.hook.frame(frame);
	}}
}

impl<H> Hook<H> where H: HandlersAbs {
	pub fn new(hook: H, rollback: Rollback) -> Self {
		let frames = VecDeque::new();
		Self {
			hook,
			rollback,
			frames,
			highest_frame: game::FIRST_FRAME_INDEX - 1,
			ignore_frame: game::FIRST_FRAME_INDEX - 1,
			ports: None,
			version: Version(0, 1, 0),
		}
	}

	pub fn into_inner(self) -> H {
		self.hook
	}

	fn rollback(&mut self, index: i32) -> Result<()> {
		match self.rollback {
			Rollback::Error => return Err(err!("Rollback considered error")),
			Rollback::Ignore => {
				self.ignore_frame = index;
			},
			Rollback::Overwrite => {
				if index < self.highest_frame - LARGEST_ROLLBACK as i32 {
					return Err(err!("rollback size too large"));
				}
				// All later frames are no longer valid
				self.highest_frame = index;
				while matches!(self.frames.back(), Some(fs) if fs.index >= index) {
					self.frames.pop_back();
				}
				self.add_new_frame(index);
			},
			Rollback::Preserve => {
				self.highest_frame = index;
				// publish all frames in order on rollback
				while let Some(frame) = self.frames.pop_front() {
					self.publish_frame(frame)?;
				}
				self.add_new_frame(index);
			}
		}
		Ok(())
	}

	fn get_frame_state(&mut self, index: i32) -> Result<Option<&mut FrameState>> {
		if index > self.highest_frame + 1 {
			// Frame skipped forward which shouldn't happen
			return Err(err!("replay skips frame {:?}", self.highest_frame + 1));
		} else if index == self.highest_frame + 1 {
			// New frame seen
			self.highest_frame += 1;
			self.add_new_frame(index);

			// publish frames that can no longer be rolled back
			while self.frames.len() > LARGEST_ROLLBACK as usize {
				let frame = self.frames.pop_front().unwrap();
				self.publish_frame(frame)?;
			}
		} else if index <= self.ignore_frame {
			return Ok(None);
		} else if index < self.highest_frame {
			return Err(err!("Unexpected rollback without frame start"));
		}

		// Saftey: index should always be <= self.highest_frame
		let idx = usize::try_from(self.highest_frame - index).unwrap();
		let fs = self.frames.get_mut(idx).ok_or(err!("frame not in buffer"))?;
		Ok(Some(fs))
	}

	fn add_new_frame(&mut self, index: i32) {
		let mut new_frame = FrameState::new(index);
		if self.version >= Version(3, 0, 0) {
			new_frame.items = Some(Vec::new());
		}
		if let Some(latest_frame) = self.frames.back() {
			// TODO: rework to copy only when needed
			new_frame.leader = latest_frame.leader.clone();
			new_frame.follow = latest_frame.follow.clone();
		}
		self.frames.push_back(new_frame);
	}

	fn publish_frame(&mut self, frame: FrameState) -> Result<()> {
		match &self.ports {
			None => return Err(err!("missing start event")),
			Some(ports) => match ports.len() {
				1 => publish_frame!(self, frame, 1 => 0),
				2 => publish_frame!(self, frame, 2 => 0, 1),
				3 => publish_frame!(self, frame, 3 => 0, 1, 2),
				4 => publish_frame!(self, frame, 4 => 0, 1, 2, 3),
				n => return Err(err!("unsupported number of ports: {}", n)),
			}
		}
		Ok(())
	}
}

impl<H> Handlers for Hook<H> where H: HandlersAbs {
	fn game_start(&mut self, start: game::Start) -> Result<()> {
		self.ports = Some(start.players.iter().map(|p| p.port as usize).collect());
		self.version = start.slippi.version;

		self.hook.game_start(start);
		Ok(())
	}
	fn game_end(&mut self, end: game::End) -> Result<()> {
		while let Some(frame) = self.frames.pop_front() {
			self.publish_frame(frame)?;
		}
		self.hook.game_end(end);
		Ok(())
	}
	fn metadata(&mut self, metadata: serde_json::Map<String, serde_json::Value>) -> Result<()> {
		self.hook.metadata(metadata);
		Ok(())
	}
	fn frame_start(&mut self, evt: FrameEvent<FrameId, frame::Start>) -> Result<()> {
		if evt.id.index <= self.highest_frame {
			self.rollback(evt.id.index)?;
		}
		if let Some(frame_state) = self.get_frame_state(evt.id.index)? {
			frame_state.start = Some(evt.event);
		}
		Ok(())
	}
	fn frame_pre(&mut self, evt: FrameEvent<PortId, Pre>) -> Result<()> {
		if let Some(frame_state) = self.get_frame_state(evt.id.index)? {
			if evt.id.is_follower {
				frame_state.follow.pre[evt.id.port as usize] = Some(evt.event);
			} else {
				frame_state.leader.pre[evt.id.port as usize] = Some(evt.event);
			}
		}
		Ok(())
	}
	fn frame_post(&mut self, evt: FrameEvent<PortId, Post>) -> Result<()> {
		if let Some(frame_state) = self.get_frame_state(evt.id.index)? {
			if evt.id.is_follower {
				frame_state.follow.post[evt.id.port as usize] = Some(evt.event);
			} else {
				frame_state.leader.post[evt.id.port as usize] = Some(evt.event);
			}
		}
		Ok(())
	}
	fn frame_end(&mut self, evt: FrameEvent<FrameId, frame::End>) -> Result<()> {
		if let Some(frame_state) = self.get_frame_state(evt.id.index)? {
			frame_state.end = Some(evt.event);

			// Uses latest finalized frame field (v3.7.0+) to publish
			// TODO: potentially superfluous, disable if inefficient
			if let Some(lff) = evt.event.latest_finalized_frame {
				while matches!(self.frames.front(), Some(fs) if fs.index <= lff) {
					let frame = self.frames.pop_front().unwrap();
					self.publish_frame(frame)?;
				}
			}
		}
		Ok(())
	}
	fn item(&mut self, evt: FrameEvent<FrameId, Item>) -> Result<()> {
		if let Some(frame_state) = self.get_frame_state(evt.id.index)? {
			if frame_state.items == None {
				frame_state.items = Some(Vec::new());
			}
			frame_state.items.as_mut().unwrap().push(evt.event);
		}
		Ok(())
	}
	fn gecko_codes(&mut self, codes: &[u8], actual_size: u16) -> Result<()> {
		self.hook.gecko_codes(game::GeckoCodes {
			bytes: codes.to_vec(),
			actual_size,
		});
		Ok(())
	}
	fn finalize(&mut self) -> Result<()> {
		// publish remaining frames here in case game_end never triggered
		while let Some(frame) = self.frames.pop_front() {
			self.publish_frame(frame)?;
		}
		self.hook.finalize();
		Ok(())
	}
}
