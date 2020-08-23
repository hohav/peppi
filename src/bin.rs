use std::path;

use clap::{App, Arg};
use log::{error};

use peppi::query::Query;

fn parse_query(query:&str) -> Vec<String> {
	let re = regex::Regex::new(r"\[(-?\d*)\]$").unwrap();
	query.split(".").flat_map(|s|
		match re.captures(s) {
			Some(c) => c.get(1).map_or(vec![], |g|
				vec![s[.. g.start()-1].to_string(), s[g.start() .. g.end()].to_string()]
			),
			_ => vec![s.to_string()],
		}
	).collect()
}

fn inspect(paths:&[&str], config:&peppi::Config) -> Result<(), String> {
	for path in paths {
		let path = path::Path::new(path);
		let game = peppi::game(path).map_err(|e| format!("{:?}", e))?;
		if let Some(query) = &config.query {
			let query:&[&str] = &match &*query[0] {
				"" => &query[1..],
				_ => &query[..],
			}.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

			game.query(&mut std::io::stdout(), config, query).map_err(|e| format!("{:?}", e))?;

			// Single-value queries need this newline, but we'll get a double newline for array queries.
			// TODO: handle newlines more smarter, esp. for queries with multiple `[]`.
			println!("");
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
			.help("Print a subset of parsed data (jq-like)")
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
		frames: matches.is_present("frames"),
		enum_names: matches.is_present("names"),
		query: matches.value_of("QUERY").map(parse_query),
	};

	unsafe { peppi::CONFIG = config.clone() };

	if let Err(e) = inspect(&[&path], &config) {
		error!("{}", e);
	}
}
