use std::{collections::HashMap, fs, io};

use chrono::{DateTime, Utc};
use pretty_assertions::assert_eq;
use serde_json;

use peppi::{
	model::{
		buttons::{Logical, Physical},
		enums::{
			action_state::{State, Zelda},
			character::{External, Internal},
			costume::Costume,
			item,
			stage::Stage,
		},
		frame::{self, Buttons},
		game::{
			DashBack, End, EndMethod, Frames, Game, Language, Netplay, Player, PlayerType, Scene,
			ShieldDrop, Start, Ucf,
		},
		item::Item,
		metadata::{self, Metadata, Platform},
		primitives::{Direction, Port, Position, Velocity},
		slippi::{Slippi, Version},
	},
	serde::{
		self, arrow,
		collect::{self, Rollback},
	},
};

#[derive(Debug)]
struct Error(pub String);

impl From<String> for Error {
	fn from(s: String) -> Self {
		Error(s)
	}
}

impl From<&str> for Error {
	fn from(s: &str) -> Self {
		Error(s.to_string())
	}
}

impl From<serde_json::Error> for Error {
	fn from(e: serde_json::Error) -> Self {
		Error(format!("serde_json error: {:?}", e))
	}
}

impl From<peppi::ParseError> for Error {
	fn from(e: peppi::ParseError) -> Self {
		Error(format!("peppi error: {:?}", e))
	}
}

fn read_game(path: &str) -> Result<Game, Error> {
	let mut buf = io::BufReader::new(fs::File::open(path).unwrap());
	Ok(peppi::game(&mut buf, None, None)?)
}

fn game(name: &str) -> Result<Game, Error> {
	read_game(&format!("tests/data/{}.slp", name))
}

fn button_seq(game: &Game) -> Result<Vec<Buttons>, Error> {
	match &game.frames {
		Frames::P2(frames) => {
			let mut last_buttons: Option<Buttons> = None;
			let mut button_seq = Vec::<Buttons>::new();
			for frame in frames {
				let b = frame.ports[0].leader.pre.buttons;
				if (b.logical.0 > 0 || b.physical.0 > 0) && Some(b) != last_buttons {
					button_seq.push(b);
					last_buttons = Some(b);
				}
			}
			Ok(button_seq)
		}
		_ => Err("wrong number of ports".into()),
	}
}

#[test]
fn slippi_old_version() -> Result<(), Error> {
	let game = game("v0.1")?;
	let players = game.start.players;

	assert_eq!(game.start.slippi.version, Version(0, 1, 0));
	assert_eq!(game.metadata.duration, None);

	assert_eq!(players.len(), 2);
	assert_eq!(players[0].character, External::FOX);
	assert_eq!(players[1].character, External::GANONDORF);

	Ok(())
}

