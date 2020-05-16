use std::path;

use clap::{App, Arg};

extern crate pretty_env_logger;
use log::{error};

fn inspect(paths:&[&str], config:peppi::Config) -> Result<(), String> {
	for path in paths {
		let path = path::Path::new(path);
		let game = peppi::game(path).map_err(|e| format!("{:?}", e))?;
		if let Some(port) = config.states_port {
			for f in &game.ports[port].as_ref().ok_or(format!("no player at port {}", port))?.leader.post {
				println!("{:?}", f.state);
			}
		} else if config.json {
			println!("{}", serde_json::to_string(&game).map_err(|e| format!("{:?}", e))?);
		} else {
			println!("{:#?}", game);
		}
	}
	Ok(())
}

fn validate_port(s:String) -> Result<(), String> {
	match s.as_str() {
		"0" | "1" | "2" | "3" => Ok(()),
		_ => Err("invalid port".to_string()),
	}
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
		.arg(Arg::with_name("PORT")
			.help("Print just the post-frame action states for PORT (0..3)")
			.short("s")
			.long("states")
			.takes_value(true)
			.validator(validate_port))
		.arg(Arg::with_name("names")
			.help("Append names for known constants")
			.short("n")
			.long("names"))
		.arg(Arg::with_name("FILE")
			.help("Replay file to parse")
			.required(true)
			.index(1))
		.get_matches();

	let path = matches.value_of("FILE").unwrap().to_string();

	let config = peppi::Config {
		json: matches.is_present("json"),
		frames: matches.is_present("frames"),
		enum_names: matches.is_present("names"),
		states_port: matches.value_of("PORT").map(|s| s.parse::<usize>().unwrap()),
	};

	unsafe { peppi::CONFIG = config };

	if let Err(e) = inspect(&[&path], config) {
		error!("{}", e);
	}
}
