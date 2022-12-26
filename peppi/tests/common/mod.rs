use peppi::model::game::Game;

use std::{fs::File, io::BufReader, path::Path};

pub fn read_game(path: impl AsRef<Path>) -> Game {
	let mut buf = BufReader::new(File::open(path).unwrap());
	peppi::game(&mut buf, None, None).unwrap()
}

pub fn game(name: &str) -> Game {
	read_game(&format!("tests/data/{}.slp", name))
}
