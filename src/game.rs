use std::fmt;
use std::io::{Write, Result};

use serde::{Serialize};

use super::{character, frame, metadata, stage};
use super::query::{Query};

#[derive(Debug, PartialEq, Serialize)]
pub struct Slippi {
	pub version: (u8, u8, u8),
}

pub const NUM_PORTS:usize = 4;
pub const FIRST_FRAME_INDEX:i32 = -123;

pseudo_enum!(PlayerType:u8 {
	0 => HUMAN,
	1 => CPU,
	2 => DEMO,
});

pseudo_enum!(TeamColor:u8 {
	0 => RED,
	1 => BLUE,
	2 => GREEN,
});

pseudo_enum!(TeamShade:u8 {
	0 => NORMAL,
	1 => LIGHT,
	2 => DARK,
});

#[derive(Debug, PartialEq, Serialize)]
pub struct Team {
	pub color: TeamColor,
	pub shade: TeamShade,
}

pseudo_enum!(DashBack:u32 {
	1 => UCF,
	2 => ARDUINO,
});

pseudo_enum!(ShieldDrop:u32 {
	1 => UCF,
	2 => ARDUINO,
});

#[derive(Debug, PartialEq, Serialize)]
pub struct Ucf {
	pub dash_back: Option<DashBack>,
	pub shield_drop: Option<ShieldDrop>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct PlayerV1_3 {
	pub name_tag: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct PlayerV1_0 {
	pub ucf: Ucf,

	#[cfg(v1_3)]
	#[serde(flatten)]
	pub v1_3: PlayerV1_3,

	#[cfg(not(v1_3))]
	#[serde(flatten)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v1_3: Option<PlayerV1_3>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Player {
	pub character: character::External,
	pub r#type: PlayerType,
	pub stocks: u8,
	pub costume: u8,
	pub team: Option<Team>,
	pub handicap: u8,
	pub bitfield: u8,
	pub cpu_level: Option<u8>,
	pub offense_ratio: f32,
	pub defense_ratio: f32,
	pub model_scale: f32,

	#[cfg(v1_0)] #[serde(flatten)]
	pub v1_0: PlayerV1_0,

	#[cfg(not(v1_0))] #[serde(flatten)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v1_0: Option<PlayerV1_0>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct StartV2_0 {
	pub is_frozen_ps: bool,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct StartV1_5 {
	pub is_pal: bool,

	#[cfg(v2_0)] #[serde(flatten)]
	pub v2_0: StartV2_0,

	#[cfg(not(v2_0))] #[serde(flatten)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v2_0: Option<StartV2_0>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Start {
	pub slippi: Slippi,
	pub is_teams: bool,
	pub item_spawn_frequency: i8,
	pub self_destruct_score: i8,
	pub stage: stage::Stage,
	pub game_timer: u32,
	pub item_spawn_bitfield: [u8; 5],
	pub damage_ratio: f32,
	pub players: [Option<Player>; NUM_PORTS],
	pub random_seed: u32,

	#[cfg(v1_5)] #[serde(flatten)]
	pub v1_5: StartV1_5,

	#[cfg(not(v1_5))] #[serde(flatten)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v1_5: Option<StartV1_5>,
}

pseudo_enum!(EndMethod:u8 {
	0 => UNRESOLVED,
	1 => TIME,
	2 => GAME,
	3 => RESOLVED,
	7 => NO_CONTEST,
});

#[derive(Debug, PartialEq, Serialize)]
pub struct EndV2_0 {
	pub lras_initiator: i8,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct End {
	pub method: EndMethod,

	#[cfg(v2_0)] #[serde(flatten)]
	pub v2_0: EndV2_0,

	#[cfg(not(v2_0))] #[serde(flatten)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v2_0: Option<EndV2_0>,
}

fn skip_frames<T>(_:&T) -> bool {
	!unsafe { super::CONFIG.frames }
}

#[derive(PartialEq, Serialize)]
pub struct Frames {
	#[serde(skip_serializing_if = "skip_frames")]
	pub pre: Vec<frame::Pre>,
	#[serde(skip_serializing_if = "skip_frames")]
	pub post: Vec<frame::Post>,
}

impl Query for Frames {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()> {
		match query.is_empty() {
			true => write!(f, "{:?}", self),
			_ => match &*query[0] {
				"pre" => self.pre.query(f, &query[1..]),
				"post" => self.post.query(f, &query[1..]),
				s => Err(err!("unknown field `frames.{}`", s)),
			},
		}
	}
}

impl fmt::Debug for Frames {
	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
		match unsafe { super::CONFIG.frames } {
			true => f.debug_struct("Frames")
				.field("pre", &self.pre)
				.field("post", &self.post)
				.finish(),
			_ => f.debug_struct("Frames")
				.field("pre", &self.pre.len())
				.field("post", &self.post.len())
				.finish(),
		}
	}
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Port {
	pub leader: Frames,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub follower: Option<Frames>,
}

impl Query for Port {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()> {
		match query.is_empty() {
			true => write!(f, "{:?}", self),
			_ => match &*query[0] {
				"leader" => self.leader.query(f, &query[1..]),
				"follower" => self.follower.query(f, &query[1..]),
				s => Err(err!("unknown field `port.{}`", s)),
			},
		}
	}
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Game {
	pub start: Start,
	pub end: End,
	pub ports: [Option<Port>; NUM_PORTS],
	pub metadata: metadata::Metadata,
}

impl Query for Game {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()> {
		match query.is_empty() {
			true => write!(f, "{:?}", self),
			_ => match &*query[0] {
				"ports" => self.ports.query(f, &query[1..]),
				s => Err(err!("unknown field `game.{}`", s)),
			},
		}
	}
}
