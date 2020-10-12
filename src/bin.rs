use std::path;

use clap::{App, Arg};
use jmespatch::ToJmespath;
use log::{error};

fn inspect(paths: &[&str], config: &peppi::Config) -> Result<(), String> {
	for path in paths {
		let path = path::Path::new(path);
		let game = peppi::game(path).map_err(|e| format!("{:?}", e))?;
		if let Some(query) = &config.query {
			let query = jmespatch::compile(query).map_err(|e| format!("{:?}", e))?;
			let jmes = game.to_jmespath().map_err(|e| format!("{:?}", e))?;
			let result = query.search(jmes).map_err(|e| format!("{:?}", e))?;
			println!("{}", serde_json::to_string(&result).map_err(|e| format!("{:?}", e))?);
		} else if config.json {
			println!("{}", serde_json::to_string(&game).map_err(|e| format!("{:?}", e))?);
		} else {
			println!("{:#?}", game);
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
		.arg(Arg::with_name("QUERY")
			.help("Print a subset of parsed data (JMESPath syntax)")
			.short("q")
			.long("query")
			.takes_value(true))
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
		frames: matches.is_present("frames") || matches.is_present("QUERY"),
		enum_names: matches.is_present("names"),
		query: matches.value_of("QUERY").map(|q| q.to_string()),
	};

	unsafe { peppi::CONFIG = config.clone() };

	if let Err(e) = inspect(&[&path], &config) {
		error!("{}", e);
	}
}
