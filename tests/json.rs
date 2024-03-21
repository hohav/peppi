use pretty_assertions::assert_eq;
use serde_json::{from_str, json, to_string, Value};

mod common;
use common::game;

#[test]
fn json_metadata() {
	let game = game("v3.12");
	assert_eq!(
		from_str::<Value>(&to_string(&game.metadata).unwrap()).unwrap(),
		json!({
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
		})
	)
}

#[test]
fn json_start() {
	let game = game("v3.12");
	assert_eq!(
		from_str::<Value>(&to_string(&game.start).unwrap()).unwrap(),
		json!({
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
					"type":"Human",
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
						"dash_back":"Ucf",
						"shield_drop":"Ucf"
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
					"type":"Human",
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
						"dash_back":"Ucf",
						"shield_drop":"Ucf"
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
			"language":"English"
		})
	);
}

#[test]
fn json_end() {
	let game = game("v3.12");
	assert_eq!(
		from_str::<Value>(&to_string(&game.end).unwrap()).unwrap(),
		json!({
			"method":"NoContest",
			"lras_initiator":"P2"
		})
	);
}
