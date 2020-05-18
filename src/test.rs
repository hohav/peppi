use std::collections::{HashMap};
use std::path;

use chrono::{DateTime, Utc};

use super::action_state::{State, Zelda};
use super::buttons::{Logical, Physical};
use super::character::{Internal, External};
use super::frame::{Buttons};
use super::game::{DashBack, Game, End, EndMethod, Start, Player, PlayerType, PlayerV1_0, ShieldDrop, Slippi, SlippiVersion, Ucf};
use super::metadata::{Metadata, MetadataPlayer};
use super::stage::{Stage};
use super::ubjson::{ToObject};

macro_rules! map {
	{ $($key:expr => $value:expr),* $(,)? } => {{
		let mut m = std::collections::HashMap::new();
		$( m.insert($key.to_string(), $value.to_object()); )+
		m
	}}
}

fn game(name:&str) -> Result<Game, String> {
	super::game(path::Path::new(&format!("test/replays/{}.slp", name))).map_err(|e| format!("couldn't parse game: {:?}", e))
}

fn button_seq(game:&Game) -> Result<Vec<Buttons>, String> {
	let mut last_buttons:Option<Buttons> = None;
	let mut button_seq = Vec::<Buttons>::new();
	for frame in &game.ports[0].as_ref().ok_or("port 0 missing")?.leader.pre {
		let b = frame.buttons;
		if (b.logical.0 > 0 || b.physical.0 > 0) && Some(b) != last_buttons {
			button_seq.push(b);
			last_buttons = Some(b);
		}
	}
	Ok(button_seq)
}

#[test]
fn slippi_old_version() -> Result<(), String> {
	let game = game("v0.1")?;
	let players = game.start.players;

	assert_eq!(game.start.slippi.version, SlippiVersion(0,1,0));
	assert_eq!(game.metadata.duration, None);

	assert_eq!(players[0].as_ref().ok_or("player 0 missing")?.character, External::FOX);
	assert_eq!(players[1].as_ref().ok_or("player 1 missing")?.character, External::GANONDORF);
	assert!(players[2].is_none());
	assert!(players[3].is_none());

	Ok(())
}

#[test]
fn basic_game() -> Result<(), String> {
	let game = game("game")?;

	assert_eq!(game.metadata, Metadata {
		date: "2018-06-22T07:52:59Z".parse::<DateTime<Utc>>().ok(),
		duration: Some(5209),
		platform: Some("dolphin".to_string()),
		players: Some([
			Some(MetadataPlayer {
				characters: {
					let mut m:HashMap<Internal, u32> = HashMap::new();
					m.insert(Internal::MARTH, 5209);
					Some(m)
				},
				netplay_name: None,
			}),
			Some(MetadataPlayer {
				characters: {
					let mut m:HashMap<Internal, u32> = HashMap::new();
					m.insert(Internal::FOX, 5209);
					Some(m)
				},
				netplay_name: None,
			}),
			None,
			None,
		]),
		console_name: None,
		json: map! {
			"startAt" => "2018-06-22T07:52:59Z",
			"lastFrame" => 5085,
			"playedOn" => "dolphin",
			"players" => map! {
				"0" => map! {
					"characters" => map! {
						format!("{}", Internal::MARTH.0) => 5209,
					},
				},
				"1" => map! {
					"characters" => map! {
						format!("{}", Internal::FOX.0) => 5209,
					},
				},
			},
		},
	});

	assert_eq!(game.start, Start {
		slippi: Slippi { version: SlippiVersion(1, 0, 0) },
		bitfield: [50, 1, 76],
		is_teams: false,
		item_spawn_frequency: -1,
		self_destruct_score: -1,
		stage: Stage::YOSHIS_STORY,
		timer: 480,
		item_spawn_bitfield: [255, 255, 255, 255, 255],
		damage_ratio: 1.0,
		players: [
			Some(Player {
				character: External::MARTH,
				r#type: PlayerType::HUMAN,
				stocks: 4,
				costume: 3,
				team: None,
				handicap: 9,
				bitfield: 192,
				cpu_level: None,
				offense_ratio: 0.0,
				defense_ratio: 1.0,
				model_scale: 1.0,
				v1_0: Some(PlayerV1_0 {
					ucf: Ucf {
						dash_back: None,
						shield_drop: None
					},
					v1_3: None
				}),
			}),
			Some(Player {
				character: External::FOX,
				r#type: PlayerType::CPU,
				stocks: 4,
				costume: 0,
				team: None,
				handicap: 9,
				bitfield: 64,
				cpu_level: Some(1),
				offense_ratio: 0.0,
				defense_ratio: 1.0,
				model_scale: 1.0,
				v1_0: Some(PlayerV1_0 {
					ucf: Ucf {
						dash_back: None,
						shield_drop: None
					},
					v1_3: None
				}),
			}),
			None,
			None,
		],
		random_seed: 3803194226,
		v1_5: None,
	});

	assert_eq!(game.end, End {
		method: EndMethod::RESOLVED,
		v2_0: None,
	});

	assert_eq!(
		game.ports.iter().map(|p| p.as_ref().map(|p| p.leader.pre.len())).collect::<Vec<Option<usize>>>(),
		vec![Some(5209), Some(5209), None, None]);

	Ok(())
}

