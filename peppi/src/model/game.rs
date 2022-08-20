use std::{
	any::Any,
	error,
	fmt::{self, Display, Debug, Formatter},
	io,
};

use serde::Serialize;

use crate::{
	model::{
		enums::{character, stage},
		frame::{Frame, PortData},
		metadata,
		primitives::Port,
		slippi,
	},
	serde::de,
};

pub const NUM_PORTS: usize = 4;
pub const MAX_PLAYERS: usize = 6;

/// Frame indexes start at -123, and reach 0 at "Go!".
pub const FIRST_FRAME_INDEX: i32 = -123;

/// We can parse files with higher versions than this, but we won't expose all information.
/// When converting a replay with a higher version number to another format like Arrow,
/// the conversion will be lossy.
pub const MAX_SUPPORTED_VERSION: slippi::Version = slippi::Version(3, 12, 0);

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

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
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

pseudo_enum!(Language: u8 {
	0 => JAPANESE,
	1 => ENGLISH,
});

/// Information about the "Universal Controller Fix" mod.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Ucf {
	pub dash_back: Option<DashBack>,
	pub shield_drop: Option<ShieldDrop>,
}

/// Netplay name, connect code, and Slippi UID.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Netplay {
	pub name: String,
	pub code: String,
	/// Slippi UID (added: v3.11)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub suid: Option<String>,
}

/// Information about each player such as character, team, stock count, etc.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Player {
	pub port: Port,

	pub character: character::External,
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
	pub name_tag: Option<String>,
	/// netplay info (added: v3.9)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub netplay: Option<Netplay>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
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
	pub stage: stage::Stage,
	pub timer: u32,
	pub item_spawn_bitfield: [u8; 5],
	pub damage_ratio: f32,
	pub players: Vec<Player>,
	pub random_seed: u32,

	/// mostly-redundant copy of the raw start block, for round-tripping
	#[serde(skip)] #[doc(hidden)]
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
	pub language: Option<Language>,
}

impl Start {
	pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
		de::game_start(&mut &bytes[..])
	}
}

pseudo_enum!(EndMethod: u8 {
	0 => UNRESOLVED,
	1 => TIME,
	2 => GAME,
	3 => RESOLVED,
	7 => NO_CONTEST,
});

/// Information about the end of the game.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct End {
	/// how the game ended
	pub method: EndMethod,

	/// mostly-redundant copy of the raw start block, for round-tripping
	#[serde(skip)] #[doc(hidden)]
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

/// Type-erasing wrapper for frame data. This allows `Game` not to be generic.
#[derive(Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Frames {
	P1(Vec<Frame<1>>),
	P2(Vec<Frame<2>>),
	P3(Vec<Frame<3>>),
	P4(Vec<Frame<4>>),
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

impl error::Error for UnexpectedPortCountError { }

impl Frames {
	/// Tries to downcast our inner value to a `Vec<Frame<N>>`.
	pub fn downcast<const N: usize>(&self) -> Result<&Vec<Frame<N>>, UnexpectedPortCountError> {
		match self {
			Self::P1(frames) => (frames as &dyn Any)
				.downcast_ref::<Vec<Frame<N>>>()
				.ok_or(UnexpectedPortCountError {expected: N, actual: 1}),
			Self::P2(frames) => (frames as &dyn Any)
				.downcast_ref::<Vec<Frame<N>>>()
				.ok_or(UnexpectedPortCountError {expected: N, actual: 2}),
			Self::P3(frames) => (frames as &dyn Any)
				.downcast_ref::<Vec<Frame<N>>>()
				.ok_or(UnexpectedPortCountError {expected: N, actual: 3}),
			Self::P4(frames) => (frames as &dyn Any)
				.downcast_ref::<Vec<Frame<N>>>()
				.ok_or(UnexpectedPortCountError {expected: N, actual: 4}),
		}
	}

	pub fn frame_count(&self) -> usize {
		match self {
			Self::P1(frames) => frames.len(),
			Self::P2(frames) => frames.len(),
			Self::P3(frames) => frames.len(),
			Self::P4(frames) => frames.len(),
		}
	}

	pub fn port_count(&self) -> usize {
		match self {
			Self::P1(_) => 1,
			Self::P2(_) => 2,
			Self::P3(_) => 3,
			Self::P4(_) => 4,
		}
	}

	/// Returns the port data for this frame & port index.
	/// Frames are indexed from zero here, not -123.
	pub fn port_data(&self, frame_idx: usize, port_idx: usize) -> Option<&PortData> {
		match self {
			Self::P1(frames) => frames.get(frame_idx).and_then(|f| f.ports.get(port_idx)),
			Self::P2(frames) => frames.get(frame_idx).and_then(|f| f.ports.get(port_idx)),
			Self::P3(frames) => frames.get(frame_idx).and_then(|f| f.ports.get(port_idx)),
			Self::P4(frames) => frames.get(frame_idx).and_then(|f| f.ports.get(port_idx)),
		}
	}
}

/// Binary blob of Gecko codes in use.
///
/// Currently unparsed, but still needed for round-tripping.
#[derive(Debug, PartialEq)]
pub struct GeckoCodes {
	pub bytes: Vec<u8>,
	pub actual_size: u16,
}

/// Replay data for a single game of Melee.
///
/// See https://github.com/project-slippi/slippi-wiki/blob/master/SPEC.md.
#[derive(PartialEq, Serialize)]
pub struct Game {
	pub start: Start,
	pub end: End,
	pub frames: Frames,
	#[serde(skip)]
	pub metadata: metadata::Metadata,
	#[serde(rename = "metadata")]
	pub metadata_raw: serde_json::Map<String, serde_json::Value>,
	#[serde(skip)] #[doc(hidden)]
	pub gecko_codes: Option<GeckoCodes>,
}

impl Debug for Game {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct("Game")
			.field("metadata", &self.metadata)
			.field("start", &self.start)
			.field("end", &self.end)
			.field("frames", &self.frames)
			.finish()
	}
}
