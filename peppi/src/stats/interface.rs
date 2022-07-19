use crate::model::game::{Start, Frames};

pub trait StatComputer {
	type Stat;

	fn new(start: &Start) -> Self;
	fn process(&mut self, frames: &Frames) -> ();
	fn into_inner(self) -> Self::Stat;
}