#[test]
fn ics() -> Result<(), String> {
	let game = game("ics")?;
	assert_eq!(game.metadata.players, Some([
		Some(MetadataPlayer {
			characters: Some({
				let mut m = HashMap::new();
				m.insert(Internal::NANA, 344);
				m.insert(Internal::POPO, 344);
				m
			}),
			netplay_name: None,
		}),
		Some(MetadataPlayer {
			characters: Some({
				let mut m = HashMap::new();
				m.insert(Internal::JIGGLYPUFF, 344);
				m
			}),
			netplay_name: None,
		}),
		None,
		None,
	]));
	assert_eq!(game.start.players[0].as_ref().map(|p| p.character), Some(External::ICE_CLIMBERS));
	assert!(game.ports[0].as_ref().ok_or("player 0 missing")?.follower.is_some());
	Ok(())
}

#[test]
fn ucf() -> Result<(), String> {
	assert_eq!(game("shield_drop")?.start.players[0].as_ref().ok_or("missing players[0]")?.v1_0.as_ref().ok_or("missing players[0].v1_0")?.ucf,
		Ucf { dash_back: None, shield_drop: Some(ShieldDrop::UCF) });
	assert_eq!(game("dash_back")?.start.players[0].as_ref().ok_or("missing players[0]")?.v1_0.as_ref().ok_or("missing players[0].v1_0")?.ucf,
		Ucf { dash_back: Some(DashBack::UCF), shield_drop: None });
	Ok(())
}

#[test]
fn buttons_lzrs() -> Result<(), String> {
	let game = game("buttons_lrzs")?;
	assert_eq!(button_seq(&game)?, vec![
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
	]);
	Ok(())
}

#[test]
fn buttons_abxy() -> Result<(), String> {
	let game = game("buttons_abxy")?;
	assert_eq!(button_seq(&game)?, vec![
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
	]);
	Ok(())
}

#[test]
fn dpad_udlr() -> Result<(), String> {
	let game = game("dpad_udlr")?;
	assert_eq!(button_seq(&game)?, vec![
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
	]);
	Ok(())
}

#[test]
fn cstick_udlr() -> Result<(), String> {
	let game = game("cstick_udlr")?;
	assert_eq!(button_seq(&game)?, vec![
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
	]);
	Ok(())
}

#[test]
fn joystick_udlr() -> Result<(), String> {
	let game = game("joystick_udlr")?;
	assert_eq!(button_seq(&game)?, vec![
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
	]);
	Ok(())
}

#[test]
fn nintendont() -> Result<(), String> {
	let game = game("nintendont")?;
	assert_eq!(game.metadata.platform, Some("nintendont".to_string()));
	Ok(())
}

#[test]
fn netplay_name() -> Result<(), String> {
	let game = game("netplay_name")?;
	let players = game.metadata.players.ok_or("missing metadata.players")?;
	assert_eq!(players[0].as_ref().and_then(|p| p.netplay_name.as_ref()), Some(&"Player1".to_string()));
	assert_eq!(players[1].as_ref().and_then(|p| p.netplay_name.as_ref()), Some(&"metonym".to_string()));
	Ok(())
}

#[test]
fn console_name() -> Result<(), String> {
	let game = game("console_name")?;
	assert_eq!(game.metadata.console_name, Some("Station 1".to_string()));
	Ok(())
}

#[test]
fn v2() -> Result<(), String> {
	let game = game("v2.0")?;
	assert_eq!(game.start.slippi.version, SlippiVersion(2,0,1));
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
	assert_eq!(game.ports[3].as_ref().ok_or("missing port 3")?.leader.pre[400].state,
		State::Zelda(Zelda::TRANSFORM_GROUND));
	Ok(())
}
