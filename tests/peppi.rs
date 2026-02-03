use std::{collections::HashSet, fs, io::Cursor, path::Path};

use pretty_assertions::assert_eq;
use serde_json::json;

use ssbm_data::{action_state, character::External, character::Internal, item::Item, stage::Stage};

use peppi::{
	frame::{
		Rollbacks,
		transpose::{self, Position},
	},
	game::{
		Bytes, DashBack, End, EndMethod, Language, Match, Netplay, Player, PlayerEnd, PlayerType,
		Port, Scene, ShieldDrop, Start, Ucf, immutable::Game, shift_jis::MeleeString,
	},
	io::{
		peppi::{self as io_peppi},
		slippi::{self, Slippi, Version},
	},
};

mod common;
use common::{game, get_path, read_game};

#[derive(Copy, Clone, Debug, PartialEq)]
struct Buttons {
	physical: u16,
	logical: u32,
}

fn button_seq(game: &Game) -> Vec<Buttons> {
	let mut last_buttons: Option<Buttons> = None;
	let mut button_seq = vec![];
	for idx in 0..game.frames.len() {
		let b = Buttons {
			logical: game.frames.ports[0].leader.pre.buttons.values()[idx],
			physical: game.frames.ports[0].leader.pre.buttons_physical.values()[idx],
		};
		if (b.physical > 0 || b.logical > 0) && Some(b) != last_buttons {
			button_seq.push(b);
			last_buttons = Some(b);
		}
	}
	button_seq
}

#[test]
fn slippi_old_version() {
	let game = game("v0.1");
	let players = game.start.players;

	assert_eq!(game.start.slippi.version, Version(0, 1, 0));
	assert_eq!(
		serde_json::Value::Object(game.metadata.unwrap()),
		json!({
			"startAt": "2018-01-24T06:19:54Z",
			"playedOn": "dolphin"
		})
	);

	assert_eq!(players.len(), 2);
	assert_eq!(players[0].character, External::Fox as u8);
	assert_eq!(players[1].character, External::Ganondorf as u8);
}

