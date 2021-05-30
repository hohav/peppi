use serde::{
	Serialize,
	ser::SerializeStruct,
};

use super::{
	action_state,
	attack,
	buttons,
	character,
	item,
	primitives::{Direction, Port, Position, Velocity},
	slippi,
	triggers,
};

const fn ver(major: u8, minor: u8) -> slippi::Version {
	slippi::Version(major, minor, 0)
}

frame_data!(Buttons {
	logical: buttons::Logical,
	physical: buttons::Physical,
}, { });

frame_data!(Triggers {
	logical: triggers::Logical,
	physical: triggers::Physical,
}, { });

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

frame_data!(Start {
	random_seed: u32,
}, { });

frame_data!(End {
}, {
	latest_finalized_frame: i32: ver(3, 7),
});

frame_data!(Velocities {
	autogenous: Velocity,
	knockback: Velocity,
}, { });

frame_data!(Pre {
	position: Position,
	direction: Direction,
	joystick: Position,
	cstick: Position,
	triggers: Triggers,
	random_seed: u32,
	buttons: Buttons,
	state: action_state::State,
}, {
	raw_analog_x: u8: ver(1, 2),
	damage: f32: ver(1, 4),
});

frame_data!(Post {
	position: Position,
	direction: Direction,
	damage: f32,
	shield: f32,
	state: action_state::State,
	character: character::Internal,
	last_attack_landed: Option<attack::Attack>,
	combo_count: u8,
	last_hit_by: Option<Port>,
	stocks: u8,
}, {
	state_age: f32: ver(0, 2),
	flags: StateFlags: ver(2, 0),
	misc_as: f32: ver(2, 0),
	airborne: bool: ver(2, 0),
	ground: u16: ver(2, 0),
	jumps: u8: ver(2, 0),
	l_cancel: Option<bool>: ver(2, 0),
	hurtbox_state: HurtboxState: ver(2, 1),
	velocities: Velocities: ver(3, 5),
	hitlag: f32: ver(3, 8),
});

frame_data!(Data {
	pre: Pre,
	post: Post,
}, { });

#[derive(Debug, PartialEq, Serialize)]
pub struct PortData {
	pub leader: Data,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub follower: Option<Box<Data>>,
}

frame_data!(Item {
	id: u32,
	r#type: item::Item,
	state: u8,
	direction: Direction,
	position: Position,
	velocity: Velocity,
	damage: u16,
	timer: f32,
}, {
	misc: [u8; 4]: ver(3, 2),
	owner: Option<Port>: ver(3, 5),
});

#[derive(Debug, PartialEq)]
pub struct Frame<const N: usize> {
	pub ports: [PortData; N],
	pub start: Option<Start>,
	pub end: Option<End>,
	pub items: Option<Vec<Item>>,
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
