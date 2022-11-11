use std::{collections::HashMap, fs, io};

use chrono::{DateTime, Utc};

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
		metadata::{self, Metadata},
		primitives::{Direction, Port, Position, Velocity},
		slippi::{Slippi, Version},
	},
	serde::{
		self,
		collect::{self, Rollback},
	},
};

fn read_game(path: &str) -> Result<Game, String> {
	let mut buf = io::BufReader::new(fs::File::open(path).unwrap());
	peppi::game(&mut buf, None, None).map_err(|e| format!("couldn't deserialize game: {:?}", e))
}

fn game(name: &str) -> Result<Game, String> {
	read_game(&format!("tests/data/{}.slp", name))
}

fn button_seq(game: &Game) -> Result<Vec<Buttons>, String> {
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
		_ => Err("wrong number of ports".to_string()),
	}
}

#[test]
fn slippi_old_version() -> Result<(), String> {
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
fn basic_game() -> Result<(), String> {
	let game = game("game")?;

	assert_eq!(
		game.metadata,
		Metadata {
			date: "2018-06-22T07:52:59Z".parse::<DateTime<Utc>>().ok(),
			duration: Some(5209),
			platform: Some("dolphin".to_string()),
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
					offense_ratio: 0.0,
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
					offense_ratio: 0.0,
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
fn ics() -> Result<(), String> {
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
fn ucf() -> Result<(), String> {
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
fn buttons_lzrs() -> Result<(), String> {
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
fn buttons_abxy() -> Result<(), String> {
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
fn dpad_udlr() -> Result<(), String> {
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
fn cstick_udlr() -> Result<(), String> {
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
fn joystick_udlr() -> Result<(), String> {
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
fn nintendont() -> Result<(), String> {
	let game = game("nintendont")?;
	assert_eq!(game.metadata.platform, Some("nintendont".to_string()));
	Ok(())
}

#[test]
fn netplay() -> Result<(), String> {
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
fn console_name() -> Result<(), String> {
	let game = game("console_name")?;
	assert_eq!(game.metadata.console, Some("Station 1".to_string()));
	Ok(())
}

#[test]
fn v2() -> Result<(), String> {
	let game = game("v2.0")?;
	assert_eq!(game.start.slippi.version, Version(2, 0, 1));
	Ok(())
}

#[test]
fn v3_12() -> Result<(), String> {
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
					offense_ratio: 0.0,
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
					offense_ratio: 0.0,
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
fn unknown_event() -> Result<(), String> {
	game("unknown_event")?;
	// TODO: check for warning
	Ok(())
}

#[test]
fn zelda_sheik_transformation() -> Result<(), String> {
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
fn items() -> Result<(), String> {
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
fn round_trip() -> Result<(), String> {
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
		_ => return Err("wrong number of ports".to_string()),
	}

	Ok(())
}

fn rollback_frames(rollback: Rollback) -> Result<Vec<frame::Frame<2>>, String> {
	let mut buf = io::BufReader::new(fs::File::open("tests/data/ics2.slp").unwrap());
	let opts = collect::Opts { rollback };
	let game = peppi::game(&mut buf, None, Some(&opts))
		.map_err(|e| format!("couldn't deserialize game: {:?}", e))?;
	match game.frames {
		Frames::P2(frames) => Ok(frames),
		_ => Err("not a 2-player game".to_string()),
	}
}

#[test]
fn rollback() -> Result<(), String> {
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
