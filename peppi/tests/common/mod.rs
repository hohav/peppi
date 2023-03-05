use peppi::model::game::Game;
use peppi::serde::de;
use peppi::ParseError;

use std::{
	fs::File,
	io::BufReader,
	path::{Path, PathBuf},
};

pub fn read_game(path: impl AsRef<Path>, skip_frames: bool) -> Result<Game, ParseError> {
	let mut buf = BufReader::new(File::open(path).unwrap());
	let opts = de::Opts {
		skip_frames,
		..Default::default()
	};
	peppi::game(&mut buf, Some(&opts), None)
}

pub fn get_path(name: &str) -> PathBuf {
	format!("tests/data/{}.slp", name).into()
}

pub fn game(name: &str) -> Game {
	read_game(get_path(name), false).unwrap()
}