#[test]
fn basic_game() -> Result<(), Error> {
	let game = game("game")?;

	assert_eq!(
		game.metadata,
		Metadata {
			date: "2018-06-22T07:52:59Z".parse::<DateTime<Utc>>().ok(),
			duration: Some(5209),
			platform: Some(Platform::Dolphin),
			players: Some(vec![
				metadata::Player {
					port: Port::P1,
					characters: {
						let mut m = HashMap::new();
						m.insert(Internal::MARTH, 5209);
						Some(m)
					},
					netplay: None,
				},
				metadata::Player {
					port: Port::P2,
					characters: {
						let mut m = HashMap::new();
						m.insert(Internal::FOX, 5209);
						Some(m)
					},
					netplay: None,
				},
			]),
			console: None,
		}
	);

	assert_eq!(
		game.start,
		Start {
			slippi: Slippi {
				version: Version(1, 0, 0)
			},
			bitfield: [50, 1, 134, 76],
			is_raining_bombs: false,
			is_teams: false,
			item_spawn_frequency: -1,
			self_destruct_score: -1,
			stage: Stage::YOSHIS_STORY,
			timer: 480,
			item_spawn_bitfield: [255, 255, 255, 255, 255],
			damage_ratio: 1.0,
			players: vec![
				Player {
					port: Port::P1,
					character: External::MARTH,
					r#type: PlayerType::HUMAN,
					stocks: 4,
					costume: Costume::from(3, External::MARTH),
					team: None,
					handicap: 9,
					bitfield: 192,
					cpu_level: None,
					offense_ratio: 1.0,
					defense_ratio: 1.0,
					model_scale: 1.0,
					ucf: Some(Ucf {
						dash_back: None,
						shield_drop: None,
					}),
					name_tag: None,
					netplay: None,
				},
				Player {
					port: Port::P2,
					character: External::FOX,
					r#type: PlayerType::CPU,
					stocks: 4,
					costume: Costume::from(0, External::FOX),
					team: None,
					handicap: 9,
					bitfield: 64,
					cpu_level: Some(1),
					offense_ratio: 1.0,
					defense_ratio: 1.0,
					model_scale: 1.0,
					ucf: Some(Ucf {
						dash_back: None,
						shield_drop: None,
					}),
					name_tag: None,
					netplay: None,
				},
			],
			random_seed: 3803194226,
			is_pal: None,
			is_frozen_ps: None,
			scene: None,
			language: None,
			raw_bytes: vec![
				1, 0, 0, 0, 50, 1, 134, 76, 195, 0, 0, 0, 0, 0, 0, 255, 255, 110, 0, 8, 0, 0, 1,
				224, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 0,
				0, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 9, 0, 4, 3, 0, 0, 0, 0, 9, 0, 120, 0, 192, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0,
				63, 128, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 2, 1, 4, 0, 0, 1, 0, 0, 9, 0, 120, 0,
				64, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0,
				26, 3, 4, 0, 0, 255, 0, 0, 9, 0, 120, 0, 64, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 63,
				128, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 26, 3, 4, 0, 0, 255, 0, 0, 9, 0, 120, 0,
				64, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0,
				33, 3, 4, 0, 0, 255, 0, 0, 9, 0, 120, 0, 64, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 63,
				128, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 33, 3, 4, 0, 0, 255, 0, 0, 9, 0, 120, 0,
				64, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0,
				226, 176, 35, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
			],
		}
	);

	assert_eq!(
		game.end,
		End {
			method: EndMethod::RESOLVED,
			lras_initiator: None,
			raw_bytes: vec![3],
		}
	);

	match game.frames {
		Frames::P2(frames) => assert_eq!(frames.len(), 5209),
		_ => Err("wrong number of ports")?,
	};

	Ok(())
}

#[test]
fn ics() -> Result<(), Error> {
	let game = game("ics")?;
	assert_eq!(
		game.metadata.players,
		Some(vec![
			metadata::Player {
				port: Port::P1,
				characters: Some({
					let mut m = HashMap::new();
					m.insert(Internal::NANA, 344);
					m.insert(Internal::POPO, 344);
					m
				}),
				netplay: None,
			},
			metadata::Player {
				port: Port::P2,
				characters: Some({
					let mut m = HashMap::new();
					m.insert(Internal::JIGGLYPUFF, 344);
					m
				}),
				netplay: None,
			},
		])
	);
	assert_eq!(game.start.players[0].character, External::ICE_CLIMBERS);
	match game.frames {
		Frames::P2(frames) => assert!(frames[0].ports[0].follower.is_some()),
		_ => Err("wrong number of ports")?,
	};
	Ok(())
}

#[test]
fn ucf() -> Result<(), Error> {
	assert_eq!(
		game("shield_drop")?.start.players[0].ucf,
		Some(Ucf {
			dash_back: None,
			shield_drop: Some(ShieldDrop::UCF)
		})
	);
	assert_eq!(
		game("dash_back")?.start.players[0].ucf,
		Some(Ucf {
			dash_back: Some(DashBack::UCF),
			shield_drop: None
		})
	);
	Ok(())
}

