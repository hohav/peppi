use std::{
	fs::File,
	io::BufReader,
	path::{Path, PathBuf},
};

use peppi::{
	game::immutable::Game,
	io::{slippi::read, PosError},
};

pub fn read_game(path: impl AsRef<Path>) -> Result<Game, PosError> {
	let mut buf = BufReader::new(File::open(path).unwrap());
	read(&mut buf, None)
}

pub fn get_path(name: &str) -> PathBuf {
	format!("tests/data/{}.slp", name).into()
}

pub fn game(name: &str) -> Game {
	read_game(get_path(name)).unwrap()
}
