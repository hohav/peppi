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

frame_data!(Start, StartCol {
	random_seed: u32,
});

frame_data!(EndV3_7, EndV3_7Col {
	latest_finalized_frame: i32,
});

frame_data!(End, EndCol {
}, v3_7: EndV3_7, EndV3_7Col);

port_data!(PreV1_4, PreV1_4Col {
	damage: f32,
});

port_data!(PreV1_2, PreV1_2Col {
	raw_analog_x: u8,
}, v1_4: PreV1_4, PreV1_4Col);

port_data!(Pre, PreCol {
	position: Position,
	direction: Direction,
	joystick: Position,
	cstick: Position,
	triggers: Triggers,
	random_seed: u32,
	buttons: Buttons,
	state: action_state::State,
}, v1_2: PreV1_2, PreV1_2Col);

port_data!(Velocities, VelocitiesCol {
	autogenous: Velocity,
	knockback: Velocity,
});

port_data!(PostV3_8, PostV3_8Col {
	hitlag: f32,
});

port_data!(PostV3_5, PostV3_5Col {
	velocities: Velocities,
}, v3_8: PostV3_8, PostV3_8Col);

port_data!(PostV2_1, PostV2_1Col {
	hurtbox_state: HurtboxState,
}, v3_5: PostV3_5, PostV3_5Col);

port_data!(PostV2_0, PostV2_0Col {
	flags: StateFlags,
	misc_as: f32,
	airborne: bool,
	ground: u16,
	jumps: u8,
	l_cancel: Option<bool>,
}, v2_1: PostV2_1, PostV2_1Col);

port_data!(PostV0_2, PostV0_2Col {
	state_age: f32,
}, v2_0: PostV2_0, PostV2_0Col);

port_data!(Post, PostCol {
	position: Position,
	direction: Direction,
	damage: f32,
	shield: f32,
	state: action_state::State,
	character: character::Internal,
	last_attack_landed: Option<attack::Attack>,
	combo_count: u8,
	last_hit_by: Option<game::Port>,
	stocks: u8,
}, v0_2: PostV0_2, PostV0_2Col);

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

item_data!(ItemV3_6, ItemV3_6Col {
	owner: Option<game::Port>,
});

item_data!(ItemV3_2, ItemV3_2Col {
	misc: [u8; 4],
}, v3_6: ItemV3_6, ItemV3_6Col);

item_data!(Item, ItemCol {
	id: u32,
	r#type: item::Item,
	state: u8,
	direction: Direction,
	position: Position,
	velocity: Velocity,
	damage: u16,
	timer: f32,
}, v3_2: ItemV3_2, ItemV3_2Col);

#[derive(Debug, PartialEq)]
pub struct Frame<const N: usize> {
	pub ports: [Port; N],

	#[cfg(v2_2)] pub start: Start,
	#[cfg(not(v2_2))] pub start: Option<Start>,

	#[cfg(v3_0)] pub end: End,
	#[cfg(not(v3_0))] pub end: Option<End>,

	#[cfg(v3_0)] pub items: Vec<Item>,
	#[cfg(not(v3_0))] pub items: Option<Vec<Item>>,
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

		#[cfg(v3_0)]
		state.serialize_field("items", &self.items)?;
		#[cfg(not(v3_0))]
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
