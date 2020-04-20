use std::fmt;

use super::action_state::{ActionState};
use super::attack::{Attack};
use super::character::{Character};

use super::pseudo_bitmask;
use super::pseudo_enum;

pseudo_enum!(LCancel:u8 {
	0 => NONE,
	1 => SUCCESSFUL,
	2 => UNSUCCESSFUL,
});

pseudo_enum!(Direction:u8 {
	0 => LEFT,
	1 => RIGHT,
});

#[derive(Debug)]
pub struct Position {
	pub x: f32,
	pub y: f32,
}

#[derive(Debug)]
pub struct Buttons {
	pub logical: ButtonsLogical,
	pub physical: ButtonsPhysical,
}

pseudo_bitmask!(ButtonsPhysical:u16 {
	1u16 << 12 => START,
	1u16 << 11 => Y,
	1u16 << 10 => X,
	1u16 << 09 => B,
	1u16 << 08 => A,
	1u16 << 06 => L,
	1u16 << 05 => R,
	1u16 << 04 => Z,
	1u16 << 03 => DPAD_UP,
	1u16 << 02 => DPAD_DOWN,
	1u16 << 01 => DPAD_RIGHT,
	1u16 << 00 => DPAD_LEFT,
});

pseudo_bitmask!(ButtonsLogical:u32 {
	1u32 << 31 => TRIGGER_ANALOG,
	1u32 << 23 => CSTICK_RIGHT,
	1u32 << 22 => CSTICK_LEFT,
	1u32 << 21 => CSTICK_DOWN,
	1u32 << 20 => CSTICK_UP,
	1u32 << 19 => JOYSTICK_RIGHT,
	1u32 << 18 => JOYSTICK_LEFT,
	1u32 << 17 => JOYSTICK_DOWN,
	1u32 << 16 => JOYSTICK_UP,
	1u32 << 12 => START,
	1u32 << 11 => Y,
	1u32 << 10 => X,
	1u32 << 09 => B,
	1u32 << 08 => A,
	1u32 << 06 => L,
	1u32 << 05 => R,
	1u32 << 04 => Z,
	1u32 << 03 => DPAD_UP,
	1u32 << 02 => DPAD_DOWN,
	1u32 << 01 => DPAD_RIGHT,
	1u32 << 00 => DPAD_LEFT,
});

#[derive(Debug)]
pub struct Triggers {
	pub logical: f32,
	pub physical: TriggersPhysical,
}

#[derive(Debug)]
pub struct TriggersPhysical {
	pub l: f32,
	pub r: f32,
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

#[derive(Debug)]
pub struct FramePre {
	pub index: i32,

	pub position: Position,
	pub direction: Direction,
	pub joystick: Position,
	pub cstick: Position,
	pub triggers: Triggers,
	pub random_seed: u32,
	pub buttons: Buttons,
	pub state: ActionState,

	// v1.2
	pub raw_analog_x: Option<u8>,

	// v1.4
	pub damage: Option<f32>,
}

impl Indexed for FramePre {
	fn index(&self) -> i32 {
		self.index
	}
}

#[derive(Debug)]
pub struct FramePost {
	pub index: i32,

	pub position: Position,
	pub direction: Direction,
	pub damage: f32,
	pub shield: f32,
	pub state: ActionState,
	pub character: Character,
	pub last_attack_landed: Attack,
	pub combo_count: u8,
	pub last_hit_by: u8,
	pub stocks: u8,

	// v0.2
	pub state_age: Option<f32>,

	// v2.0
	pub flags: Option<StateFlags>,
	pub misc_as: Option<f32>,
	pub ground: Option<u16>,
	pub jumps: Option<u8>,
	pub l_cancel: Option<LCancel>,
	pub airborne: Option<bool>,

	// v2.1
	pub hurtbox_state: Option<HurtboxState>,
}

impl Indexed for FramePost {
	fn index(&self) -> i32 {
		self.index
	}
}

pub struct Frames {
	pub pre: Vec<FramePre>,
	pub post: Vec<FramePost>,
}

impl fmt::Debug for Frames {
	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Frames {{ pre: [...]({}), post: [...]({}) }}", self.pre.len(), self.post.len())
	}
}
