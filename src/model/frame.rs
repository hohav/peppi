use crate::model::game::Port;

pub mod immutable;
pub mod mutable;
pub mod transpose;

/// Frame indexes start at -123, and reach 0 at "Go!".
pub const FIRST_INDEX: i32 = -123;

#[derive(Clone, Copy, Debug)]
pub struct PortOccupancy {
	pub port: Port,
	pub follower: bool,
}
