use std::error::{Error};
use std::collections::{HashMap};
use std::convert::{TryFrom};

use log::{warn};
use chrono::{DateTime, Utc};

use super::character;
use super::game::{NUM_PORTS, FIRST_FRAME_INDEX};
use super::ubjson::{Object};

#[derive(Debug, PartialEq, Eq)]
pub struct Metadata {
	pub json: HashMap<String, Object>,
	pub date: Option<DateTime<Utc>>,
	pub duration: Option<u32>,
	pub platform: Option<String>,
	pub players: Option<[Option<MetadataPlayer>; NUM_PORTS]>,
	pub console_name: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MetadataPlayer {
	pub characters: Option<HashMap<character::Internal, u32>>,
	pub netplay_name: Option<String>,
}

fn date(json:&HashMap<String, Object>) -> Option<DateTime<Utc>> {
	let date_too_short = "2000-01-01T00:00:00".parse::<DateTime<Utc>>();
	match json.get("startAt") {
		None => None,
		Some(Object::Str(start_at)) => match start_at.parse::<DateTime<Utc>>() {
			Ok(start_at) => Some(start_at),
			e if e == date_too_short => {
				match format!("{}Z", start_at).parse::<DateTime<Utc>>() {
					Ok(start_at) => Some(start_at),
					Err(e) => {
						warn!("metadata.startAt: parse error: {:?}, {:?}", e, start_at);
						None
					},
				}
			},
			Err(e) => {
				warn!("metadata.startAt: parse error: {:?}, {:?}", e, start_at);
				None
			},
		},
		start_at => {
			warn!("metadata.startAt: expected str, but got: {:?}", start_at);
			None
		},
	}
}

fn duration(json:&HashMap<String, Object>) -> Option<u32> {
	match json.get("lastFrame") {
		None => None,
		Some(Object::Int(last_frame)) => match u32::try_from(*last_frame - FIRST_FRAME_INDEX as i64 + 1) {
			Ok(duration) => Some(duration),
			Err(e) => {
				warn!("metadata.lastFrame: value out of range: {:?}, {:?}", last_frame, e);
				None
			},
		},
		last_frame => {
			warn!("metadata.lastFrame: expected int, but got: {:?}", last_frame);
			None
		},
	}
}

fn platform(json:&HashMap<String, Object>) -> Option<String> {
	match json.get("playedOn") {
		None => None,
		Some(Object::Str(played_on)) => Some(played_on.clone()),
		played_on => {
			warn!("metadata.playedOn: expected str, but got: {:?}", played_on);
			None
		},
	}
}

fn parse_characters(characters:&HashMap<String, Object>) -> Result<HashMap<character::Internal, u32>, Box<dyn Error>> {
	characters.iter().map(|(k, v)|
		match v {
			Object::Int(v) => Ok((
				character::Internal(k.parse::<u8>().map_err(|e| Box::new(e))?),
				u32::try_from(*v).map_err(|e| Box::new(e))?,
			)),
			v => Err(err!("metadata.players.N.characters.{}: expected int, but got: {:?}", k, v).into()),
		}
	).collect::<Result<HashMap<character::Internal, u32>, Box<dyn Error>>>()
}

fn metadata_player(player:&HashMap<String, Object>) -> Result<MetadataPlayer, Box<dyn Error>> {
	Ok(MetadataPlayer {
		characters: match player.get("characters") {
			Some(Object::Map(characters)) => match parse_characters(&characters) {
				Ok(characters) => Some(characters),
				Err(e) => {
					warn!("metadata.players.N.characters: parse error: {:?}, {:?}", e, characters);
					None
				},
			},
			characters => {
				warn!("metadata.players.N.characters: expected map, but got: {:?}", characters);
				None
			},
		},
		netplay_name: match player.get("names") {
			None => None,
			Some(Object::Map(names)) => match names.get("netplay") {
				None => None,
				Some(Object::Str(netplay)) => Some(netplay.clone()),
				netplay => {
					warn!("metadata.players.N.names.netplay: expected str, but got: {:?}", netplay);
					None
				},
			},
			names => {
				warn!("metadata.players.N.names: expected map, but got: {:?}", names);
				None
			},
		},
	})
}

fn players(json:&HashMap<String, Object>) -> Option<[Option<MetadataPlayer>; NUM_PORTS]> {
	match json.get("players") {
		None => None,
		Some(Object::Map(players)) => {
			let mut result:[Option<MetadataPlayer>; NUM_PORTS] = [None, None, None, None];
			for (port, player) in players {
				match port.parse::<usize>() {
					Ok(port) if port < NUM_PORTS => {
						match player {
							Object::Map(player) => {
								match metadata_player(player) {
									Ok(player) => result[port as usize] = Some(player),
									Err(e) => warn!("metadata.players.N: parse error: {:?}, {:?}", e, player),
								};
							},
							player => warn!("metadata.players.N: expected map, but got: {:?}", player),
						}
					},
					Ok(port) => warn!("metadata.players: port number out of valid range: {}", port),
					Err(e) => warn!("metadata.players: non-numeric port: {}, {:?}", port, e),
				};
			}
			Some(result)
		},
		players => {
			warn!("metadata.players: expected map, but got: {:?}", players);
			None
		}
	}
}

fn console_name(json:&HashMap<String, Object>) -> Option<String> {
	match json.get("consoleNick") {
		None => None,
		Some(Object::Str(console_nick)) => Some(console_nick.clone()),
		console_nick => {
			warn!("metadata.consoleNick: expected str, but got: {:?}", console_nick);
			None
		},
	}
}

pub fn parse(json:&HashMap<String, Object>) -> Metadata {
	Metadata {
		json: json.clone(),
		date: date(json),
		duration: duration(json),
		platform: platform(json),
		players: players(json),
		console_name: console_name(json),
	}
}