#[test]
fn buttons_lzrs() -> Result<(), Error> {
	let game = game("buttons_lrzs")?;
	assert_eq!(
		button_seq(&game)?,
		vec![
			Buttons {
				logical: Logical::TRIGGER_ANALOG,
				physical: Physical::NONE,
			},
			Buttons {
				logical: Logical::TRIGGER_ANALOG | Logical::L,
				physical: Physical::L,
			},
			Buttons {
				logical: Logical::TRIGGER_ANALOG,
				physical: Physical::NONE,
			},
			Buttons {
				logical: Logical::TRIGGER_ANALOG | Logical::R,
				physical: Physical::R,
			},
			Buttons {
				logical: Logical::TRIGGER_ANALOG | Logical::A | Logical::Z,
				physical: Physical::Z,
			},
			Buttons {
				logical: Logical::START,
				physical: Physical::START,
			},
		]
	);
	Ok(())
}

#[test]
fn buttons_abxy() -> Result<(), Error> {
	let game = game("buttons_abxy")?;
	assert_eq!(
		button_seq(&game)?,
		vec![
			Buttons {
				logical: Logical::A,
				physical: Physical::A,
			},
			Buttons {
				logical: Logical::B,
				physical: Physical::B,
			},
			Buttons {
				logical: Logical::X,
				physical: Physical::X,
			},
			Buttons {
				logical: Logical::Y,
				physical: Physical::Y,
			},
		]
	);
	Ok(())
}

#[test]
fn dpad_udlr() -> Result<(), Error> {
	let game = game("dpad_udlr")?;
	assert_eq!(
		button_seq(&game)?,
		vec![
			Buttons {
				logical: Logical::DPAD_UP,
				physical: Physical::DPAD_UP,
			},
			Buttons {
				logical: Logical::DPAD_DOWN,
				physical: Physical::DPAD_DOWN,
			},
			Buttons {
				logical: Logical::DPAD_LEFT,
				physical: Physical::DPAD_LEFT,
			},
			Buttons {
				logical: Logical::DPAD_RIGHT,
				physical: Physical::DPAD_RIGHT,
			},
		]
	);
	Ok(())
}

#[test]
fn cstick_udlr() -> Result<(), Error> {
	let game = game("cstick_udlr")?;
	assert_eq!(
		button_seq(&game)?,
		vec![
			Buttons {
				logical: Logical::CSTICK_UP,
				physical: Physical::NONE,
			},
			Buttons {
				logical: Logical::CSTICK_DOWN,
				physical: Physical::NONE,
			},
			Buttons {
				logical: Logical::CSTICK_LEFT,
				physical: Physical::NONE,
			},
			Buttons {
				logical: Logical::CSTICK_RIGHT,
				physical: Physical::NONE,
			},
		]
	);
	Ok(())
}

#[test]
fn joystick_udlr() -> Result<(), Error> {
	let game = game("joystick_udlr")?;
	assert_eq!(
		button_seq(&game)?,
		vec![
			Buttons {
				logical: Logical::JOYSTICK_UP,
				physical: Physical::NONE,
			},
			Buttons {
				logical: Logical::JOYSTICK_DOWN,
				physical: Physical::NONE,
			},
			Buttons {
				logical: Logical::JOYSTICK_LEFT,
				physical: Physical::NONE,
			},
			Buttons {
				logical: Logical::JOYSTICK_RIGHT,
				physical: Physical::NONE,
			},
		]
	);
	Ok(())
}

#[test]
fn nintendont() -> Result<(), Error> {
	let game = game("nintendont")?;
	assert_eq!(game.metadata.platform, Some(Platform::Nintendont));
	Ok(())
}

#[test]
fn netplay() -> Result<(), Error> {
	let game = game("netplay")?;
	let players = game.metadata.players.ok_or("missing metadata.players")?;
	let names: Vec<_> = players
		.into_iter()
		.flat_map(|p| p.netplay)
		.map(|n| n.name)
		.collect();
	assert_eq!(names, vec!["abcdefghijk", "nobody"]);
	Ok(())
}

#[test]
fn console_name() -> Result<(), Error> {
	let game = game("console_name")?;
	assert_eq!(game.metadata.console, Some("Station 1".to_string()));
	Ok(())
}

#[test]
fn v2() -> Result<(), Error> {
	let game = game("v2.0")?;
	assert_eq!(game.start.slippi.version, Version(2, 0, 1));
	Ok(())
}

