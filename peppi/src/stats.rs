pub mod actions;

use crate::model::{
	frame::Frame,
	game::{Game, Frames, Start},
};

use crate::model::slippi::Version;

#[derive(Clone, Copy, Debug)]
pub enum StatError {
	MalformedInput,
	OldVersion(Version, Version),
}

type StatResult<T> = Result<T, StatError>;

pub trait Computer {
	type Stat;
	const MIN_VERSION: Version = Version(0, 1, 0);
	fn check_version(ver: Version) -> StatResult<()> {
		if ver < Self::MIN_VERSION {
			Err(StatError::OldVersion(ver, Self::MIN_VERSION))
		} else {
			Ok(())
		}
	}
	fn compute_game(game: &Game) -> StatResult<Self::Stat>;
}

pub trait PartialComputer<'a>: Computer + Sized {
	type PartialStat;
	fn new(start: &Start) -> StatResult<Self>;
	fn process_frame<const N: usize>(&mut self, _: &Frame<N>) -> StatResult<()> { Ok(()) }

	fn peek_inner(&'a self) -> Self::PartialStat;
	fn into_inner(self) -> Self::Stat;

	fn compute_game(game: &Game) -> StatResult<Self::Stat> {
		let mut comp: Self = Self::new(&game.start)?;

		match &game.frames {
			Frames::P1(fs) => for f in fs { comp.process_frame(f)?; },
			Frames::P2(fs) => for f in fs { comp.process_frame(f)?; },
			Frames::P3(fs) => for f in fs { comp.process_frame(f)?; },
			Frames::P4(fs) => for f in fs { comp.process_frame(f)?; },
		}

		Ok(comp.into_inner())
	}
}