#[test]
fn basic_game() {
	let game = game("game");

	assert_eq!(
		serde_json::Value::Object(game.metadata.unwrap()),
		json!({
			"startAt": "2018-06-22T07:52:59Z",
			"lastFrame": 5085,
			"players": {
				"1": {
					"characters": {
						"1": 5209, // Fox
					}
				},
				"0": {
					"characters": {
						"18": 5209 // Marth
					}
				}
			},
			"playedOn": "dolphin"
		})
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
			stage: Stage::YoshisStory as u16,
			timer: 480,
			item_spawn_bitfield: [255, 255, 255, 255, 255],
			damage_ratio: 1.0,
			players: vec![
				Player {
					port: Port::P1,
					character: External::Marth as u8,
					r#type: PlayerType::Human,
					stocks: 4,
					costume: 3,
					team: None,
					handicap: 9,
					bitfield: 192,
					cpu_level: None,
					damage_start: 0,
					damage_spawn: 0,
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
					character: External::Fox as u8,
					r#type: PlayerType::Cpu,
					stocks: 4,
					costume: 0,
					team: None,
					handicap: 9,
					bitfield: 64,
					cpu_level: Some(1),
					damage_start: 0,
					damage_spawn: 0,
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
			r#match: None,
			bytes: Bytes(vec![
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
			]),
		}
	);

	assert_eq!(
		game.end.unwrap(),
		End {
			method: EndMethod::Resolved,
			bytes: Bytes(vec![3]),
			lras_initiator: None,
			players: None,
		}
	);

	assert_eq!(game.frames.len(), 5209);

	assert_eq!(
		game.frames.transpose_one(1000, game.start.slippi.version),
		transpose::Frame {
			id: 877,
			ports: vec![
				transpose::PortData {
					port: Port::P1,
					leader: transpose::Data {
						pre: transpose::Pre {
							random_seed: 1046068084,
							state: action_state::Common::JumpAerialF as u16,
							position: Position {
								x: 56.81875,
								y: -18.63733,
							},
							direction: -1.0,
							joystick: Position { x: 0.0, y: 0.0 },
							cstick: Position { x: 0.0, y: 0.0 },
							triggers: 0.0,
							buttons: 0,
							buttons_physical: 0,
							triggers_physical: transpose::TriggersPhysical {
								l: 0.0,
								r: 3.7793343e22,
							},
							..Default::default()
						},
						post: transpose::Post {
							character: Internal::Marth as u8,
							state: action_state::Common::JumpAerialF as u16,
							position: Position {
								x: 57.292168,
								y: -17.290329,
							},
							direction: -1.0,
							percent: 0.0,
							shield: 60.0,
							last_attack_landed: 15,
							combo_count: 1,
							last_hit_by: 6,
							stocks: 4,
							state_age: Some(8.0,),
							..Default::default()
						},
					},
					follower: None,
				},
				transpose::PortData {
					port: Port::P2,
					leader: transpose::Data {
						pre: transpose::Pre {
							random_seed: 1046068084,
							state: action_state::Fox::FireFoxAir as u16,
							position: Position {
								x: 42.195168,
								y: 9.287016,
							},
							direction: -1.0,
							joystick: Position {
								x: -0.6875,
								y: 0.6929134,
							},
							cstick: Position { x: 0.0, y: 0.0 },
							triggers: 0.0,
							buttons: 0,
							buttons_physical: 0,
							triggers_physical: transpose::TriggersPhysical {
								l: 0.0,
								r: 3.7793343e22,
							},
							..Default::default()
						},
						post: transpose::Post {
							character: Internal::Fox as u8,
							state: action_state::Fox::FireFoxAir as u16,
							position: Position {
								x: 40.50478,
								y: 10.990714,
							},
							direction: -1.0,
							percent: 85.6,
							shield: 60.0,
							last_attack_landed: 0,
							combo_count: 0,
							last_hit_by: 0,
							stocks: 4,
							state_age: Some(18.0,),
							..Default::default()
						},
					},
					follower: None,
				},
			],
			..Default::default()
		}
	);
}

#[test]
fn skip_frames() {
	let g1 = read_game(get_path("game"), false).unwrap();
	let g2 = read_game(get_path("game"), true).unwrap();

	assert_eq!(g1.start, g2.start);
	assert_eq!(g1.end, g2.end);
	assert_eq!(g1.metadata, g2.metadata);
	assert_eq!(g2.frames.len(), 0);
}

#[test]
fn ics() {
	let game = game("ics");
	assert_eq!(
		game.metadata.unwrap()["players"],
		json!({
			"1": {
				"characters": {
					"15": 344 // Jigglypuff
				}
			},
			"0": {
				"characters": {
					"11": 344, // Nana
					"10": 344 // Popo
				}
			}
		})
	);
	assert_eq!(game.start.players[0].character, 14);
	assert!(game.frames.ports[0].follower.is_some());
}

#[test]
fn ucf() {
	assert_eq!(
		game("shield_drop").start.players[0].ucf,
		Some(Ucf {
			dash_back: None,
			shield_drop: Some(ShieldDrop::Ucf),
		})
	);
	assert_eq!(
		game("dash_back").start.players[0].ucf,
		Some(Ucf {
			dash_back: Some(DashBack::Ucf),
			shield_drop: None,
		})
	);
}

#[test]
fn buttons_lzrs() {
	let game = game("buttons_lrzs");
	assert_eq!(
		button_seq(&game),
		vec![
			Buttons {
				logical: 2147483648, // Trigger
				physical: 0,
			},
			Buttons {
				logical: 2147483712, // Trigger | L
				physical: 64,        // L
			},
			Buttons {
				logical: 2147483648, // Trigger
				physical: 0,
			},
			Buttons {
				logical: 2147483680, // Trigger | R
				physical: 32,        // R
			},
			Buttons {
				logical: 2147483920, // Trigger | A | Z
				physical: 16,        // Z
			},
			Buttons {
				logical: 4096,  // Start
				physical: 4096, // Start
			},
		]
	);
}

#[test]
fn buttons_abxy() {
	let game = game("buttons_abxy");
	assert_eq!(
		button_seq(&game),
		vec![
			Buttons {
				logical: 256,  // A
				physical: 256, // A
			},
			Buttons {
				logical: 512,  // B
				physical: 512, // B
			},
			Buttons {
				logical: 1024,  // X
				physical: 1024, // X
			},
			Buttons {
				logical: 2048,  // Y
				physical: 2048, // Y
			},
		]
	);
}

#[test]
fn dpad_udlr() {
	let game = game("dpad_udlr");
	assert_eq!(
		button_seq(&game),
		vec![
			Buttons {
				logical: 8,  // D-pad up
				physical: 8, // D-pad up
			},
			Buttons {
				logical: 4,  // D-pad down
				physical: 4, // D-pad down
			},
			Buttons {
				logical: 1,  // D-pad left
				physical: 1, // D-pad left
			},
			Buttons {
				logical: 2,  // D-pad right
				physical: 2, // D-pad right
			},
		]
	);
}

#[test]
fn cstick_udlr() {
	let game = game("cstick_udlr");
	assert_eq!(
		button_seq(&game),
		vec![
			Buttons {
				logical: 1048576, // C-stick up
				physical: 0,
			},
			Buttons {
				logical: 2097152, // C-stick down
				physical: 0,
			},
			Buttons {
				logical: 4194304, // C-stick left
				physical: 0,
			},
			Buttons {
				logical: 8388608, // C-stick right
				physical: 0,
			},
		]
	);
}

#[test]
fn joystick_udlr() {
	let game = game("joystick_udlr");
	assert_eq!(
		button_seq(&game),
		vec![
			Buttons {
				logical: 65536, // Joystick up
				physical: 0,
			},
			Buttons {
				logical: 131072, // Joystick down
				physical: 0,
			},
			Buttons {
				logical: 262144, // Joystick left
				physical: 0,
			},
			Buttons {
				logical: 524288, // Joystick right
				physical: 0,
			},
		]
	);
}

#[test]
fn nintendont() {
	let game = game("nintendont");
	assert_eq!(
		game.metadata.unwrap()["playedOn"],
		serde_json::Value::String("nintendont".to_string())
	);
}

#[test]
fn netplay() {
	let game = game("netplay");
	assert_eq!(
		game.metadata.unwrap()["players"],
		json!({
			"0": {
				"names": {
					"netplay": "abcdefghijk",
					"code": "ABCD#123"
				},
				"characters": {
					"13": 128,
				}
			},
			"1": {
				"names": {
					"netplay": "nobody",
					"code": "XX#000"
				},
				"characters": {
					"18": 128,
				}
			}
		})
	);
}

#[test]
fn console_name() {
	let game = game("console_name");
	assert_eq!(
		game.metadata.unwrap()["consoleNick"],
		serde_json::Value::String("Station 1".to_string())
	)
}

#[test]
fn v2() {
	let game = game("v2.0");
	assert_eq!(game.start.slippi.version, Version(2, 0, 1));
}

#[test]
fn v3_12() {
	let game = game("v3.12");

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
			stage: Stage::PokemonStadium as u16,
			timer: 480,
			item_spawn_bitfield: [255, 255, 255, 255, 255],
			damage_ratio: 1.0,
			players: vec![
				Player {
					port: Port::P1,
					character: External::Marth as u8,
					r#type: PlayerType::Human,
					stocks: 4,
					costume: 3,
					team: None,
					handicap: 9,
					bitfield: 192,
					cpu_level: None,
					damage_start: 0,
					damage_spawn: 0,
					offense_ratio: 1.0,
					defense_ratio: 1.0,
					model_scale: 1.0,
					ucf: Some(Ucf {
						dash_back: Some(DashBack::Ucf),
						shield_drop: Some(ShieldDrop::Ucf),
					}),
					name_tag: Some(MeleeString("".to_string())),
					netplay: Some(Netplay {
						name: MeleeString("xxxxxx".to_string()),
						code: MeleeString("XX＃111".to_string()),
						suid: Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string())
					})
				},
				Player {
					port: Port::P2,
					character: External::Marth as u8,
					r#type: PlayerType::Human,
					stocks: 4,
					costume: 0,
					team: None,
					handicap: 9,
					bitfield: 192,
					cpu_level: None,
					damage_start: 0,
					damage_spawn: 0,
					offense_ratio: 1.0,
					defense_ratio: 1.0,
					model_scale: 1.0,
					ucf: Some(Ucf {
						dash_back: Some(DashBack::Ucf),
						shield_drop: Some(ShieldDrop::Ucf),
					}),
					name_tag: Some(MeleeString("".to_string())),
					netplay: Some(Netplay {
						name: MeleeString("yyyyyyyyyy".to_string()),
						code: MeleeString("YYYY＃222".to_string()),
						suid: Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string())
					})
				}
			],
			random_seed: 39656,
			is_pal: Some(false),
			is_frozen_ps: Some(false),
			scene: Some(Scene { minor: 2, major: 8 }),
			language: Some(Language::English),
			r#match: None,
			bytes: Bytes(vec![
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
			]),
		}
	);
}

