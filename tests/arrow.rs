use std::{fs, io::Cursor, path::Path};

use arrow_json::writer::ArrayWriter;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::io::BufWriter;

use peppi::{
	frame::{Frame, Frames, PortOccupancy},
	game::{Game, Port},
	io::{peppi as io_peppi, slippi as io_slippi},
};

mod common;
use common::game;

#[test]
fn into_struct_array() {
	let game = game::<Frame>("v3.12");
	let ports = vec![
		PortOccupancy {
			port: Port::P1,
			follower: false,
		},
		PortOccupancy {
			port: Port::P2,
			follower: false,
		},
	];
	let frames = game
		.frames
		.into_struct_array(game.start.slippi.version, &ports);

	assert_eq!(
		vec![124; 5],
		frames.columns().iter().map(|v| v.len()).collect::<Vec<_>>(),
	);

	{
		let mut writer = ArrayWriter::new(BufWriter::new(Vec::new()));
		writer.write(&frames.slice(0, 1).into()).unwrap();
		writer.finish().unwrap();
		let buf = writer.into_inner();
		assert_eq!(
			serde_json::from_slice::<serde_json::Value>(&buf.into_inner().unwrap()).unwrap(),
			json!([{
				"id": -123,
				"ports": {
					"P1": {
						"leader": {
							"pre": {
								"random_seed": 39656,
								"state": 322,
								"position": {
									"x": -40.0,
									"y": 32.0
								},
								"direction": 1.0,
								"joystick": {
									"x": 0.0,
									"y": 0.0
								},
								"cstick": {
									"x": 0.0,"y": 0.0
								},
								"triggers": 0.0,
								"buttons": 0,
								"buttons_physical": 0,
								"triggers_physical": {
									"l": 0.0,
									"r": 0.0
								},
								"raw_analog_x": 0,
								"percent": 0.0
							},
							"post": {
								"character": 18,
								"state": 322,
								"position": {
									"x": -40.0,
									"y": 32.0
								},
								"direction": 1.0,
								"percent": 0.0,
								"shield": 60.0,
								"last_attack_landed": 0,
								"combo_count": 0,
								"last_hit_by": 6,
								"stocks": 4,
								"state_age": -1.0,
								"state_flags": {
									"0": 0,
									"1": 0,
									"2": 0,
									"3": 0,
									"4": 64
								},
								"misc_as": 6e-45,
								"airborne": 1,
								"ground": 65535,
								"jumps": 1,
								"l_cancel": 0,
								"hurtbox_state": 0,
								"velocities": {
									"self_x_air": 0.0,
									"self_y": 0.0,
									"knockback_x": 0.0,
									"knockback_y": 0.0,
									"self_x_ground": 0.0
								},
								"hitlag": 0.0,
								"animation_index": 4294967295u32
							}
						}
					},
					"P2": {
						"leader": {
							"pre": {
								"random_seed": 39656,
								"state": 322,
								"position": {
									"x": 40.0,
									"y": 32.0
								},
								"direction": -1.0,
								"joystick": {
									"x": 0.0,
									"y": 0.0
								},
								"cstick": {
									"x": 0.0,
									"y": 0.0
								},
								"triggers": 0.0,
								"buttons": 0,
								"buttons_physical": 0,
								"triggers_physical": {
									"l": 0.0,
									"r": 0.0
								},
								"raw_analog_x": 0,
								"percent": 0.0
							},"post": {
								"character": 18,
								"state": 322,
								"position": {
									"x": 40.0,
									"y": 32.0
								},
								"direction": -1.0,
								"percent": 0.0,
								"shield": 60.0,
								"last_attack_landed": 0,
								"combo_count": 0,
								"last_hit_by": 6,
								"stocks": 4,
								"state_age": -1.0,
								"state_flags": {
									"0": 0,
									"1": 0,
									"2": 0,
									"3": 0,
									"4": 64
								},
								"misc_as": 1.3000000000000002e-44,
								"airborne": 1,
								"ground": 65535,
								"jumps": 1,
								"l_cancel": 0,
								"hurtbox_state": 0,
								"velocities": {
									"self_x_air": 0.0,
									"self_y": 0.0,
									"knockback_x": 0.0,
									"knockback_y": 0.0,
									"self_x_ground": 0.0
								},
								"hitlag": 0.0,
								"animation_index": 4294967295u32
							}
						}
					}
				},
				"start": {
					"random_seed": 39656,
					"scene_frame_counter":0
				},
				"end": {
					"latest_finalized_frame": -123
				},
				"item": []
			}]),
		);
	}

	{
		let mut buf = BufWriter::new(Vec::new());
		let mut writer = ArrayWriter::new(buf);
		writer.write(&frames.slice(123, 1).into()).unwrap();
		writer.finish().unwrap();
		buf = writer.into_inner();
		assert_eq!(
			serde_json::from_slice::<serde_json::Value>(&buf.into_inner().unwrap()).unwrap(),
			json!([{
				"id": 0,
				"start": {
					"random_seed": 8100584,
					"scene_frame_counter": 123
				},
				"end": {
					"latest_finalized_frame": 0
				},
				"ports": {
					"P1": {
						"leader": {
							"pre": {
								"random_seed": 8100584,
								"state": 20,
								"position": {
									"x": -35.766,
									"y": 0.0001
								},
								"direction": -1.0,
								"joystick": {
									"x": -0.95,
									"y": 0.0
								},
								"cstick": {
									"x": 0.0,
									"y": 0.0
								},
								"triggers": 0.0,
								"buttons": 262144,
								"buttons_physical": 0,
								"triggers_physical": {
									"l": 0.0,
									"r": 0.0
								},
								"raw_analog_x": -127,
								"percent": 0.0
							},
							"post": {
								"character": 18,
								"state": 20,
								"position": {
									"x": -37.322998,
									"y": 0.0001
								},
								"direction": -1.0,
								"percent": 0.0,
								"shield": 60.0,
								"last_attack_landed": 0,
								"combo_count": 0,
								"last_hit_by": 6,
								"stocks": 4,
								"state_age": 2.0,
								"state_flags": {
									"0": 0,
									"1": 0,
									"2": 0,
									"3": 0,
									"4": 0
								},
								"misc_as": 0.0,
								"airborne": 0,
								"ground": 34,
								"jumps": 2,
								"l_cancel": 0,
								"hurtbox_state": 0,
								"velocities": {
									"self_x_air": -1.5569999,
									"self_y": 0.0,
									"knockback_x": 0.0,
									"knockback_y": 0.0,
									"self_x_ground": -1.557
								},
								"hitlag": 0.0,
								"animation_index": 12
							}
						}
					},
					"P2": {
						"leader": {
							"pre": {
								"random_seed": 8100584,
								"state": 341,
								"position": {
									"x": 40.0,
									"y": 25.0001
								},
								"direction": -1.0,
								"joystick": {
									"x": 0.0,
									"y": 0.0
								},
								"cstick": {
									"x": 0.0,
									"y": 0.0
								},
								"triggers": 1.0,
								"buttons": 2147488096u32,
								"buttons_physical": 4448,
								"triggers_physical": {
									"l": 0.71428573,
									"r": 0.0
								},
								"raw_analog_x": 0,
								"percent": 0.0
							},
							"post": {
								"character": 18,
								"state": 341,
								"position": {
									"x": 40.0,
									"y": 25.0001
								},
								"direction": -1.0,
								"percent": 0.0,
								"shield": 60.0,
								"last_attack_landed": 0,
								"combo_count": 0,
								"last_hit_by": 6,
								"stocks": 4,
								"state_age": 10.0,
								"state_flags": {
									"0": 0,
									"1": 0,
									"2": 0,
									"3": 0,
									"4": 0
								},
								"misc_as": 0.0,
								"airborne": 0,
								"ground": 36,
								"jumps": 2,
								"l_cancel": 0,
								"hurtbox_state": 0,
								"velocities": {
									"self_x_air": 0.0,
									"self_y": 0.0,
									"knockback_x": 0.0,
									"knockback_y": 0.0,
									"self_x_ground": 0.0
								},
								"hitlag": 0.0,
								"animation_index": 295
							}
						}
					}
				},
				"item": []
			}]),
		);
	}
}

