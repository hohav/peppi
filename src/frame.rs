use serde::{Serialize};

use super::{action_state, attack, buttons, character, triggers};

pseudo_enum!(LCancel:u8 {
	1 => SUCCESSFUL,
	2 => UNSUCCESSFUL,
});

pseudo_enum!(Direction:u8 {
	0 => LEFT,
	1 => RIGHT,
});

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Position {
	pub x: f32,
	pub y: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Buttons {
	pub logical: buttons::Logical,
	pub physical: buttons::Physical,
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
