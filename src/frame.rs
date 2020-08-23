use std::fmt;

use serde::Serialize;
use serde::ser::SerializeStruct;

use super::{action_state, attack, buttons, character, game, triggers};

pseudo_enum!(Direction: i8 {
	-1 => LEFT,
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

query_impl!(Position);

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Buttons {
	pub logical: buttons::Logical,
	pub physical: buttons::Physical,
}

query_impl!(Buttons, self, f, config, query {
	match &*query[0] {
		"logical" => self.logical.query(f, config, &query[1..]),
		"physical" => self.physical.query(f, config, &query[1..]),
		s => Err(err!("unknown field `buttons.{}`", s)),
	}
});

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Triggers {
	pub logical: triggers::Logical,
	pub physical: triggers::Physical,
}

query_impl!(Triggers);

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

query_impl!(Start, self, f, config, query {
	match &*query[0] {
		"random_seed" => self.random_seed.query(f, config, &query[1..]),
		s => Err(err!("unknown field `start.{}`", s)),
	}
});

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct EndV3_7 {
	pub latest_finalized_frame: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct End {
	#[cfg(v3_7)] #[serde(flatten)]
	pub v3_7: EndV3_7,

	#[cfg(not(v3_7))] #[serde(flatten)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v3_7: Option<EndV3_7>,
}

query_impl!(EndV3_7, self, f, config, query {
	match &*query[0] {
		"latest_finalized_frame" => self.latest_finalized_frame.query(f, config, &query[1..]),
		s => Err(err!("unknown field `end.{}`", s)),
	}
});

query_impl!(End, self, f, config, query {
	match &*query[0] {
		"v3_7" => self.v3_7.query(f, config, &query[1..]),
		_ => self.v3_7.query(f, config, query),
	}
});

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
	#[serde(skip_serializing_if = "Option::is_none")]
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
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v1_2: Option<PreV1_2>,
}

query_impl!(Pre, self, f, config, query {
	match &*query[0] {
		"position" => self.position.query(f, config, &query[1..]),
		"direction" => self.direction.query(f, config, &query[1..]),
		"joystick" => self.joystick.query(f, config, &query[1..]),
		"cstick" => self.cstick.query(f, config, &query[1..]),
		"triggers" => self.triggers.query(f, config, &query[1..]),
		"random_seed" => self.random_seed.query(f, config, &query[1..]),
		"buttons" => self.buttons.query(f, config, &query[1..]),
		"state" => self.state.query(f, config, &query[1..]),
		"v1_2" => self.v1_2.query(f, config, &query[1..]),
		_ => self.v1_2.query(f, config, query),
	}
});

query_impl!(PreV1_2, self, f, config, query {
	match &*query[0] {
		"raw_analog_x" => self.raw_analog_x.query(f, config, &query[1..]),
		"v1_4" => self.v1_4.query(f, config, &query[1..]),
		_ => self.v1_4.query(f, config, query),
	}
});

query_impl!(PreV1_4, self, f, config, query {
	match &*query[0] {
		"damage" => self.damage.query(f, config, &query[1..]),
		s => Err(err!("unknown field `pre.{}`", s)),
	}
});

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PostV2_1 {
	pub hurtbox_state: HurtboxState,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PostV2_0 {
	pub flags: StateFlags,
	pub misc_as: f32,
	pub ground: u16,
	pub jumps: u8,
	pub l_cancel: Option<bool>,
	pub airborne: bool,

	#[cfg(v2_1)] #[serde(flatten)]
	pub v2_1: PostV2_1,

	#[cfg(not(v2_1))] #[serde(flatten)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v2_1: Option<PostV2_1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PostV0_2 {
	pub state_age: f32,

	#[cfg(v2_0)] #[serde(flatten)]
	pub v2_0: PostV2_0,

	#[cfg(not(v2_0))] #[serde(flatten)]
	#[serde(skip_serializing_if = "Option::is_none")]
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
	#[serde(skip_serializing_if = "Option::is_none")]
	pub v0_2: Option<PostV0_2>,
}

query_impl!(Post, self, f, config, query {
	match &*query[0] {
		"position" => self.position.query(f, config, &query[1..]),
		"direction" => self.direction.query(f, config, &query[1..]),
		"damage" => self.damage.query(f, config, &query[1..]),
		"shield" => self.shield.query(f, config, &query[1..]),
		"state" => self.state.query(f, config, &query[1..]),
		"character" => self.character.query(f, config, &query[1..]),
		"last_attack_landed" => self.last_attack_landed.query(f, config, &query[1..]),
		"combo_count" => self.combo_count.query(f, config, &query[1..]),
		"last_hit_by" => self.last_hit_by.query(f, config, &query[1..]),
		"stocks" => self.stocks.query(f, config, &query[1..]),
		"v0_2" => self.v0_2.query(f, config, &query[1..]),
		_ => self.v0_2.query(f, config, query),
	}
});

query_impl!(PostV0_2, self, f, config, query {
	match &*query[0] {
		"state_age" => self.state_age.query(f, config, &query[1..]),
		"v2_0" => self.v2_0.query(f, config, &query[1..]),
		_ => self.v2_0.query(f, config, query),
	}
});

query_impl!(PostV2_0, self, f, config, query {
	match &*query[0] {
		"flags" => self.flags.query(f, config, &query[1..]),
		"misc_as" => self.misc_as.query(f, config, &query[1..]),
		"ground" => self.ground.query(f, config, &query[1..]),
		"jumps" => self.jumps.query(f, config, &query[1..]),
		"l_cancel" => self.l_cancel.query(f, config, &query[1..]),
		"airborne" => self.airborne.query(f, config, &query[1..]),
		"v2_1" => self.v2_1.query(f, config, &query[1..]),
		_ => self.v2_1.query(f, config, query),
	}
});

query_impl!(PostV2_1, self, f, config, query {
	match &*query[0] {
		"hurtbox_state" => self.hurtbox_state.query(f, config, &query[1..]),
		s => Err(err!("unknown field `post.{}`", s)),
	}
});

#[derive(Debug, PartialEq, Serialize)]
pub struct Data {
	pub pre: Pre,
	pub post: Post,
}

query_impl!(Data, self, f, config, query {
	match &*query[0] {
		"pre" => self.pre.query(f, config, &query[1..]),
		"post" => self.post.query(f, config, &query[1..]),
		s => Err(err!("unknown field `leader.{}`", s)),
	}
});

#[derive(Debug, PartialEq, Serialize)]
pub struct Port {
	pub leader: Data,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub follower: Option<Box<Data>>,
}

query_impl!(Port, self, f, config, query {
	match &*query[0] {
		"leader" => self.leader.query(f, config, &query[1..]),
		"follower" => self.follower.query(f, config, &query[1..]),
		s => Err(err!("unknown field `port.{}`", s)),
	}
});

#[derive(Debug, PartialEq)]
pub struct Frame<const N: usize> {
	#[cfg(v2_2)]
	pub start: Start,
	#[cfg(not(v2_2))]
	pub start: Option<Start>,

	#[cfg(v3_0)]
	pub end: End,
	#[cfg(not(v3_0))]
	pub end: Option<End>,

	pub ports: [Port; N],
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

		state.end()
	}
}

query_impl!(N: usize, Frame<N>, self, f, config, query {
	match &*query[0] {
		"start" => self.start.query(f, config, &query[1..]),
		"end" => self.end.query(f, config, &query[1..]),
		"ports" => self.ports.query(f, config, &query[1..]),
		s => Err(err!("unknown field `frame.{}`", s)),
	}
});

pub type Frame1 = Frame<1>;
pub type Frame2 = Frame<2>;
pub type Frame3 = Frame<3>;
pub type Frame4 = Frame<4>;
