use std::env;
use std::path::Path;

extern crate pretty_env_logger;
use log::{error};

fn main() {
	pretty_env_logger::init();

	let args: Vec<String> = env::args().collect();

	if args.len() < 2 {
		println!("usage: {} REPLAY.slp", Path::new(&args[0]).file_name().and_then(|s| s.to_str()).unwrap_or(""));
	} else {
		let path = Path::new(&args[1]);
		match slippi::parse(&path) {
			Ok(game) => println!("{:#?}", game),
			Err(err) => error!("{}", err),
		}
	}
}
