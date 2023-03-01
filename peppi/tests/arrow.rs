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
		"StructArray[{index: -123, ports: [{leader: {pre: {position: {x: -40, y: 32}, facing_direction: 1, joystick: {x: 0, y: 0}, cstick: {x: 0, y: 0}, triggers: {logical: 0, physical: {l: 0, r: 0}}, random_seed: 39656, buttons: {logical: 0, physical: 0}, state: 322, raw_analog_x: 0, percent: 0}, post: {character: 18, state: 322, position: {x: -40, y: 32}, facing_direction: 1, percent: 0, shield_health: 60, last_attack_landed: None, combo_count: 0, last_hit_by: None, stocks_remaining: 4, state_age: -1, flags: 274877906944, misc_as: 0.000000000000000000000000000000000000000000006, is_airborne: true, last_ground_id: 65535, jumps_remaining: 1, l_cancel: None, hurtbox_state: 0, velocities: {self_induced: {x: 0, y: 0}, knockback: {x: 0, y: 0}, self_induced_x: {air: 0, ground: 0}}, hitlag_remaining: 0, animation_index: 4294967295}}, follower: None}, {leader: {pre: {position: {x: 40, y: 32}, facing_direction: 0, joystick: {x: 0, y: 0}, cstick: {x: 0, y: 0}, triggers: {logical: 0, physical: {l: 0, r: 0}}, random_seed: 39656, buttons: {logical: 0, physical: 0}, state: 322, raw_analog_x: 0, percent: 0}, post: {character: 18, state: 322, position: {x: 40, y: 32}, facing_direction: 0, percent: 0, shield_health: 60, last_attack_landed: None, combo_count: 0, last_hit_by: None, stocks_remaining: 4, state_age: -1, flags: 274877906944, misc_as: 0.000000000000000000000000000000000000000000013, is_airborne: true, last_ground_id: 65535, jumps_remaining: 1, l_cancel: None, hurtbox_state: 0, velocities: {self_induced: {x: 0, y: 0}, knockback: {x: 0, y: 0}, self_induced_x: {air: 0, ground: 0}}, hitlag_remaining: 0, animation_index: 4294967295}}, follower: None}], start: {random_seed: 39656, scene_frame_counter: 0}, end: {latest_finalized_frame: -123}, items: []}]",
		format!("{:?}", frames.slice(0, 1))
	);

	assert_eq!(
		"StructArray[{index: 0, ports: [{leader: {pre: {position: {x: -35.766, y: 0.0001}, facing_direction: 0, joystick: {x: -0.95, y: 0}, cstick: {x: 0, y: 0}, triggers: {logical: 0, physical: {l: 0, r: 0}}, random_seed: 8100584, buttons: {logical: 262144, physical: 0}, state: 20, raw_analog_x: -127, percent: 0}, post: {character: 18, state: 20, position: {x: -37.322998, y: 0.0001}, facing_direction: 0, percent: 0, shield_health: 60, last_attack_landed: None, combo_count: 0, last_hit_by: None, stocks_remaining: 4, state_age: 2, flags: 0, misc_as: 0, is_airborne: false, last_ground_id: 34, jumps_remaining: 2, l_cancel: None, hurtbox_state: 0, velocities: {self_induced: {x: -1.557, y: -0}, knockback: {x: 0, y: 0}, self_induced_x: {air: -1.5569999, ground: -1.557}}, hitlag_remaining: 0, animation_index: 12}}, follower: None}, {leader: {pre: {position: {x: 40, y: 25.0001}, facing_direction: 0, joystick: {x: 0, y: 0}, cstick: {x: 0, y: 0}, triggers: {logical: 1, physical: {l: 0.71428573, r: 0}}, random_seed: 8100584, buttons: {logical: 2147488096, physical: 4448}, state: 341, raw_analog_x: 0, percent: 0}, post: {character: 18, state: 341, position: {x: 40, y: 25.0001}, facing_direction: 0, percent: 0, shield_health: 60, last_attack_landed: None, combo_count: 0, last_hit_by: None, stocks_remaining: 4, state_age: 10, flags: 0, misc_as: 0, is_airborne: false, last_ground_id: 36, jumps_remaining: 2, l_cancel: None, hurtbox_state: 0, velocities: {self_induced: {x: 0, y: 0}, knockback: {x: 0, y: 0}, self_induced_x: {air: 0, ground: 0}}, hitlag_remaining: 0, animation_index: 295}}, follower: None}], start: {random_seed: 8100584, scene_frame_counter: 123}, end: {latest_finalized_frame: 0}, items: []}]",
		format!("{:?}", frames.slice(123, 1))
	);
}