#[test]
fn v3_13() {
	let game = game("v3.13");
	assert_eq!(
		game.end,
		Some(End {
			method: EndMethod::Game,
			bytes: Bytes(vec![2, 255, 1, 255, 0, 255]),
			lras_initiator: Some(None),
			players: Some(vec![
				PlayerEnd {
					port: Port::P1,
					placement: 1
				},
				PlayerEnd {
					port: Port::P3,
					placement: 0
				},
			]),
		})
	);
}

#[test]
fn v3_14() {
	let game = game("v3.16");
	assert_eq!(
		game.start.r#match,
		Some(Match {
			id: String::from("mode.unranked-2024-02-15T14:37:23.22-0"),
			game: 1,
			tiebreaker: 0
		})
	)
}

#[test]
fn v3_15() {
	let game = game("v3.16");
	assert_eq!(
		game.frames
			.transpose_one(200, game.start.slippi.version)
			.ports[0]
			.leader
			.pre
			.raw_analog_y,
		Some(-21)
	)
}

#[test]
fn v3_16() {
	let game = game("v3.16");
	let player1_ids = game.frames.ports[0]
		.leader
		.post
		.instance_id
		.as_ref()
		.unwrap();
	let player2_ids = game.frames.ports[1]
		.leader
		.post
		.instance_id
		.as_ref()
		.unwrap();
	let mut id_set: HashSet<u16> = player1_ids.values_iter().cloned().collect();
	id_set.remove(&0);

	// player1 and player2 ids should not share any instance ids
	assert!(player2_ids.values_iter().all(|id| !id_set.contains(id)));

	let player2_hit_bys = game.frames.ports[1]
		.leader
		.post
		.last_hit_by_instance
		.as_ref()
		.unwrap();

	// player2 should be hit by player1 ids
	assert!(
		player2_hit_bys
			.values_iter()
			.all(|id| *id == 0 || id_set.contains(id))
	);

	let items = game.frames.item.as_ref().unwrap();
	let item_instance_ids = items.instance_id.as_ref().unwrap();
	let item_types = &items.r#type;

	// Shy guy (210) should have instance id of 0
	// The laser/gun should share instance id with the Fox player (P1)
	assert!(
		item_instance_ids
			.values_iter()
			.zip(item_types.values_iter())
			.all(|(&id, &r#type)| (r#type == 210 && id == 0) || id_set.contains(&id))
	);
}

#[test]
fn v3_18() {
	let game = game("v3.18");
	assert_eq!(
		game.frames
			.transpose_one(822, game.start.slippi.version)
			.fod_platforms,
		Some(vec![])
	);
	assert_eq!(
		game.frames
			.transpose_one(823, game.start.slippi.version)
			.fod_platforms,
		Some(vec![transpose::FodPlatform {
			platform: 1,
			height: 20.15f32,
		}])
	);
}

#[test]
fn unknown_event() {
	// shouldn't panic
	// TODO: check for warning
	game("unknown_event");
}

#[test]
fn corrupt_replay() {
	assert!(matches!(read_game(get_path("corrupt"), false), Err(_)));
}

#[test]
fn multi_gecko_list() {
	let err = read_game(get_path("multi_gecko_list"), false).unwrap_err();
	assert!(
		err.to_string().contains("Multiple Gecko List"),
		"Expected error about multiple Gecko List events, got: {}",
		err
	);
}

#[test]
fn zelda_sheik_transformation() {
	let game = game("transform");
	assert_eq!(
		game.frames.ports[1].leader.pre.state.values()[400],
		action_state::Zelda::TransformGround as u16,
	);
}

#[test]
fn items() {
	let game = game("items");
	assert_eq!(
		game.frames
			.transpose_one(121, game.start.slippi.version)
			.items,
		Some(vec![transpose::Item {
			id: 0,
			damage: 0,
			direction: 1.0,
			position: transpose::Position {
				x: -62.709_606,
				y: -1.493_274_9
			},
			state: 0,
			timer: 140.0,
			r#type: Item::PeachTurnip as u16,
			velocity: transpose::Velocity { x: 0.0, y: 0.0 },
			misc: Some(transpose::ItemMisc(5, 5, 5, 5)),
			owner: Some(0),
			instance_id: None,
		}])
	);
	assert_eq!(
		game.frames
			.transpose_one(275, game.start.slippi.version)
			.items,
		Some(vec![transpose::Item {
			id: 1,
			damage: 0,
			direction: -1.0,
			position: transpose::Position {
				x: 20.395_56,
				y: -1.493_274_9
			},
			state: 0,
			timer: 140.0,
			r#type: Item::PeachTurnip as u16,
			velocity: transpose::Velocity { x: 0.0, y: 0.0 },
			misc: Some(transpose::ItemMisc(5, 0, 5, 5)),
			owner: Some(0),
			instance_id: None,
		}])
	);
	assert_eq!(
		game.frames
			.transpose_one(503, game.start.slippi.version)
			.items,
		Some(vec![transpose::Item {
			id: 2,
			damage: 0,
			direction: 1.0,
			position: transpose::Position {
				x: -3.982_539_2,
				y: -1.493_274_9
			},
			state: 0,
			timer: 140.0,
			r#type: Item::PeachTurnip as u16,
			velocity: transpose::Velocity { x: 0.0, y: 0.0 },
			misc: Some(transpose::ItemMisc(5, 0, 5, 5)),
			owner: Some(0),
			instance_id: None,
		}])
	);
}

fn _round_trip(in_path: impl AsRef<Path> + Clone) {
	let bytes1 = fs::read(in_path.clone()).unwrap();

	let slippi_game = slippi::read(Cursor::new(bytes1.as_slice()), None).unwrap();
	let peppi_game = {
		let mut buf = Vec::new();
		io_peppi::write(&mut buf, slippi_game, Default::default()).unwrap();
		io_peppi::read(&mut &*buf, None).unwrap()
	};

	let mut bytes2 = Vec::with_capacity(bytes1.len());
	slippi::write(&mut bytes2, &peppi_game).unwrap();

	// If we get a perfect byte-wise match, we know we're correct.
	// If not, we'll try to detect where the difference is.
	if bytes1 == bytes2 {
		return;
	}

	let game2 = slippi::read(Cursor::new(bytes2.as_slice()), None).unwrap();
	let game1 = slippi::read(Cursor::new(bytes1.as_slice()), None).unwrap();

	assert_eq!(game1.start, game2.start);
	assert_eq!(game1.end, game2.end);
	assert_eq!(game1.metadata, game2.metadata);

	assert_eq!(game1.frames.len(), game2.frames.len());
	for idx in 0..game1.frames.len() {
		assert_eq!(
			game1.frames.transpose_one(idx, game1.start.slippi.version),
			game2.frames.transpose_one(idx, game2.start.slippi.version),
		);
	}

	assert_eq!(bytes1.len(), bytes2.len());
	assert_eq!(bytes1, bytes2);
}

#[test]
fn round_trip() {
	for entry in fs::read_dir("tests/data")
		.unwrap()
		.into_iter()
		.map(|e| e.unwrap())
		.filter(|e| match e.file_name().to_str().unwrap() {
			"unknown_event.slp" | "corrupt.slp" | "multi_gecko_list.slp" => false,
			_ => true,
		}) {
		println!("{:?}", entry.file_name());
		_round_trip(entry.path());
	}
}

#[test]
fn rollbacks() {
	let game = game("ics2");
	assert_eq!(game.frames.len(), 9530);
	assert_eq!(
		game.frames.id.values().clone().sliced(473, 4).as_slice(),
		[350, 351, 351, 352]
	);
	assert_eq!(
		game.frames.rollbacks(Rollbacks::ExceptFirst)[473..477],
		[false, false, true, false],
		"returns true for second instance of frame 351"
	);
	assert_eq!(
		game.frames.rollbacks(Rollbacks::ExceptLast)[473..477],
		[false, true, false, false],
		"returns true for first instance of frame 351"
	);
}
