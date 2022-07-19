use crate::model::game::{Start, Frames};

pub trait StatComputer {
	type Stat;
	type Error;
	
	fn setup(&mut self, start: &Start) -> ();
	fn process(&mut self, frames: &Frames) -> ();
	fn fetch(&self) -> Result<Self::Stat, Self::Error>;
}

pub enum StatError {
	Unintialized(),
	WrongPlayerCount(u8),
}