#[test]
fn v3_12() -> Result<(), Error> {
	let game = game("v3.12")?;

	assert_eq!(
		game.start,
		Start {
			slippi: Slippi {
				version: Version(3, 12, 0)
			},
			bitfield: [50, 1, 142, 76],
			is_raining_bombs: false,
			is_teams: false,
			item_spawn_frequency: -1,
			self_destruct_score: -1,
			stage: Stage::POKEMON_STADIUM,
			timer: 480,
			item_spawn_bitfield: [255, 255, 255, 255, 255],
			damage_ratio: 1.0,
			players: vec![
				Player {
					port: Port::P1,
					character: External::MARTH,
					r#type: PlayerType::HUMAN,
					stocks: 4,
					costume: Costume::from(3, External::MARTH),
					team: None,
					handicap: 9,
					bitfield: 192,
					cpu_level: None,
					offense_ratio: 1.0,
					defense_ratio: 1.0,
					model_scale: 1.0,
					ucf: Some(Ucf {
						dash_back: Some(DashBack::UCF),
						shield_drop: Some(ShieldDrop::UCF)
					}),
					name_tag: Some("".to_string()),
					netplay: Some(Netplay {
						name: "xxxxxx".to_string(),
						code: "XX＃111".to_string(),
						suid: Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string())
					})
				},
				Player {
					port: Port::P2,
					character: External::MARTH,
					r#type: PlayerType::HUMAN,
					stocks: 4,
					costume: Costume::from(0, External::MARTH),
					team: None,
					handicap: 9,
					bitfield: 192,
					cpu_level: None,
					offense_ratio: 1.0,
					defense_ratio: 1.0,
					model_scale: 1.0,
					ucf: Some(Ucf {
						dash_back: Some(DashBack::UCF),
						shield_drop: Some(ShieldDrop::UCF)
					}),
					name_tag: Some("".to_string()),
					netplay: Some(Netplay {
						name: "yyyyyyyyyy".to_string(),
						code: "YYYY＃222".to_string(),
						suid: Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string())
					})
				}
			],
			random_seed: 39656,
			raw_bytes: vec![
				3, 12, 0, 0, 50, 1, 142, 76, 195, 0, 0, 0, 0, 0, 0, 255, 255, 110, 0, 3, 0, 0, 1,
				224, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 0,
				0, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 9, 0, 4, 3, 0, 0, 0, 0, 9, 0, 120, 0, 192, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0,
				63, 128, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 9, 0, 4, 0, 0, 1, 0, 0, 9, 1, 120, 0,
				192, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0,
				21, 3, 4, 0, 0, 255, 0, 0, 9, 0, 120, 0, 192, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 63,
				128, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 21, 3, 4, 0, 0, 255, 0, 0, 9, 0, 120, 0,
				192, 0, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0,
				33, 3, 4, 0, 0, 255, 0, 0, 9, 0, 120, 0, 64, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 63,
				128, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 33, 3, 4, 0, 0, 255, 0, 0, 9, 0, 120, 0,
				64, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0, 63, 128, 0, 0,
				0, 0, 154, 232, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0,
				0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 8, 120, 120, 120,
				120, 120, 120, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 121, 121, 121, 121, 121, 121, 121, 121, 121, 121, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 88, 88, 129, 148, 49, 49, 49,
				0, 0, 0, 89, 89, 89, 89, 129, 148, 50, 50, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0, 0, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97,
				97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 0, 98, 98, 98, 98, 98, 98,
				98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98, 98,
				98, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
				0, 0, 0, 0, 0, 1
			],
			is_pal: Some(false),
			is_frozen_ps: Some(false),
			scene: Some(Scene { minor: 2, major: 8 }),
			language: Some(Language::ENGLISH)
		}
	);

	Ok(())
}

#[test]
fn unknown_event() -> Result<(), Error> {
	game("unknown_event")?;
	// TODO: check for warning
	Ok(())
}

