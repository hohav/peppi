use serde::Serialize;
use serde::ser::SerializeStruct;

use super::{action_state, attack, buttons, character, game, item, triggers};
use super::primitives::{Direction, Position, Velocity};

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

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Start {
	pub random_seed: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct EndV3_7 {
	pub latest_finalized_frame: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct End {
	#[cfg(v3_7)] #[serde(flatten)]
	pub v3_7: EndV3_7,
	#[cfg(not(v3_7))] #[serde(flatten)]
	pub v3_7: Option<EndV3_7>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PreV1_4 {
	pub damage: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PreV1_2 {
	pub raw_analog_x: u8,

	#[cfg(v1_4)] #[serde(flatten)]
	pub v1_4: PreV1_4,
	#[cfg(not(v1_4))] #[serde(flatten)]
	pub v1_4: Option<PreV1_4>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Pre {
	pub position: Position,
	pub direction: Direction,
	pub joystick: Position,
	pub cstick: Position,
	pub triggers: Triggers,
	pub random_seed: u32,
	pub buttons: Buttons,
	pub state: action_state::State,

	#[cfg(v1_2)] #[serde(flatten)]
	pub v1_2: PreV1_2,
	#[cfg(not(v1_2))] #[serde(flatten)]
	pub v1_2: Option<PreV1_2>,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Velocities {
	pub autogenous: Velocity,
	pub knockback: Velocity,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PostV3_5 {
	pub velocities: Velocities,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PostV2_1 {
	pub hurtbox_state: HurtboxState,

	#[cfg(v3_5)] #[serde(flatten)]
	pub v3_5: PostV3_5,
	#[cfg(not(v3_5))] #[serde(flatten)]
	pub v3_5: Option<PostV3_5>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PostV2_0 {
	pub flags: StateFlags,
	pub misc_as: f32,
	pub airborne: bool,
	pub ground: u16,
	pub jumps: u8,
	pub l_cancel: Option<bool>,

	#[cfg(v2_1)] #[serde(flatten)]
	pub v2_1: PostV2_1,
	#[cfg(not(v2_1))] #[serde(flatten)]
	pub v2_1: Option<PostV2_1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PostV0_2 {
	pub state_age: f32,

	#[cfg(v2_0)] #[serde(flatten)]
	pub v2_0: PostV2_0,
	#[cfg(not(v2_0))] #[serde(flatten)]
	pub v2_0: Option<PostV2_0>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Post {
	pub position: Position,
	pub direction: Direction,
	pub damage: f32,
	pub shield: f32,
	pub state: action_state::State,
	pub character: character::Internal,
	pub last_attack_landed: Option<attack::Attack>,
	pub combo_count: u8,
	pub last_hit_by: Option<game::Port>,
	pub stocks: u8,

	#[cfg(v0_2)] #[serde(flatten)]
	pub v0_2: PostV0_2,
	#[cfg(not(v0_2))] #[serde(flatten)]
	pub v0_2: Option<PostV0_2>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Data {
	pub pre: Pre,
	pub post: Post,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Port {
	pub leader: Data,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub follower: Option<Box<Data>>,
}

#[derive(Debug, PartialEq)]
pub struct Frame<const N: usize> {
	#[cfg(v2_2)] pub start: Start,
	#[cfg(not(v2_2))] pub start: Option<Start>,

	#[cfg(v3_0)] pub end: End,
	#[cfg(not(v3_0))] pub end: Option<End>,

	pub ports: [Port; N],
	pub items: Vec<Item>,
}

// workaround for Serde not supporting const generics
impl<const N: usize> Serialize for Frame<N> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
		let mut state = serializer.serialize_struct("Frame", 1)?;

		#[cfg(v2_2)]
		state.serialize_field("start", &self.start)?;
		#[cfg(not(v2_2))]
		if let Some(start) = self.start {
			state.serialize_field("start", &start)?;
		}

		#[cfg(v3_0)]
		state.serialize_field("end", &self.end)?;
		#[cfg(not(v3_0))]
		if let Some(end) = self.end {
			state.serialize_field("end", &end)?;
		}

		state.serialize_field("ports", &self.ports[..])?;
		state.serialize_field("items", &self.items[..])?;

		state.end()
	}
}

pub type Frame1 = Frame<1>;
pub type Frame2 = Frame<2>;
pub type Frame3 = Frame<3>;
pub type Frame4 = Frame<4>;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ItemV3_6 {
	pub owner: Option<game::Port>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ItemV3_2 {
	pub misc: [u8; 4],

	#[cfg(v3_6)] #[serde(flatten)]
	pub v3_6: ItemV3_6,
	#[cfg(not(v3_6))] #[serde(flatten)]
	pub v3_6: Option<ItemV3_6>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Item {
	pub id: u32,
	pub r#type: item::Item,
	pub state: u8,
	pub direction: Direction,
	pub position: Position,
	pub velocity: Velocity,
	pub damage: u16,
	pub timer: f32,

	#[cfg(v3_2)] #[serde(flatten)]
	pub v3_2: ItemV3_2,
	#[cfg(not(v3_2))] #[serde(flatten)]
	pub v3_2: Option<ItemV3_2>,
}
