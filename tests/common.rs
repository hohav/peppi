use std::{
	fs::File,
	io::BufReader,
	path::{Path, PathBuf},
};

use peppi::{
	frame::{Frames, Reader},
	game::Game,
	io::{Result, slippi},
};

pub fn read_game<F: Frames+Reader>(path: impl AsRef<Path>, skip_frames: bool) -> Result<Game<F>> {
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

pub fn game<F: Frames+Reader>(name: &str) -> Game<F> {
	read_game(get_path(name), false).unwrap()
}