#[test]
fn zelda_sheik_transformation() -> Result<(), Error> {
	let game = game("transform")?;
	match game.frames {
		Frames::P2(frames) => assert_eq!(
			frames[400].ports[1].leader.pre.state,
			State::Zelda(Zelda::TRANSFORM_GROUND)
		),
		_ => Err("wrong number of ports")?,
	};
	Ok(())
}

#[test]
fn items() -> Result<(), Error> {
	let game = game("items")?;
	match game.frames {
		Frames::P2(frames) => {
			let mut items: HashMap<u32, Item> = HashMap::new();
			for f in frames {
				for i in f.items.unwrap() {
					items.entry(i.id).or_insert(i);
				}
			}
			assert_eq!(
				items[&0],
				Item {
					id: 0,
					damage: 0,
					direction: Some(Direction::Right),
					position: Position {
						x: -62.709_606,
						y: -1.493_274_9
					},
					state: item::State(0),
					timer: 140.0,
					r#type: item::Type::PEACH_TURNIP,
					velocity: Velocity { x: 0.0, y: 0.0 },
					misc: Some([5, 5, 5, 5]),
					owner: Some(Some(Port::P1)),
				}
			);
			assert_eq!(
				items[&1],
				Item {
					id: 1,
					damage: 0,
					direction: Some(Direction::Left),
					position: Position {
						x: 20.395_56,
						y: -1.493_274_9
					},
					state: item::State(0),
					timer: 140.0,
					r#type: item::Type::PEACH_TURNIP,
					velocity: Velocity { x: 0.0, y: 0.0 },
					misc: Some([5, 0, 5, 5]),
					owner: Some(Some(Port::P1)),
				}
			);
			assert_eq!(
				items[&2],
				Item {
					id: 2,
					damage: 0,
					direction: Some(Direction::Right),
					position: Position {
						x: -3.982_539_2,
						y: -1.493_274_9
					},
					state: item::State(0),
					timer: 140.0,
					r#type: item::Type::PEACH_TURNIP,
					velocity: Velocity { x: 0.0, y: 0.0 },
					misc: Some([5, 0, 5, 5]),
					owner: Some(Some(Port::P1)),
				}
			);
		}
		_ => Err("wrong number of ports")?,
	};
	Ok(())
}

#[test]
fn round_trip() -> Result<(), Error> {
	let game1 = game("ics2")?;
	let path = "/tmp/peppi_test_round_trip.slp";
	let mut buf = fs::File::create(path).unwrap();
	serde::ser::serialize(&mut buf, &game1)
		.map_err(|e| format!("couldn't serialize game: {:?}", e))?;
	let game2 = read_game(path)?;

	assert_eq!(game1.start, game2.start);
	assert_eq!(game1.end, game2.end);
	assert_eq!(game1.metadata, game2.metadata);
	assert_eq!(game1.metadata_raw, game2.metadata_raw);

	match (game1.frames, game2.frames) {
		(Frames::P2(f1), Frames::P2(f2)) => {
			assert_eq!(f1.len(), f2.len());
			for idx in 0..f1.len() {
				assert_eq!(f1[idx], f2[idx], "frame: {}", idx);
			}
		}
		_ => return Err("wrong number of ports".into()),
	}

	Ok(())
}

fn rollback_frames(rollback: Rollback) -> Result<Vec<frame::Frame<2>>, Error> {
	let mut buf = io::BufReader::new(fs::File::open("tests/data/ics2.slp").unwrap());
	let opts = collect::Opts { rollback };
	let game = peppi::game(&mut buf, None, Some(&opts))
		.map_err(|e| format!("couldn't deserialize game: {:?}", e))?;
	match game.frames {
		Frames::P2(frames) => Ok(frames),
		_ => Err("not a 2-player game".into()),
	}
}

#[test]
fn rollback() -> Result<(), Error> {
	let frames_all = rollback_frames(Rollback::All)?;
	let frames_first = rollback_frames(Rollback::First)?;
	let frames_last = rollback_frames(Rollback::Last)?;
	assert_eq!(frames_all.len(), 9530);
	assert_eq!(frames_first.len(), 9519);
	assert_eq!(frames_last.len(), 9519);
	assert_eq!(frames_all[474], frames_first[474]);
	assert_eq!(frames_all[475], frames_last[474]);
	assert_eq!(frames_all[476], frames_first[475]);
	assert_eq!(frames_all[476], frames_last[475]);
	Ok(())
}

