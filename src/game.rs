use std::fmt::{Debug, Display, Formatter, Result};

use serde::{Deserialize, Serialize};

use super::{character, frame, metadata, stage};

pub const NUM_PORTS: usize = 4;
pub const FIRST_FRAME_INDEX: i32 = -123;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum Port {
	P1 = 0,
	P2 = 1,
	P3 = 2,
	P4 = 3,
}

impl Display for Port {
	fn fmt(&self, f: &mut Formatter) -> Result {
		use Port::*;
		match *self {
			P1 => write!(f, "P1"),
			P2 => write!(f, "P2"),
			P3 => write!(f, "P3"),
			P4 => write!(f, "P4"),
		}
	}
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct SlippiVersion(pub u8, pub u8, pub u8);

impl Display for SlippiVersion {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{}.{}.{}", self.0, self.1, self.2)
	}
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Slippi {
	pub version: SlippiVersion,
}

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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Ucf {
	pub dash_back: Option<DashBack>,
	pub shield_drop: Option<ShieldDrop>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PlayerV1_3 {
	pub name_tag: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PlayerV1_0 {
	pub ucf: Ucf,

	#[cfg(v1_3)] #[serde(flatten)]
	pub v1_3: PlayerV1_3,
	#[cfg(not(v1_3))] #[serde(flatten)]
	pub v1_3: Option<PlayerV1_3>,
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

	#[cfg(v1_0)] #[serde(flatten)]
	pub v1_0: PlayerV1_0,
	#[cfg(not(v1_0))] #[serde(flatten)]
	pub v1_0: Option<PlayerV1_0>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Scene {
	pub minor: u8,
	pub major: u8,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct StartV3_7 {
	pub scene: Scene,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct StartV2_0 {
	pub is_frozen_ps: bool,

	#[cfg(v3_7)] #[serde(flatten)]
	pub v3_7: StartV3_7,
	#[cfg(not(v3_7))] #[serde(flatten)]
	pub v3_7: Option<StartV3_7>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct StartV1_5 {
	pub is_pal: bool,

	#[cfg(v2_0)] #[serde(flatten)]
	pub v2_0: StartV2_0,
	#[cfg(not(v2_0))] #[serde(flatten)]
	pub v2_0: Option<StartV2_0>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Start {
	pub slippi: Slippi,
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

	#[cfg(v1_5)] #[serde(flatten)]
	pub v1_5: StartV1_5,
	#[cfg(not(v1_5))] #[serde(flatten)]
	pub v1_5: Option<StartV1_5>,
}

pseudo_enum!(EndMethod: u8 {
	0 => UNRESOLVED,
	1 => TIME,
	2 => GAME,
	3 => RESOLVED,
	7 => NO_CONTEST,
});

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EndV2_0 {
	pub lras_initiator: Option<Port>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct End {
	pub method: EndMethod,

	#[cfg(v2_0)] #[serde(flatten)]
	pub v2_0: EndV2_0,
	#[cfg(not(v2_0))] #[serde(flatten)]
	pub v2_0: Option<EndV2_0>,
}

fn skip_frames<T>(_: &T) -> bool {
	unsafe { super::CONFIG.skip_frames }
}

#[derive(PartialEq, Serialize)]
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

impl Debug for Frames {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match unsafe { super::CONFIG.skip_frames } {
			true => f.debug_list().finish(),
			_ => match self {
				Self::P1(frames) => f.debug_list().entries(frames.iter()).finish(),
				Self::P2(frames) => f.debug_list().entries(frames.iter()).finish(),
				Self::P3(frames) => f.debug_list().entries(frames.iter()).finish(),
				Self::P4(frames) => f.debug_list().entries(frames.iter()).finish(),
			}
		}
	}
}

#[derive(PartialEq, Serialize)]
pub struct Game {
	pub start: Start,
	pub end: End,
	#[serde(skip_serializing_if = "skip_frames")]
	pub frames: Frames,
	pub metadata: metadata::Metadata,
	pub metadata_raw: serde_json::Map<String, serde_json::Value>,
}

impl Debug for Game {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		f.debug_struct("Game")
			.field("metadata", &self.metadata)
			.field("start", &self.start)
			.field("end", &self.end)
			.field("frames", &self.frames)
			.finish()
	}
}
