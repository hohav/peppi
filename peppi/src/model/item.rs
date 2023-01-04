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

	pub direction: Direction,

	pub position: Position,

	pub velocity: Velocity,

	pub damage: u16,

	pub timer: f32,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "3.2")]
	pub misc: Option<[u8; 4]>,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[slippi(version = "3.5")]
	pub owner: Option<Option<Port>>,
}