#[test]
fn json_metadata() -> Result<(), Error> {
	let game = game("v3.12")?;
	let expected: serde_json::Value = serde_json::from_str(
		r#"{
			"startAt":"2022-06-04T21:58:00Z",
			"lastFrame":0,
			"players":{
				"1":{
					"names":{
						"netplay":"yyyyyyyyy",
						"code":"YYYY#222"
					},
					"characters":{
						"18":124
					}
				},
				"0":{
					"names":{
						"netplay":"xxxxxx",
						"code":"XX#111"
					},
					"characters":{
						"18":124
					}
				}
			},
			"playedOn":"dolphin"
		}"#,
	)?;
	let actual: serde_json::Value =
		serde_json::from_str(&serde_json::to_string(&game.metadata_raw)?)?;
	assert_eq!(expected, actual);
	Ok(())
}

#[test]
fn json_start() -> Result<(), Error> {
	let game = game("v3.12")?;
	let expected: serde_json::Value = serde_json::from_str(
		r#"{
			"slippi":{
				"version":[3,12,0]
			},
			"bitfield":[50,1,142,76],
			"is_raining_bombs":false,
			"is_teams":false,
			"item_spawn_frequency":-1,
			"self_destruct_score":-1,
			"stage":3,
			"timer":480,
			"item_spawn_bitfield":[255,255,255,255,255],
			"damage_ratio":1.0,
			"players":[
				{
					"port":"P1",
					"character":9,
					"type":0,
					"stocks":4,
					"costume":3,
					"team":null,
					"handicap":9,
					"bitfield":192,
					"cpu_level":null,
					"offense_ratio":1.0,
					"defense_ratio":1.0,
					"model_scale":1.0,
					"ucf":{
						"dash_back":1,
						"shield_drop":1
					},
					"name_tag":"",
					"netplay":{
						"name":"xxxxxx",
						"code":"XX＃111",
						"suid":"aaaaaaaaaaaaaaaaaaaaaaaaaaaa"
					}
				},
				{
					"port":"P2",
					"character":9,
					"type":0,
					"stocks":4,
					"costume":0,
					"team":null,
					"handicap":9,
					"bitfield":192,
					"cpu_level":null,
					"offense_ratio":1.0,
					"defense_ratio":1.0,
					"model_scale":1.0,
					"ucf":{
						"dash_back":1,
						"shield_drop":1
					},
					"name_tag":"",
					"netplay":{
						"name":"yyyyyyyyyy",
						"code":"YYYY＃222",
						"suid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbb"
					}
				}
			],
			"random_seed":39656,
			"is_pal":false,
			"is_frozen_ps":false,
			"scene":{
				"minor":2,
				"major":8
			},
			"language":1
		}"#,
	)?;
	let actual: serde_json::Value = serde_json::from_str(&serde_json::to_string(&game.start)?)?;
	assert_eq!(expected, actual);
	Ok(())
}

#[test]
fn json_end() -> Result<(), Error> {
	let game = game("v3.12")?;
	let expected: serde_json::Value = serde_json::from_str(
		r#"{
			"method":7,
			"lras_initiator":"P2"
		}"#,
	)?;
	let actual: serde_json::Value = serde_json::from_str(&serde_json::to_string(&game.end)?)?;
	assert_eq!(expected, actual);
	Ok(())
}

