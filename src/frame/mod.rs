//! Frame data representation.
//!
//! Peppi represents frame data using Arrow arrays (i.e. "struct-of-arrays").
//! This allows us to efficiently share frame data with other languages,
//! and enables simple serialization into a highly-compressible disk format.
//!
//! The mutable/immutable distinction is essentially an artifact of the underlying Arrow library.
//! You'll only encounter mutable data if you're parsing live games.

use crate::game::Port;

pub mod immutable;
pub mod mutable;
pub mod transpose;

/// Frame indexes start at -123, and reach 0 at "Go!".
pub const FIRST_INDEX: i32 = -123;

/// Port number plus ICs-specific discriminant.
#[derive(Clone, Copy, Debug)]
pub struct PortOccupancy {
	pub port: Port,
	/// For ICs, distinguishes between Nana and Popo.
	pub follower: bool,
}

/// Rollback-aware processing typically ignores all but the first or last rollback for a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rollbacks {
	ExceptFirst,
	ExceptLast,
}
