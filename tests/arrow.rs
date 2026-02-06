use arrow2::io::json::write as json_write;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::io::BufWriter;

use peppi::{frame::PortOccupancy, game::Port};

mod common;
use common::game;

/// Test that v3.6 replays can be converted to Arrow.
/// v3.6 predates the `latest_finalized_frame` field (added in v3.7),
/// so the `End` struct has no fields for this version.
#[test]
fn v3_6_into_struct_array() {
	let game = game("v3.6");
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
	// This should not panic
	let _frames = game
		.frames
		.into_struct_array(game.start.slippi.version, &ports);
}

#[test]
fn into_struct_array() {
	let game = game("v3.12");
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
		frames.values().iter().map(|v| v.len()).collect::<Vec<_>>(),
	);

	let frames = frames.boxed();
	{
		let mut serializer =
			json_write::Serializer::new(vec![Ok(frames.sliced(0, 1))].into_iter(), vec![]);
		let mut buf = BufWriter::new(Vec::new());
		json_write::write(&mut buf, &mut serializer).unwrap();
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
		let mut serializer =
			json_write::Serializer::new(vec![Ok(frames.sliced(123, 1))].into_iter(), vec![]);
		let mut buf = BufWriter::new(Vec::new());
		json_write::write(&mut buf, &mut serializer).unwrap();
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
