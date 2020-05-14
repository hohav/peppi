use std::fmt;
use serde::{Serialize};

use super::{character, frame, metadata, stage};

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

	// v1.0
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ucf: Option<Ucf>,

	// v1.3
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name_tag: Option<String>,
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

	// v1.5
	#[serde(skip_serializing_if = "Option::is_none")]
	pub is_pal: Option<bool>,

	// v2.0
	#[serde(skip_serializing_if = "Option::is_none")]
	pub is_frozen_ps: Option<bool>,
}

pseudo_enum!(EndMethod:u8 {
	0 => UNRESOLVED,
	1 => TIME,
	2 => GAME,
	3 => RESOLVED,
	7 => NO_CONTEST,
});

#[derive(Debug, PartialEq, Serialize)]
pub struct End {
	pub method: EndMethod,

	// v2.0
	#[serde(skip_serializing_if = "Option::is_none")]
	pub lras_initiator: Option<i8>,
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

#[derive(Debug, PartialEq, Serialize)]
pub struct Game {
	pub start: Start,
	pub end: End,
	pub ports: [Option<Port>; NUM_PORTS],
	pub metadata: metadata::Metadata,
}
