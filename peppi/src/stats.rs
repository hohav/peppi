pub mod actions;

use crate::model::game::{Start, Frames};
use crate::model::slippi::Version;

pub trait StatComputer {
	type Stat;
	const MIN_VERSION: Version = Version(0, 1, 0);
	fn new(start: &Start) -> Self;
	fn process(&mut self, frames: &Frames);
	fn into_inner(self) -> Self::Stat;
}
