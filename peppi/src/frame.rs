use serde::{
	Serialize,
	ser::SerializeStruct,
};

use super::{
	action_state,
	attack,
	buttons,
	character,
	ground,
	item,
	primitives::{Direction, Port, Position, Velocity},
	triggers,
};

use peppi_derive::Arrow;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct Buttons {
	pub logical: buttons::Logical,
	pub physical: buttons::Physical,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct Triggers {
	pub logical: triggers::Logical,
	pub physical: triggers::Physical,
}

pseudo_bitmask!(StateFlags: u64 {
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

pseudo_enum!(HurtboxState: u8 {
	0 => VULNERABLE,
	1 => INVULNERABLE,
	2 => INTANGIBLE,
});

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct Start {
	pub random_seed: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct End {
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "3.7")] pub latest_finalized_frame: Option<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct Velocities {
	pub autogenous: Velocity,
	pub knockback: Velocity,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct Pre {
	pub position: Position,
	pub direction: Direction,
	pub joystick: Position,
	pub cstick: Position,
	pub triggers: Triggers,
	pub random_seed: u32,
	pub buttons: Buttons,
	pub state: action_state::State,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "1.2")] pub raw_analog_x: Option<u8>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "1.4")] pub damage: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct Post {
	pub character: character::Internal,
	pub state: action_state::State,
	pub position: Position,
	pub direction: Direction,
	pub damage: f32,
	pub shield: f32,
	pub last_attack_landed: Option<attack::Attack>,
	pub combo_count: u8,
	pub last_hit_by: Option<Port>,
	pub stocks: u8,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "0.2")] pub state_age: Option<f32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "2.0")] pub flags: Option<StateFlags>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "2.0")] pub misc_as: Option<f32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "2.0")] pub airborne: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "2.0")] pub ground: Option<ground::Ground>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "2.0")] pub jumps: Option<u8>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "2.0")] pub l_cancel: Option<Option<bool>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "2.1")] pub hurtbox_state: Option<HurtboxState>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "3.5")] pub velocities: Option<Velocities>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "3.8")] pub hitlag: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct Data {
	pub pre: Pre,
	pub post: Post,
}

#[derive(Clone, Debug, PartialEq, Serialize, Arrow)]
pub struct PortData {
	pub leader: Data,
	pub follower: Option<Box<Data>>,
}

#[derive(Clone, Debug, PartialEq, Arrow)]
pub struct Frame<const N: usize> {
	pub ports: [PortData; N],
	#[slippi(version = "2.2")] pub start: Option<Start>,
	#[slippi(version = "3.0")] pub end: Option<End>,
	#[slippi(version = "3.0")] pub items: Option<Vec<item::Item>>,
}

// workaround for Serde not supporting const generics
impl<const N: usize> Serialize for Frame<N> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
		let mut state = serializer.serialize_struct("Frame", 1)?;

		if let Some(start) = self.start {
			state.serialize_field("start", &start)?;
		}

		if let Some(end) = self.end {
			state.serialize_field("end", &end)?;
		}

		state.serialize_field("ports", &self.ports[..])?;

		if let Some(items) = &self.items {
			state.serialize_field("items", &items)?;
		}

		state.end()
	}
}

pub type Frame1 = Frame<1>;
pub type Frame2 = Frame<2>;
pub type Frame3 = Frame<3>;
pub type Frame4 = Frame<4>;
