use std::{
	error,
	fmt::{self, Debug, Display, Formatter},
	io,
};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;

use crate::{
	model::{
		frame,
		shift_jis::MeleeString,
		slippi,
	},
	serde::de,
};

/// We can parse files with higher versions than this, but we won't expose all information.
/// When converting a replay with a higher version number to another format like Arrow,
/// the conversion will be lossy.
pub const MAX_SUPPORTED_VERSION: slippi::Version = slippi::Version(3, 12, 0);

pub const NUM_PORTS: u8 = 4;

#[derive(
	Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, IntoPrimitive, TryFromPrimitive,
)]
#[repr(u8)]
pub enum Port {
	P1 = 0,
	P2 = 1,
	P3 = 2,
	P4 = 3,
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
	HUMAN = 0,
	CPU = 1,
	DEMO = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Team {
	pub color: u8,
	pub shade: u8,
}

/// Information about the "Universal Controller Fix" mod.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Ucf {
	pub dash_back: u32,
	pub shield_drop: u32,
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
	pub raw_bytes: Vec<u8>,

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
	pub language: Option<u8>,
}

impl Start {
	pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
		de::game_start(&mut &bytes[..])
	}
}

/// Information about the end of the game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct End {
	/// how the game ended
	pub method: u8,

	/// mostly-redundant copy of the raw start block, for round-tripping
	#[serde(skip)]
	#[doc(hidden)]
	pub raw_bytes: Vec<u8>,

	/// player who LRAS'd, if any (added: v2.0)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub lras_initiator: Option<Option<Port>>,
}

impl End {
	pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
		de::game_end(&mut &bytes[..])
	}
}

#[derive(Clone, Copy, Debug)]
pub struct UnexpectedPortCountError {
	pub expected: usize,
	pub actual: usize,
}

impl Display for UnexpectedPortCountError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
		write!(f, "expected {} ports, got {}", self.expected, self.actual)
	}
}

impl error::Error for UnexpectedPortCountError {}

/// Binary blob of Gecko codes in use.
///
/// Currently unparsed, but still needed for round-tripping.
#[derive(Debug, PartialEq, Eq)]
pub struct GeckoCodes {
	pub bytes: Vec<u8>,
	pub actual_size: u32,
}

pub struct Game {
	pub start: Start,
	pub end: Option<End>,
	pub frames: frame::Frame,
	pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
	pub gecko_codes: Option<GeckoCodes>,
}
