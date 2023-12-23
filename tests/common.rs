use std::{
	fs::File,
	io::BufReader,
	path::{Path, PathBuf},
};

use peppi::{
	game::immutable::Game,
	io::{slippi, PosError},
};

pub fn read_game(path: impl AsRef<Path>, skip_frames: bool) -> Result<Game, PosError> {
	let mut buf = BufReader::new(File::open(path).unwrap());
	slippi::read(
		&mut buf,
		Some(&slippi::de::Opts {
			skip_frames: skip_frames,
			..Default::default()
		}),
	)
}

pub fn get_path(name: &str) -> PathBuf {
	format!("tests/data/{}.slp", name).into()
}

pub fn game(name: &str) -> Game {
	read_game(get_path(name), false).unwrap()
}
