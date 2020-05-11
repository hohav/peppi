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

	// v1.2
	#[serde(skip_serializing_if = "Option::is_none")]
	pub raw_analog_x: Option<u8>,

	// v1.4
	#[serde(skip_serializing_if = "Option::is_none")]
	pub damage: Option<f32>,
}

impl Indexed for Pre {
	fn index(&self) -> i32 {
		self.index
	}
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

	// v0.2
	#[serde(skip_serializing_if = "Option::is_none")]
	pub state_age: Option<f32>,

	// v2.0
	#[serde(skip_serializing_if = "Option::is_none")]
	pub flags: Option<StateFlags>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub misc_as: Option<f32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ground: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub jumps: Option<u8>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub l_cancel: Option<Option<LCancel>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub airborne: Option<bool>,

	// v2.1
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hurtbox_state: Option<HurtboxState>,
}

impl Indexed for Post {
	fn index(&self) -> i32 {
		self.index
	}
}
