use std::fmt;
use std::collections::HashMap;
use super::ubjson;

#[derive(Debug)]
pub struct Slippi {
	pub version:(u8, u8, u8),
}

#[derive(Debug)]
pub struct Player {
	pub character:u8,
	pub r#type:u8,
	pub stocks:u8,
	pub costume:u8,
	pub team_shade:u8,
	pub handicap:u8,
	pub team:u8,
	pub bitfield:u8,
	pub cpu_level:u8,
	pub offense_ratio:f32,
	pub defense_ratio:f32,
	pub model_scale:f32,
}

#[derive(Debug)]
pub struct GameStart {
	pub slippi:Slippi,
	pub is_teams:bool,
	pub item_spawn_frequency:i8,
	pub self_destruct_score:i8,
	pub stage:u16,
	pub game_timer:u32,
	pub item_spawn_bitfield:[u8; 5],
	pub damage_ratio:f32,
	pub players:[Option<Player>; 4],
}

#[derive(Debug)]
pub struct GameEnd {
	pub method:u8,
	pub lras_initiator:i8,
}

#[derive(Debug)]
pub struct FramePre {
	pub position_x:f32,
	pub position_y:f32,
	pub direction:f32,
	pub joystick_x:f32,
	pub joystick_y:f32,
	pub cstick_x:f32,
	pub cstick_y:f32,
	pub trigger_logical:f32,
	pub trigger_physical_l:f32,
	pub trigger_physical_r:f32,
	pub random_seed:u32,
	pub buttons_logical:u32,
	pub buttons_physical:u16,
	pub state:u16,

	pub v1_2:Option<FramePreV1_2>,
	pub v1_4:Option<FramePreV1_4>,
}

#[derive(Debug)]
pub struct FramePreV1_2 {
	pub raw_analog_x:u8,
}

#[derive(Debug)]
pub struct FramePreV1_4 {
	pub damage:f32,
}

#[derive(Debug)]
pub struct FramePost {
	pub position_x:f32,
	pub position_y:f32,
	pub direction:f32,
	pub damage:f32,
	pub shield:f32,
	pub state:u16,
	pub character:u8,
	pub last_attack_landed:u8,
	pub combo_count:u8,
	pub last_hit_by:u8,
	pub stocks:u8,

	pub v0_2:Option<FramePostV0_2>,
	pub v2_0:Option<FramePostV2_0>,
	pub v2_1:Option<FramePostV2_1>,
}

#[derive(Debug)]
pub struct FramePostV0_2 {
	pub state_age:f32,
}

#[derive(Debug)]
pub struct FramePostV2_0 {
	pub misc_as:f32,
	pub ground:u16,
	pub jumps:u8,
	pub l_cancel:u8,
	pub airborne:bool,
	pub flags:[u8; 5],
}

#[derive(Debug)]
pub struct FramePostV2_1 {
	pub hurtbox_state:u8,
}

pub struct Frames {
	pub pre:Vec<FramePre>,
	pub post:Vec<FramePost>,
}

impl fmt::Debug for Frames {
	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Frames {{ pre: [...({})...], post: [...({})...] }}", self.pre.len(), self.post.len())
	}
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
