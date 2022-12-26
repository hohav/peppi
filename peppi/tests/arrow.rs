use peppi::serde::arrow;

mod common;
use common::game;

#[test]
fn frames_to_arrow() {
	let game = game("v3.12");
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
		"StructArray[{index: 0, ports: [{leader: {pre: {position: {x: -35.766, y: 0.0001}, direction: 0, joystick: {x: -0.95, y: 0}, cstick: {x: 0, y: 0}, triggers: {logical: 0, physical: {l: 0, r: 0}}, random_seed: 8100584, buttons: {logical: 262144, physical: 0}, state: 20, raw_analog_x: -127, damage: 0}, post: {character: 18, state: 20, position: {x: -37.322998, y: 0.0001}, direction: 0, damage: 0, shield: 60, last_attack_landed: None, combo_count: 0, last_hit_by: None, stocks: 4, state_age: 2, flags: 0, misc_as: 0, airborne: false, ground: 34, jumps: 2, l_cancel: None, hurtbox_state: 0, velocities: {autogenous: {x: -1.557, y: -0}, knockback: {x: 0, y: 0}, autogenous_x: {air: -1.5569999, ground: -1.557}}, hitlag: 0, animation_index: 12}}, follower: None}, {leader: {pre: {position: {x: 40, y: 25.0001}, direction: 0, joystick: {x: 0, y: 0}, cstick: {x: 0, y: 0}, triggers: {logical: 1, physical: {l: 0.71428573, r: 0}}, random_seed: 8100584, buttons: {logical: 2147488096, physical: 4448}, state: 341, raw_analog_x: 0, damage: 0}, post: {character: 18, state: 341, position: {x: 40, y: 25.0001}, direction: 0, damage: 0, shield: 60, last_attack_landed: None, combo_count: 0, last_hit_by: None, stocks: 4, state_age: 10, flags: 0, misc_as: 0, airborne: false, ground: 36, jumps: 2, l_cancel: None, hurtbox_state: 0, velocities: {autogenous: {x: 0, y: 0}, knockback: {x: 0, y: 0}, autogenous_x: {air: 0, ground: 0}}, hitlag: 0, animation_index: 295}}, follower: None}], start: {random_seed: 8100584, scene_frame_counter: 123}, end: {latest_finalized_frame: 0}, items: []}]",
		format!("{:?}", frames.slice(123, 1))
	);
}
