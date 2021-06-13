use std::fmt::{self, Debug};

use serde::{Deserialize, Serialize};

use super::{
	character,
	frame,
	metadata,
	primitives::Port,
	slippi,
	stage,
};

pub const NUM_PORTS: usize = 4;
pub const FIRST_FRAME_INDEX: i32 = -123;

/// We can parse files with higher versions than this, but we won't expose all information.
/// When converting a replay with a higher version number to another format like Arrow,
/// the conversion will be lossy.
pub const MAX_SUPPORTED_VERSION: slippi::Version = slippi::Version(3, 9, 0);

pseudo_enum!(PlayerType: u8 {
	0 => HUMAN,
	1 => CPU,
	2 => DEMO,
});

pseudo_enum!(TeamColor: u8 {
	0 => RED,
	1 => BLUE,
	2 => GREEN,
});

pseudo_enum!(TeamShade: u8 {
	0 => NORMAL,
	1 => LIGHT,
	2 => DARK,
});

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Team {
	pub color: TeamColor,
	pub shade: TeamShade,
}

pseudo_enum!(DashBack: u32 {
	1 => UCF,
	2 => ARDUINO,
});

pseudo_enum!(ShieldDrop: u32 {
	1 => UCF,
	2 => ARDUINO,
});

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Ucf {
	pub dash_back: Option<DashBack>,
	pub shield_drop: Option<ShieldDrop>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Netplay {
	pub name: String,
	pub code: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Player {
	pub port: Port,

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

	// v1_0
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ucf: Option<Ucf>,
	// v1_3
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name_tag: Option<String>,
	// v3.9
	#[serde(skip_serializing_if = "Option::is_none")]
	pub netplay: Option<Netplay>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Scene {
	pub minor: u8,
	pub major: u8,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Start {
	pub slippi: slippi::Slippi,
	pub bitfield: [u8; 3],
	pub is_teams: bool,
	pub item_spawn_frequency: i8,
	pub self_destruct_score: i8,
	pub stage: stage::Stage,
	pub timer: u32,
	pub item_spawn_bitfield: [u8; 5],
	pub damage_ratio: f32,
	pub players: Vec<Player>,
	pub random_seed: u32,
	// v1.5
	#[serde(skip_serializing_if = "Option::is_none")]
	pub is_pal: Option<bool>,
	// v2.0
	#[serde(skip_serializing_if = "Option::is_none")]
	pub is_frozen_ps: Option<bool>,
	// v3.7
	#[serde(skip_serializing_if = "Option::is_none")]
	pub scene: Option<Scene>,
}

pseudo_enum!(EndMethod: u8 {
	0 => UNRESOLVED,
	1 => TIME,
	2 => GAME,
	3 => RESOLVED,
	7 => NO_CONTEST,
});

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct End {
	pub method: EndMethod,
	// v2.0
	#[serde(skip_serializing_if = "Option::is_none")]
	pub lras_initiator: Option<Option<Port>>,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Frames {
	P1(Vec<frame::Frame1>),
	P2(Vec<frame::Frame2>),
	P3(Vec<frame::Frame3>),
	P4(Vec<frame::Frame4>),
}

impl Frames {
	pub fn len(&self) -> usize {
		match self {
			Self::P1(frames) => frames.len(),
			Self::P2(frames) => frames.len(),
			Self::P3(frames) => frames.len(),
			Self::P4(frames) => frames.len(),
		}
	}
}

/// Replay data for a single game of Melee.
///
/// See https://github.com/project-slippi/slippi-wiki/blob/master/SPEC.md.
#[derive(PartialEq, Serialize)]
pub struct Game {
	pub start: Start,
	pub end: End,
	pub frames: Frames,
	pub metadata: metadata::Metadata,
	pub metadata_raw: serde_json::Map<String, serde_json::Value>,
}

impl Debug for Game {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Game")
			.field("metadata", &self.metadata)
			.field("start", &self.start)
			.field("end", &self.end)
			.field("frames", &self.frames)
			.finish()
	}
}
