use std::path;

use clap::{App, Arg};

extern crate pretty_env_logger;
use log::{error};

fn inspect(paths:&[&str], as_json:bool) -> Result<(), String> {
	for path in paths {
		let path = path::Path::new(path);
		let game = slippi::game(path).map_err(|e| format!("{:?}", e))?;
		match as_json {
			true => println!("{}", serde_json::to_string(&game).map_err(|e| format!("{:?}", e))?),
			_ => println!("{:#?}", game),
		}
	}
	Ok(())
}

fn main() {
	pretty_env_logger::init();

    let matches = App::new("Peppi")
		.version("0.1")
		.author("melkor <hohav@fastmail.com>")
		.about("Inspector for Slippi SSBM replay files")
		.arg(Arg::with_name("json")
			.help("Output as JSON")
			.short("j")
			.long("json"))
		.arg(Arg::with_name("frames")
			.help("Output frame data")
			.short("f")
			.long("frames"))
		.arg(Arg::with_name("FILE")
			.help("Replay file to parse")
			.required(true)
			.index(1))
		.get_matches();

	let path = matches.value_of("FILE").unwrap().to_string();
	let as_json = matches.is_present("json");

	unsafe {
		slippi::game::SERIALIZE_FRAMES = matches.is_present("frames");
	}

	if let Err(e) = inspect(&[&path], as_json) {
		error!("{}", e);
	}
}
