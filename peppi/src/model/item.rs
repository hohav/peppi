use serde::Serialize;

use crate::model::{
	enums::item::{State, Type},
	primitives::{Direction, Port, Position, Velocity},
};
use peppi_derive::Arrow;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Arrow)]
pub struct Item {
	pub id: u32,

	pub r#type: Type,

	pub state: State,

	pub facing_direction: Option<Direction>,

	pub position: Position,

	pub velocity: Velocity,

	pub damage_taken: u16,

	pub expiration_timer: f32,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "3.2")]
	pub misc: Option<[u8; 4]>,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "3.5")]
	pub owner: Option<Option<Port>>,
}
