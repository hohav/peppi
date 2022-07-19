use crate::stats::interface::{StatComputer, StatError};
use crate::model::game::{Start, Frames};

pub struct ActionComputer {
	pub actions: Vec<ActionStat>,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, )]
pub struct ActionStat {
	jab1: u16,
	jab2: u16,
	jab3: u16,
	jabm: u16,
	dash: u16,
	ftilt: u16,
	utilt: u16,
	dtilt: u16,
	fsmash: u16,
	usmash: u16,
	dsmash: u16,
	nair: u16,
	fair: u16,
	bair: u16,
	uair: u16,
	dair: u16,
}

impl StatComputer for ActionComputer {
	type Stat = ActionStat;
	type Error = StatError;

	fn setup(&mut self, start: &Start) -> () {
		self.actions = Vec::new();
		self.actions.resize_with(start.players.len(), Default::default);
	}

	fn process(&mut self, frames: &Frames) -> () {
		match frames {
			Frames::P1(_fs) => {

			},
			Frames::P2(_fs) => {

			},
			Frames::P3(_fs) => {

			},
			Frames::P4(_fs) => {

			},
		}
	}

	fn fetch(&self) -> Result<Self::Stat, Self::Error> {
		Err(StatError::Unintialized())
	}
}
