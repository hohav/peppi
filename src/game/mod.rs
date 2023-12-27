use std::fmt::{self, Debug, Display, Formatter};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::{
	frame::transpose,
	game::shift_jis::MeleeString,
	io::slippi::{self, Version},
};

pub mod immutable;
pub mod mutable;
pub mod shift_jis;

pub const NUM_PORTS: usize = 4;
pub const MAX_PLAYERS: usize = 6;
pub const ICE_CLIMBERS: u8 = 14;

#[repr(u8)]
#[derive(
	Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, IntoPrimitive, TryFromPrimitive,
)]
pub enum Port {
	P1 = 0,
	P2 = 1,
	P3 = 2,
	P4 = 3,
}

impl Port {
	pub fn parse(s: &str) -> Result<Self, String> {
		match s {
			"P1" => Ok(Port::P1),
			"P2" => Ok(Port::P2),
			"P3" => Ok(Port::P3),
			"P4" => Ok(Port::P4),
			_ => Err(format!("invalid port: {}", s)),
		}
	}
}

impl Display for Port {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		use Port::*;
		match *self {
			P1 => write!(f, "P1"),
			P2 => write!(f, "P2"),
			P3 => write!(f, "P3"),
			P4 => write!(f, "P4"),
		}
	}
}

impl Default for Port {
	fn default() -> Self {
		Self::P1
	}
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TryFromPrimitive)]
pub enum PlayerType {
	Human = 0,
	Cpu = 1,
	Demo = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Team {
	pub color: u8,
	pub shade: u8,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TryFromPrimitive)]
pub enum DashBack {
	Ucf = 1,
	Arduino = 2,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TryFromPrimitive)]
pub enum ShieldDrop {
	Ucf = 1,
	Arduino = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TryFromPrimitive)]
pub enum Language {
	Japanese = 0,
	English = 1,
}

/// Information about the "Universal Controller Fix" mod.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Ucf {
	pub dash_back: Option<DashBack>,
	pub shield_drop: Option<ShieldDrop>,
}

/// Netplay name, connect code, and Slippi UID.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Netplay {
	pub name: MeleeString,

	pub code: MeleeString,

	/// Slippi UID (added: v3.11)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub suid: Option<String>,
}

/// Information about each player such as character, team, stock count, etc.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Player {
	pub port: Port,

	pub character: u8,

	pub r#type: PlayerType,

	/// starting stock count
	pub stocks: u8,

	pub costume: u8,

	pub team: Option<Team>,

	/// handicap level; affects `offense_ratio` & `defense_ratio`
	pub handicap: u8,

	/// miscellaneous flags (metal, stamina mode, etc)
	pub bitfield: u8,

	pub cpu_level: Option<u8>,

	/// knockback multiplier when this player hits another
	pub offense_ratio: f32,

	/// knockback multiplier when this player is hit
	pub defense_ratio: f32,

	pub model_scale: f32,

	/// UCF info (added: v1.0)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ucf: Option<Ucf>,

	/// in-game name-tag (added: v1.3)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name_tag: Option<MeleeString>,

	/// netplay info (added: v3.9)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub netplay: Option<Netplay>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Scene {
	pub minor: u8,
	pub major: u8,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Bytes(pub Vec<u8>);

impl Debug for Bytes {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		write!(f, "Bytes {{ len: {} }}", self.0.len())
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Match {
	pub id: String,
	pub game: u32,
	pub tiebreaker: u32,
}

/// Information used to initialize the game such as the game mode, settings, characters & stage.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Start {
	pub slippi: slippi::Slippi,

	pub bitfield: [u8; 4],

	pub is_raining_bombs: bool,

	pub is_teams: bool,

	pub item_spawn_frequency: i8,

	pub self_destruct_score: i8,

	pub stage: u16,

	pub timer: u32,

	pub item_spawn_bitfield: [u8; 5],

	pub damage_ratio: f32,

	pub players: Vec<Player>,

	pub random_seed: u32,

	/// mostly-redundant copy of the raw start block, for round-tripping
	#[serde(skip)]
	#[doc(hidden)]
	pub bytes: Bytes,

	/// (added: v1.5)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub is_pal: Option<bool>,

	/// (added: v2.0)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub is_frozen_ps: Option<bool>,

	/// (added: v3.7)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub scene: Option<Scene>,

	/// (added: v3.12)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub language: Option<Language>,

	/// (added: v3.14)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub r#match: Option<Match>,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TryFromPrimitive)]
pub enum EndMethod {
	Unresolved = 0,
	Time = 1,
	Game = 2,
	Resolved = 3,
	NoContest = 7,
}

/// Player placements.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlayerEnd {
	pub port: Port,
	pub placement: u8,
}

/// Information about the end of the game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct End {
	/// how the game ended
	pub method: EndMethod,

	/// mostly-redundant copy of the raw end block, for round-tripping
	#[serde(skip)]
	#[doc(hidden)]
	pub bytes: Bytes,

	/// player who LRAS'd, if any (added: v2.0)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub lras_initiator: Option<Option<Port>>,

	/// player-specific data (added: v3.13)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub players: Option<Vec<PlayerEnd>>,
}

impl End {
	pub(crate) fn size(version: Version) -> usize {
		if version.gte(3, 13) {
			6
		} else if version.gte(2, 0) {
			2
		} else {
			1
		}
	}
}

/// Binary blob of Gecko codes in use.
///
/// Currently unparsed, but still needed for round-tripping.
#[derive(Debug, PartialEq, Eq)]
pub struct GeckoCodes {
	pub bytes: Vec<u8>,
	pub actual_size: u32,
}

pub trait Game {
	fn start(&self) -> &Start;
	fn end(&self) -> &Option<End>;
	fn metadata(&self) -> &Option<Map<String, Value>>;
	fn gecko_codes(&self) -> &Option<GeckoCodes>;

	/// Duration of the game in frames.
	fn len(&self) -> usize;

	/// Combines all data for a single frame into a struct.
	/// Avoid calling this if you need maximum performance.
	fn frame(&self, idx: usize) -> transpose::Frame;
}