#[test]
fn frames_to_arrow() -> Result<(), Error> {
	let game = game("v3.12")?;
	let frames = arrow::frames_to_arrow(&game, None);

	assert_eq!(
		vec![124, 124, 124, 124, 124],
		frames.values().iter().map(|v| v.len()).collect::<Vec<_>>()
	);

	assert_eq!(
		"StructArray[{index: -123, ports: [{leader: {pre: {position: {x: -40, y: 32}, direction: 1, joystick: {x: 0, y: 0}, cstick: {x: 0, y: 0}, triggers: {logical: 0, physical: {l: 0, r: 0}}, random_seed: 39656, buttons: {logical: 0, physical: 0}, state: 322, raw_analog_x: 0, damage: 0}, post: {character: 18, state: 322, position: {x: -40, y: 32}, direction: 1, damage: 0, shield: 60, last_attack_landed: None, combo_count: 0, last_hit_by: None, stocks: 4, state_age: -1, flags: 274877906944, misc_as: 0.000000000000000000000000000000000000000000006, airborne: true, ground: 65535, jumps: 1, l_cancel: None, hurtbox_state: 0, velocities: {autogenous: {x: 0, y: 0}, knockback: {x: 0, y: 0}, autogenous_x: {air: 0, ground: 0}}, hitlag: 0, animation_index: 4294967295}}, follower: None}, {leader: {pre: {position: {x: 40, y: 32}, direction: 0, joystick: {x: 0, y: 0}, cstick: {x: 0, y: 0}, triggers: {logical: 0, physical: {l: 0, r: 0}}, random_seed: 39656, buttons: {logical: 0, physical: 0}, state: 322, raw_analog_x: 0, damage: 0}, post: {character: 18, state: 322, position: {x: 40, y: 32}, direction: 0, damage: 0, shield: 60, last_attack_landed: None, combo_count: 0, last_hit_by: None, stocks: 4, state_age: -1, flags: 274877906944, misc_as: 0.000000000000000000000000000000000000000000013, airborne: true, ground: 65535, jumps: 1, l_cancel: None, hurtbox_state: 0, velocities: {autogenous: {x: 0, y: 0}, knockback: {x: 0, y: 0}, autogenous_x: {air: 0, ground: 0}}, hitlag: 0, animation_index: 4294967295}}, follower: None}], start: {random_seed: 39656, scene_frame_counter: 0}, end: {latest_finalized_frame: -123}, items: []}]",
		format!("{:?}", frames.slice(0, 1))
	);

	assert_eq!(
		"StructArray[{index: 0, ports: [{leader: {pre: {position: {x: -35.766, y: 0.0001}, direction: 0, joystick: {x: -0.95, y: 0}, cstick: {x: 0, y: 0}, triggers: {logical: 0, physical: {l: 0, r: 0}}, random_seed: 8100584, buttons: {logical: 262144, physical: 0}, state: 20, raw_analog_x: 129, damage: 0}, post: {character: 18, state: 20, position: {x: -37.322998, y: 0.0001}, direction: 0, damage: 0, shield: 60, last_attack_landed: None, combo_count: 0, last_hit_by: None, stocks: 4, state_age: 2, flags: 0, misc_as: 0, airborne: false, ground: 34, jumps: 2, l_cancel: None, hurtbox_state: 0, velocities: {autogenous: {x: -1.557, y: -0}, knockback: {x: 0, y: 0}, autogenous_x: {air: -1.5569999, ground: -1.557}}, hitlag: 0, animation_index: 12}}, follower: None}, {leader: {pre: {position: {x: 40, y: 25.0001}, direction: 0, joystick: {x: 0, y: 0}, cstick: {x: 0, y: 0}, triggers: {logical: 1, physical: {l: 0.71428573, r: 0}}, random_seed: 8100584, buttons: {logical: 2147488096, physical: 4448}, state: 341, raw_analog_x: 0, damage: 0}, post: {character: 18, state: 341, position: {x: 40, y: 25.0001}, direction: 0, damage: 0, shield: 60, last_attack_landed: None, combo_count: 0, last_hit_by: None, stocks: 4, state_age: 10, flags: 0, misc_as: 0, airborne: false, ground: 36, jumps: 2, l_cancel: None, hurtbox_state: 0, velocities: {autogenous: {x: 0, y: 0}, knockback: {x: 0, y: 0}, autogenous_x: {air: 0, ground: 0}}, hitlag: 0, animation_index: 295}}, follower: None}], start: {random_seed: 8100584, scene_frame_counter: 123}, end: {latest_finalized_frame: 0}, items: []}]",
		format!("{:?}", frames.slice(123, 1))
	);

	Ok(())
}