fn _round_trip(in_path: impl AsRef<Path> + Clone) {
	let bytes1 = fs::read(in_path.clone()).unwrap();

	let slippi_game = io_slippi::read(Cursor::new(bytes1.as_slice()), None).unwrap();
	let peppi_game = {
		let mut buf = Vec::new();
		io_peppi::write(&mut buf, slippi_game, Default::default()).unwrap();
		io_peppi::read(&mut &*buf, None).unwrap()
	};

	let mut bytes2 = Vec::with_capacity(bytes1.len());
	io_slippi::write(&mut bytes2, &peppi_game).unwrap();

	// If we get a perfect byte-wise match, we know we're correct.
	// If not, we'll try to detect where the difference is.
	if bytes1 == bytes2 {
		return;
	}

	let game2: Game<Frame> = io_slippi::read(Cursor::new(bytes2.as_slice()), None).unwrap();
	let game1: Game<Frame> = io_slippi::read(Cursor::new(bytes1.as_slice()), None).unwrap();

	assert_eq!(game1.start, game2.start);
	assert_eq!(game1.end, game2.end);
	assert_eq!(game1.metadata, game2.metadata);

	assert_eq!(game1.frames.len(), game2.frames.len());
	for idx in 0..game1.frames.len() {
		assert_eq!(
			game1.frames.frame(idx, game1.start.slippi.version),
			game2.frames.frame(idx, game2.start.slippi.version),
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
			"unknown_event.slp" | "corrupt.slp" => false,
			_ => true,
		}) {
		println!("{:?}", entry.file_name());
		_round_trip(entry.path());
	}
}
