use std::{
	fs::File,
	io::BufReader,
	path::{Path, PathBuf},
};

use peppi::{
	io::{
		slippi::{
			de::{Debug, Opts},
			read,
		},
		PosError,
	},
	model::game::immutable::Game,
};

pub fn read_game(path: impl AsRef<Path>) -> Result<Game, PosError> {
	let mut buf = BufReader::new(File::open(path).unwrap());
	read(
		&mut buf,
		Some(&Opts {
			skip_frames: false,
			debug: Some(Debug {
				dir: PathBuf::from("debug"),
			}),
		}),
	)
}

pub fn get_path(name: &str) -> PathBuf {
	format!("tests/data/{}.slp", name).into()
}

pub fn game(name: &str) -> Game {
	read_game(get_path(name)).unwrap()
}
