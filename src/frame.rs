use std::fmt;
use std::io::{Write, Result};

use serde::{Serialize};

use super::{action_state, attack, buttons, character, triggers};
use super::query::{Query};

pseudo_enum!(LCancel:u8 {
	1 => SUCCESSFUL,
	2 => UNSUCCESSFUL,
});

pseudo_enum!(Direction:u8 {
	0 => LEFT,
	1 => RIGHT,
});

#[derive(Copy, Clone, PartialEq, Serialize)]
pub struct Position {
	pub x: f32,
	pub y: f32,
}

impl fmt::Debug for Position {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Buttons {
	pub logical: buttons::Logical,
	pub physical: buttons::Physical,
}

impl Query for Buttons {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()> {
		match query.is_empty() {
			true => write!(f, "{:?}", self),
			_ => match &*query[0] {
				"logical" => write!(f, "{:?}", self.logical),
				"physical" => write!(f, "{:?}", self.physical),
				s => Err(err!("unknown field `buttons.{}`", s)),
			}
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Triggers {
	pub logical: triggers::Logical,
	pub physical: triggers::Physical,
}

pseudo_bitmask!(StateFlags:u64 {
	1u64 << 04 => REFLECT,
	1u64 << 10 => UNTOUCHABLE,
	1u64 << 11 => FAST_FALL,
	1u64 << 13 => HIT_LAG,
	1u64 << 23 => SHIELD,
	1u64 << 25 => HIT_STUN,
	1u64 << 26 => SHIELD_TOUCH,
	1u64 << 29 => POWER_SHIELD,
	1u64 << 35 => FOLLOWER,
	1u64 << 36 => SLEEP,
	1u64 << 38 => DEAD,
	1u64 << 39 => OFF_SCREEN,
});

pseudo_enum!(HurtboxState:u8 {
	0 => VULNERABLE,
	1 => INVULNERABLE,
	2 => INTANGIBLE,
});

pub trait Indexed {
	fn index(&self) -> i32;
}

#[derive(Debug, PartialEq, Serialize)]
pub struct PreV1_4 {
	pub damage: f32,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct PreV1_2 {
	pub raw_analog_x: u8,

	#[cfg(v1_4)]
	#[serde(flatten)]
	pub v1_4: PreV1_4,

	#[cfg(not(v1_4))]
	#[serde(flatten)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v1_4: Option<PreV1_4>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Pre {
	pub index: i32,

	pub position: Position,
	pub direction: Direction,
	pub joystick: Position,
	pub cstick: Position,
	pub triggers: Triggers,
	pub random_seed: u32,
	pub buttons: Buttons,
	pub state: action_state::State,

	#[cfg(v1_2)]
	#[serde(flatten)]
	pub v1_2: PreV1_2,

	#[cfg(not(v1_2))]
	#[serde(flatten)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v1_2: Option<PreV1_2>,
}

impl Indexed for Pre {
	fn index(&self) -> i32 {
		self.index
	}
}

impl Query for Pre {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()> {
		match query.is_empty() {
			true => write!(f, "{:?}", self),
			_ => match &*query[0] {
				"index" => write!(f, "{:?}", self.index),
				"position" => write!(f, "{:?}", self.position),
				"direction" => write!(f, "{:?}", self.direction),
				"joystick" => write!(f, "{:?}", self.joystick),
				"cstick" => write!(f, "{:?}", self.cstick),
				"triggers" => write!(f, "{:?}", self.triggers),
				"random_seed" => write!(f, "{:?}", self.random_seed),
				"buttons" => self.buttons.query(f, &query[1..]),
				"state" => write!(f, "{:?}", self.state),
				"v1_2" => self.v1_2.query(f, &query[1..]),
				_ => self.v1_2.query(f, query),
			},
		}
	}
}

impl Query for PreV1_2 {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()> {
		match query.is_empty() {
			true => write!(f, "{:?}", self),
			_ => match &*query[0] {
				"raw_analog_x" => write!(f, "{:?}", self.raw_analog_x),
				"v1_4" => self.v1_4.query(f, &query[1..]),
				_ => self.v1_4.query(f, query),
			},
		}
	}
}

impl Query for PreV1_4 {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()> {
		match query.is_empty() {
			true => write!(f, "{:?}", self),
			_ => match &*query[0] {
				"damage" => write!(f, "{:?}", self.damage),
				s => Err(err!("unknown field `pre.{}`", s)),
			},
		}
	}
}

#[derive(Debug, PartialEq, Serialize)]
pub struct PostV2_1 {
	pub hurtbox_state: HurtboxState,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct PostV2_0 {
	pub flags: StateFlags,
	pub misc_as: f32,
	pub ground: u16,
	pub jumps: u8,
	pub l_cancel: Option<LCancel>,
	pub airborne: bool,

	#[cfg(v2_1)]
	#[serde(flatten)]
	pub v2_1: PostV2_1,

	#[cfg(not(v2_1))]
	#[serde(flatten)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v2_1: Option<PostV2_1>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct PostV0_2 {
	pub state_age: f32,

	#[cfg(v2_0)]
	#[serde(flatten)]
	pub v2_0: PostV2_0,

	#[cfg(not(v2_0))]
	#[serde(flatten)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v2_0: Option<PostV2_0>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Post {
	pub index: i32,

	pub position: Position,
	pub direction: Direction,
	pub damage: f32,
	pub shield: f32,
	pub state: action_state::State,
	pub character: character::Internal,
	pub last_attack_landed: Option<attack::Attack>,
	pub combo_count: u8,
	pub last_hit_by: u8,
	pub stocks: u8,

	#[cfg(v0_2)]
	#[serde(flatten)]
	pub v0_2: PostV0_2,

	#[cfg(not(v0_2))]
	#[serde(flatten)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v0_2: Option<PostV0_2>,
}

impl Indexed for Post {
	fn index(&self) -> i32 {
		self.index
	}
}

impl Query for Post {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()> {
		match query.is_empty() {
			true => write!(f, "{:?}", self),
			_ => match &*query[0] {
				"index" => write!(f, "{:?}", self.index),
				"position" => write!(f, "{:?}", self.position),
				"direction" => write!(f, "{:?}", self.direction),
				"damage" => write!(f, "{:?}", self.damage),
				"shield" => write!(f, "{:?}", self.shield),
				"state" => write!(f, "{:?}", self.state),
				"character" => write!(f, "{:?}", self.character),
				"last_attack_landed" => write!(f, "{:?}", self.last_attack_landed),
				"combo_count" => write!(f, "{:?}", self.combo_count),
				"last_hit_by" => write!(f, "{:?}", self.last_hit_by),
				"stocks" => write!(f, "{:?}", self.stocks),
				"v0_2" => self.v0_2.query(f, &query[1..]),
				_ => self.v0_2.query(f, query),
			},
		}
	}
}

impl Query for PostV0_2 {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()> {
		match query.is_empty() {
			true => write!(f, "{:?}", self),
			_ => match &*query[0] {
				"state_age" => write!(f, "{:?}", self.state_age),
				"v2_0" => self.v2_0.query(f, &query[1..]),
				_ => self.v2_0.query(f, query),
			},
		}
	}
}

impl Query for PostV2_0 {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()> {
		match query.is_empty() {
			true => write!(f, "{:?}", self),
			_ => match &*query[0] {
				"flags" => write!(f, "{:?}", self.flags),
				"misc_as" => write!(f, "{:?}", self.misc_as),
				"ground" => write!(f, "{:?}", self.ground),
				"jumps" => write!(f, "{:?}", self.jumps),
				"l_cancel" => write!(f, "{:?}", self.l_cancel),
				"airborne" => write!(f, "{:?}", self.l_cancel),
				"v2_1" => self.v2_1.query(f, &query[1..]),
				_ => self.v2_1.query(f, query),
			},
		}
	}
}

impl Query for PostV2_1 {
	fn query(&self, f:&mut dyn Write, query:&[&str]) -> Result<()> {
		match query.is_empty() {
			true => write!(f, "{:?}", self),
			_ => match &*query[0] {
				"hurtbox_state" => write!(f, "{:?}", self.hurtbox_state),
				s => Err(err!("unknown field `post.{}`", s)),
			},
		}
	}
}
