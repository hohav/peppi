use std::fmt;

use serde::Serialize;

use super::{character, frame, metadata, stage};

pub const NUM_PORTS:usize = 4;
pub const FIRST_FRAME_INDEX:i32 = -123;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum Port {
	P1 = 0,
	P2 = 1,
	P3 = 2,
	P4 = 3,
}

query_impl!(Port);

#[derive(Debug, PartialEq, Serialize)]
pub struct SlippiVersion(pub u8, pub u8, pub u8);

query_impl!(SlippiVersion);

#[derive(Debug, PartialEq, Serialize)]
pub struct Slippi {
	pub version: SlippiVersion,
}

query_impl!(Slippi, self, f, config, query {
	match &*query[0] {
		"version" => self.version.query(f, config, &query[1..]),
		s => Err(err!("unknown field `slippi.{}`", s)),
	}
});

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

query_impl!(Team, self, f, config, query {
	match &*query[0] {
		"color" => self.color.query(f, config, &query[1..]),
		"shade" => self.shade.query(f, config, &query[1..]),
		s => Err(err!("unknown field `team.{}`", s)),
	}
});

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

query_impl!(Ucf, self, f, config, query {
	match &*query[0] {
		"dash_back" => self.dash_back.query(f, config, &query[1..]),
		"shield_drop" => self.shield_drop.query(f, config, &query[1..]),
		s => Err(err!("unknown field `ucf.{}`", s)),
	}
});

#[derive(Debug, PartialEq, Serialize)]
pub struct PlayerV1_3 {
	pub name_tag: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct PlayerV1_0 {
	pub ucf: Ucf,

	#[cfg(v1_3)] #[serde(flatten)]
	pub v1_3: PlayerV1_3,
	#[cfg(not(v1_3))] #[serde(flatten)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v1_3: Option<PlayerV1_3>,
}

#[derive(Debug, PartialEq, Serialize)]
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
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v1_0: Option<PlayerV1_0>,
}

query_impl!(PlayerV1_3, self, f, config, query {
	match &*query[0] {
		"name_tag" => self.name_tag.query(f, config, &query[1..]),
		s => Err(err!("unknown field `player.{}`", s)),
	}
});

query_impl!(PlayerV1_0, self, f, config, query {
	match &*query[0] {
		"ucf" => self.ucf.query(f, config, &query[1..]),
		"v1_3" => self.v1_3.query(f, config, &query[1..]),
		_ => self.v1_3.query(f, config, query),
	}
});

query_impl!(Player, self, f, config, query {
	match &*query[0] {
		"character" => self.character.query(f, config, &query[1..]),
		"r#type" => self.r#type.query(f, config, &query[1..]),
		"stocks" => self.stocks.query(f, config, &query[1..]),
		"costume" => self.costume.query(f, config, &query[1..]),
		"team" => self.team.query(f, config, &query[1..]),
		"handicap" => self.handicap.query(f, config, &query[1..]),
		"bitfield" => self.bitfield.query(f, config, &query[1..]),
		"cpu_level" => self.cpu_level.query(f, config, &query[1..]),
		"offense_ratio" => self.offense_ratio.query(f, config, &query[1..]),
		"defense_ratio" => self.defense_ratio.query(f, config, &query[1..]),
		"model_scale" => self.model_scale.query(f, config, &query[1..]),
		"v1_0" => self.v1_0.query(f, config, &query[1..]),
		_ => self.v1_0.query(f, config, query),
	}
});

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
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v1_5: Option<StartV1_5>,
}

query_impl!(StartV2_0, self, f, config, query {
	match &*query[0] {
		"is_frozen_ps" => self.is_frozen_ps.query(f, config, &query[1..]),
		s => Err(err!("unknown field `start.{}`", s)),
	}
});

query_impl!(StartV1_5, self, f, config, query {
	match &*query[0] {
		"is_pal" => self.is_pal.query(f, config, &query[1..]),
		"v2_0" => self.v2_0.query(f, config, &query[1..]),
		_ => self.v2_0.query(f, config, query),
	}
});

query_impl!(Start, self, f, config, query {
	match &*query[0] {
		"slippi" => self.slippi.query(f, config, &query[1..]),
		"bitfield" => self.bitfield.query(f, config, &query[1..]),
		"is_teams" => self.is_teams.query(f, config, &query[1..]),
		"item_spawn_frequency" => self.item_spawn_frequency.query(f, config, &query[1..]),
		"self_destruct_score" => self.self_destruct_score.query(f, config, &query[1..]),
		"stage" => self.stage.query(f, config, &query[1..]),
		"timer" => self.timer.query(f, config, &query[1..]),
		"item_spawn_bitfield" => self.item_spawn_bitfield.query(f, config, &query[1..]),
		"damage_ratio" => self.damage_ratio.query(f, config, &query[1..]),
		"players" => self.players.query(f, config, &query[1..]),
		"random_seed" => self.random_seed.query(f, config, &query[1..]),
		"v1_5" => self.v1_5.query(f, config, &query[1..]),
		_ => self.v1_5.query(f, config, query),
	}
});

pseudo_enum!(EndMethod:u8 {
	0 => UNRESOLVED,
	1 => TIME,
	2 => GAME,
	3 => RESOLVED,
	7 => NO_CONTEST,
});

#[derive(Debug, PartialEq, Serialize)]
pub struct EndV2_0 {
	pub lras_initiator: Option<Port>,
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

query_impl!(EndV2_0, self, f, config, query {
	match &*query[0] {
		"lras_initiator" => self.lras_initiator.query(f, config, &query[1..]),
		s => Err(err!("unknown field `end.{}`", s)),
	}
});

query_impl!(End, self, f, config, query {
	match &*query[0] {
		"method" => self.method.query(f, config, &query[1..]),
		"v2_0" => self.v2_0.query(f, config, &query[1..]),
		_ => self.v2_0.query(f, config, query),
	}
});

fn skip_frames<T>(_:&T) -> bool {
	!unsafe { super::CONFIG.frames }
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

query_impl!(Frames, self, f, config, query {
	match self {
		Self::P1(frames) => frames.query(f, config, query),
		Self::P2(frames) => frames.query(f, config, query),
		Self::P3(frames) => frames.query(f, config, query),
		Self::P4(frames) => frames.query(f, config, query),
	}
});

#[derive(PartialEq, Serialize)]
pub struct Game {
	pub start: Start,
	pub end: End,
	#[serde(skip_serializing_if = "skip_frames")]
	pub frames: Frames,
	pub metadata: metadata::Metadata,
}

impl fmt::Debug for Game {
	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
		match unsafe { super::CONFIG.frames } {
			true => f.debug_struct("Frames")
				.field("metadata", &self.metadata)
				.field("start", &self.start)
				.field("end", &self.end)
				.field("frames", &self.frames)
				.finish(),
			_ => f.debug_struct("Frames")
				.field("metadata", &self.metadata)
				.field("start", &self.start)
				.field("end", &self.end)
				.field("frames", &self.frames.len())
				.finish(),
		}
	}
}

query_impl!(Game, self, f, config, query {
	match &*query[0] {
		"start" => self.start.query(f, config, &query[1..]),
		"end" => self.end.query(f, config, &query[1..]),
		"frames" => self.frames.query(f, config, &query[1..]),
		"metadata" => self.metadata.query(f, config, &query[1..]),
		s => Err(err!("unknown field `game.{}`", s)),
	}
});
