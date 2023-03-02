use serde::{ser::SerializeStruct, Serialize};

use crate::model::{
	buttons,
	enums::{action_state, attack, character, ground},
	item,
	primitives::{Direction, Port, Position, Velocity},
	triggers,
};
use peppi_derive::Arrow;

/// Controller button state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Arrow)]
pub struct Buttons {
	/// Bitfield representing all currently active buttons and analog inputs that the game engine uses to calculate the next frame
	pub logical: buttons::Logical,
	/// Bitfield representing all currently active buttons read by the console
	pub physical: buttons::Physical,
}

/// Controller trigger state.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct Triggers {
	/// The analog trigger value the game engine uses to calculate the next frame
	pub logical: triggers::Logical,
	/// The analog trigger values of each trigger read by the console
	pub physical: triggers::Physical,
}

pseudo_bitmask!(StateFlags: u64 {
	1u64 << 04 => REFLECT_BUBBLE, // any reflect bubble is active
	1u64 << 10 => STATE_INVULN, // used for state-based intang/invinc that is removed upon state change
	1u64 << 11 => FAST_FALLING,
	1u64 << 13 => HIT_LAG,
	1u64 << 23 => SHIELDING,
	1u64 << 25 => HIT_STUN,
	1u64 << 26 => SHIELD_TOUCH, // relevant to player touching other player's shield, but only activates under rare obscure circumstances
	1u64 << 29 => POWER_SHIELD_BUBBLE, // powershield bubble is active
	1u64 << 35 => FOLLOWER, // Nana
	1u64 << 36 => INACTIVE, // shiek/zelda when other is in play, teammate with no stocks, etc. Should never appear in replays
	1u64 << 38 => DEAD,
	1u64 << 39 => OFF_SCREEN,
});

pseudo_enum!(HurtboxState: u8 {
	0 => VULNERABLE,
	1 => INVULNERABLE,
	2 => INTANGIBLE,
});

/// Start-of-frame data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Arrow)]
pub struct Start {
	pub random_seed: u32,

	/// Scene frame counter. Starts at 0 when game starts. Continues to count frames
	/// even if the game is paused.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "3.10")]
	pub scene_frame_counter: Option<u32>,
}

/// End-of-frame data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Arrow)]
pub struct End {
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "3.7")]
	pub latest_finalized_frame: Option<i32>,
}

/// The game tracks two different "velocities" per character, autogenous (self-induced)
/// and knockback. These are added to obtain an effective velocity, which may be further
/// modified by other factors like obstacles.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct Velocities {
	/// self-induced velocity
	pub self_induced: Velocity,

	/// knockback-induced velocity
	pub knockback: Velocity,

	/// For ergonomics we merge air+ground autogenous velocities into `.autogenous`, based on
	/// the character's `airborne` state. But we also keep the original values for round-tripping.
	#[serde(skip)]
	#[doc(hidden)]
	pub self_induced_x: SelfXVelocity,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct SelfXVelocity {
	pub air: f32,
	pub ground: f32,
}

/// Pre-frame update data, required to reconstruct a replay.
///
/// Collected right before controller inputs are processed.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct Pre {
	pub position: Position,

	pub facing_direction: Direction,

	pub joystick: Position,

	pub cstick: Position,

	pub triggers: Triggers,

	pub random_seed: u32,

	pub buttons: Buttons,

	pub state: action_state::State,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "1.2")]
	pub raw_analog_x: Option<i8>,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "1.4")]
	pub percent: Option<f32>,
}

/// Post-frame update data, for computing stats etc.
///
/// Collected at the end of collision detection, the last consideration of the game engine.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct Post {
	/// in-game character (can only change for Zelda/Sheik)
	pub character: character::Internal,

	/// action state (very useful for stats)
	pub state: action_state::State,

	pub position: Position,

	pub facing_direction: Direction,

	pub percent: f32,

	pub shield_health: f32,

	pub last_attack_landed: Option<attack::Attack>,

	pub combo_count: u8,

	pub last_hit_by: Option<Port>,

	/// stocks remaining
	pub stocks_remaining: u8,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "0.2")]
	pub state_age: Option<f32>,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "2.0")]
	pub flags: Option<StateFlags>,

	/// used for multiple things, including hitstun frames remaining
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "2.0")]
	pub misc_as: Option<f32>,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "2.0")]
	pub is_airborne: Option<bool>,

	/// The ground the character was standing on most recently
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "2.0")]
	pub last_ground_id: Option<ground::Ground>,

	/// jumps remaining
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "2.0")]
	pub jumps_remaining: Option<u8>,

	/// true = successful L-Cancel
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "2.0")]
	pub l_cancel: Option<Option<bool>>,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "2.1")]
	pub hurtbox_state: Option<HurtboxState>,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "3.5")]
	pub velocities: Option<Velocities>,

	/// hitlag remaining
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "3.8")]
	pub hitlag_remaining: Option<f32>,

	/// animation the character is in (for Wait: 2 = Wait1, 3 = Wait2, 4 = Wait3)
	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "3.11")]
	pub animation_index: Option<u32>,
}

/// Frame data for a single character. Includes both pre-frame and post-frame data.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct Data {
	pub pre: Pre,
	pub post: Post,
}

/// Frame data for a single port/player. Can include two characters’ frame data (ICs).
#[derive(Clone, Debug, PartialEq, Serialize, Arrow)]
pub struct PortData {
	/// Frame data for the controlled character.
	pub leader: Data,

	/// Frame data for the follower, if any (Nana).
	// Boxing reduces memory usage greatly for most characters,
	// at the expense of pointer dereferences for Nana.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub follower: Option<Box<Data>>,
}

/// A single frame of the game. `N` is the number of players in the game.
// Const generics allow our memory layout to depend on the number of players,
// so that a 2-player game takes up half the memory of a 4-player game.
// This is better for memory locality than using pointers.
#[derive(Clone, Debug, PartialEq, Arrow)]
pub struct Frame<const N: usize> {
	/// Frame index (starts at `peppi::game::FIRST_FRAME_INDEX`).
	///
	/// Indexes should never skip values, but may decrease if rollbacks
	/// are enabled (see `peppi::serde::collect::Opts`).
	pub index: i32,

	/// Frame data for each port. The player with the lowest port is always at index 0.
	pub ports: [PortData; N],

	#[slippi(version = "2.2")]
	pub start: Option<Start>,

	#[slippi(version = "3.0")]
	pub end: Option<End>,

	#[slippi(version = "3.0")]
	pub items: Option<Vec<item::Item>>,
}

// workaround for Serde not supporting const generics
impl<const N: usize> Serialize for Frame<N> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut state = serializer.serialize_struct("Frame", 1)?;

		state.serialize_field("index", &self.index)?;

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
