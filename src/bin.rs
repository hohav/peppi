use std::env;
use std::path;

extern crate pretty_env_logger;
use log::{error};

fn main() {
	pretty_env_logger::init();

	let args: Vec<String> = env::args().collect();

	for path in &args[1..] {
		let path = path::Path::new(&path);
		match slippi::game(&path) {
			Ok(game) => println!("{:#?}", game),
			Err(err) => error!("{}", err),
		}
	}
}
