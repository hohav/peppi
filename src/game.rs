use std::collections::HashMap;

use super::character::{CSSCharacter};
use super::frame::{Frames};
use super::stage::{Stage};

use super::pseudo_enum;
use super::ubjson;

#[derive(Debug)]
pub struct Slippi {
	pub version:(u8, u8, u8),
}

pseudo_enum!(PlayerType:u8 {
	0 => HUMAN,
	1 => CPU,
	2 => DEMO,
});

pseudo_enum!(Team:u8 {
	0 => RED,
	1 => BLUE,
	2 => GREEN,
});

pseudo_enum!(TeamShade:u8 {
	0 => NORMAL,
	1 => LIGHT,
	2 => DARK,
});

pseudo_enum!(DashBack:u32 {
	0 => NONE,
	1 => UCF,
	2 => ARDUINO,
});

pseudo_enum!(ShieldDrop:u32 {
	0 => NONE,
	1 => UCF,
	2 => ARDUINO,
});

#[derive(Debug)]
pub struct Player {
	pub character:CSSCharacter,
	pub r#type:PlayerType,
	pub stocks:u8,
	pub costume:u8,
	pub team_shade:TeamShade,
	pub handicap:u8,
	pub team:Team,
	pub bitfield:u8,
	pub cpu_level:u8,
	pub offense_ratio:f32,
	pub defense_ratio:f32,
	pub model_scale:f32,
	pub dash_back:Option<DashBack>,
	pub shield_drop:Option<ShieldDrop>,
	pub name_tag:Option<String>,
}

#[derive(Debug)]
pub struct GameStart {
	pub slippi:Slippi,
	pub is_teams:bool,
	pub item_spawn_frequency:i8,
	pub self_destruct_score:i8,
	pub stage:Stage,
	pub game_timer:u32,
	pub item_spawn_bitfield:[u8; 5],
	pub damage_ratio:f32,
	pub players:[Option<Player>; 4],
	pub random_seed:u32,
	pub is_pal:Option<bool>,
	pub is_frozen_ps:Option<bool>,
}

#[derive(Debug)]
pub struct GameEnd {
	pub method:u8,
	pub lras_initiator:i8,
}

#[derive(Debug)]
pub struct Port {
	pub leader:Frames,
	pub follower:Option<Frames>,
}

#[derive(Debug)]
pub struct Game {
	pub start:GameStart,
	pub end:GameEnd,
	pub ports:[Option<Port>; 4],
	pub metadata:HashMap<String, ubjson::Object>,
}
