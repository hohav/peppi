use std::cmp::{min};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::{Read, Seek, Result};

use byteorder::{BigEndian, ReadBytesExt};
use encoding_rs::SHIFT_JIS;
use log::{debug};

use super::{action_state, buttons, character, frame, game, stage, triggers, ubjson};
use super::action_state::{Common, State};
use super::attack::{Attack};
use super::character::{Internal};
use super::frame::{Pre, Post, Direction, Position};
use super::game::{Start, End, Player, PlayerType, Slippi, NUM_PORTS};

const ZELDA_TRANSFORM_FRAME:u32 = 43;
const SHEIK_TRANSFORM_FRAME:u32 = 36;

// We only track this for Sheik/Zelda transformations, which can't happen on
// the first frame. So we can initialize with any arbitrary character value.
const DEFAULT_CHAR_STATE:CharState = CharState {
	character: Internal(255),
	state: State::Common(Common::WAIT),
	age: 0
};

#[derive(Copy, Clone, Debug, PartialEq)]
struct CharState {
	character: Internal,
	state: State,
	age: u32,
}

#[derive(Debug, PartialEq, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum Event {
	Payloads = 0x35,
	GameStart = 0x36,
	FramePre = 0x37,
	FramePost = 0x38,
	GameEnd = 0x39,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FrameId {
	pub index: i32,
	pub port: u8,
	pub is_follower: bool,
}

#[derive(Debug)]
pub struct FrameEvent<F> {
	pub id: FrameId,
	pub event: F,
}

fn parse_event_payloads<R:Read>(r:&mut R) -> Result<HashMap<u8, u16>> {
	let code = r.read_u8()?;
	if code != Event::Payloads as u8 {
		Err(err!("expected event payloads, but got: {}", code))?;
	}

	let payload_size = r.read_u8()?; // includes size byte for some reason
	if payload_size % 3 != 1 {
		Err(err!("invalid payload size: {}", payload_size))?;
	}

	let mut payload_sizes = HashMap::new();
	for _ in (0..(payload_size - 1)).step_by(3) {
		payload_sizes.insert(r.read_u8()?, r.read_u16::<BigEndian>()?);
	}

	Ok(payload_sizes)
}

fn parse_player<R:Read>(r:&mut R, is_teams:bool) -> Result<Option<Player>> {
	let character = character::External(r.read_u8()?);
	let r#type = game::PlayerType(r.read_u8()?);
	let stocks = r.read_u8()?;
	let costume = r.read_u8()?;
	r.read_exact(&mut [0; 3])?; // ???
	let team_shade = r.read_u8()?;
	let handicap = r.read_u8()?;
	let team_color = r.read_u8()?;
	let team = {
		match is_teams {
			true => Some(game::Team {
				color: game::TeamColor(team_color),
				shade: game::TeamShade(team_shade),
			}),
			false => None,
		}
	};
	r.read_u16::<BigEndian>()?; // ???
	let bitfield = r.read_u8()?;
	r.read_u16::<BigEndian>()?; // ???
	let cpu_level = {
		let cpu_level = r.read_u8()?;
		match r#type {
			PlayerType::CPU => Some(cpu_level),
			_ => None,
		}
	};
	r.read_u32::<BigEndian>()?; // ???
	let offense_ratio = r.read_f32::<BigEndian>()?;
	let defense_ratio = r.read_f32::<BigEndian>()?;
	let model_scale = r.read_f32::<BigEndian>()?;
	r.read_u32::<BigEndian>()?; // ???
	// total bytes: 0x24
	Ok(match r#type {
		PlayerType::HUMAN | PlayerType::CPU | PlayerType::DEMO => Some(Player {
			character: character,
			r#type: r#type,
			stocks: stocks,
			costume: costume,
			team: team,
			handicap: handicap,
			bitfield: bitfield,
			cpu_level: cpu_level,
			offense_ratio: offense_ratio,
			defense_ratio: defense_ratio,
			model_scale: model_scale,
			ucf: None,
			name_tag: None,
		}),
		_ => None
	})
}

fn parse_game_start(mut r:&[u8]) -> Result<Start> {
	let slippi = Slippi {version: (r.read_u8()?, r.read_u8()?, r.read_u8()?)};
	debug!("game::Start: {:?}", slippi);

	r.read_u8()?; // unused (build number)
	r.read_u8()?; // bitfield 1
	r.read_u8()?; // bitfield 2
	r.read_u8()?; // ???
	r.read_u8()?; // bitfield 3
	r.read_u32::<BigEndian>()?; // ???
	let is_teams = r.read_u8()? != 0;
	r.read_u16::<BigEndian>()?; // ???
	let item_spawn_frequency = r.read_i8()?;
	let self_destruct_score = r.read_i8()?;
	r.read_u8()?; // ???
	let stage = stage::Stage(r.read_u16::<BigEndian>()?);
	let game_timer = r.read_u32::<BigEndian>()?;
	r.read_exact(&mut [0; 15])?; // ???
	let item_spawn_bitfield = {
		let mut buf = [0; 5];
		r.read_exact(&mut buf)?;
		buf
	};
	r.read_u64::<BigEndian>()?; // ???
	let damage_ratio = r.read_f32::<BigEndian>()?;
	r.read_exact(&mut [0; 44])?; // ???
	// @0x65
	let mut players = [parse_player(&mut r, is_teams)?, parse_player(&mut r, is_teams)?, parse_player(&mut r, is_teams)?, parse_player(&mut r, is_teams)?];
	// @0xf5
	r.read_exact(&mut [0; 72])?; // ???
	// @0x13d
	let random_seed = r.read_u32::<BigEndian>()?;

	let mut is_pal = None;
	let mut is_frozen_ps = None;
	if !r.is_empty() { // v1.0
		for p in &mut players {
			let dash_back = r.read_u32::<BigEndian>()?;
			let shield_drop = r.read_u32::<BigEndian>()?;
			if let Some(p) = p {
				p.ucf = Some(game::Ucf {
					dash_back: match dash_back {
						0 => None,
						dash_back => Some(game::DashBack(dash_back)),
					},
					shield_drop: match shield_drop {
						0 => None,
						shield_drop => Some(game::ShieldDrop(shield_drop)),
					},
				});
			}
		}
		if !r.is_empty() { // v1.3
			for p in &mut players {
				let mut name_tag = [0; 16];
				r.read_exact(&mut name_tag)?;
				if let Some(p) = p {
					let first_null = name_tag.iter().position(|&x| x == 0).unwrap_or(16);
					let (name_tag, _) = SHIFT_JIS.decode_without_bom_handling(&name_tag[0..first_null]);
					p.name_tag = Some(name_tag.to_string());
				}
			}
			if !r.is_empty() { // v1.5
				is_pal = Some(r.read_u8()? != 0);
				if !r.is_empty() { // v2.0
					is_frozen_ps = Some(r.read_u8()? != 0);
				}
			}
		}
	}

	Ok(Start {
		slippi: slippi,
		is_teams: is_teams,
		item_spawn_frequency: item_spawn_frequency,
		self_destruct_score: self_destruct_score,
		stage: stage,
		game_timer: game_timer,
		item_spawn_bitfield: item_spawn_bitfield,
		damage_ratio: damage_ratio,
		players: players,
		random_seed: random_seed,
		is_pal: is_pal,
		is_frozen_ps: is_frozen_ps,
	})
}

fn parse_game_end<R:Read>(mut r:R) -> Result<End> {
	debug!("game::End");
	Ok(End {
		method: game::EndMethod(r.read_u8()?),
		lras_initiator: r.read_i8().ok(),
	})
}

fn direction(value:f32) -> Result<Direction> {
	match value {
		v if v < 0.0 => Ok(Direction::LEFT),
		v if v > 0.0 => Ok(Direction::RIGHT),
		_ => Err(err!("direction == 0")),
	}
}

fn predict_character(id:FrameId, last_char_states:&[CharState; NUM_PORTS]) -> Internal {
	let prev = last_char_states[id.port as usize];
	match prev.state {
		State::Zelda(action_state::Zelda::TRANSFORM_GROUND) |
		State::Zelda(action_state::Zelda::TRANSFORM_AIR)
			if prev.age >= ZELDA_TRANSFORM_FRAME => Internal::SHEIK,
		State::Sheik(action_state::Sheik::TRANSFORM_GROUND) |
		State::Sheik(action_state::Sheik::TRANSFORM_AIR)
			if prev.age >= SHEIK_TRANSFORM_FRAME => Internal::ZELDA,
		_ => prev.character,
	}
}

fn parse_frame_pre(mut r:&[u8], last_char_states:&[CharState; NUM_PORTS]) -> Result<FrameEvent<Pre>> {
	let id = FrameId {
		index: r.read_i32::<BigEndian>()?,
		port: r.read_u8()?,
		is_follower: r.read_u8()? != 0,
	};
	debug!("frame::Pre: {:?}", id);

	// We need to know the character to interpret the action state properly, but for Sheik/Zelda we
	// don't know whether they transformed this frame untilwe get the corresponding frame::Post
	// event. So we predict based on whether we were on the last frame of `TRANSFORM_AIR` or
	// `TRANSFORM_GROUND` during the *previous* frame.
	let character = predict_character(id, last_char_states);

	let random_seed = r.read_u32::<BigEndian>()?;
	let state = State::from(r.read_u16::<BigEndian>()?, character);

	let position = Position {
		x: r.read_f32::<BigEndian>()?,
		y: r.read_f32::<BigEndian>()?,
	};
	let direction = direction(r.read_f32::<BigEndian>()?)?;
	let joystick = Position {
		x: r.read_f32::<BigEndian>()?,
		y: r.read_f32::<BigEndian>()?,
	};
	let cstick = Position {
		x: r.read_f32::<BigEndian>()?,
		y: r.read_f32::<BigEndian>()?,
	};
	let trigger_logical = r.read_f32::<BigEndian>()?;
	let buttons = frame::Buttons {
		logical: buttons::Logical(r.read_u32::<BigEndian>()?),
		physical: buttons::Physical(r.read_u16::<BigEndian>()?),
	};
	let triggers = frame::Triggers {
		logical: trigger_logical,
		physical: triggers::Physical {
			l: r.read_f32::<BigEndian>()?,
			r: r.read_f32::<BigEndian>()?,
		},
	};

	let mut raw_analog_x = None;
	let mut damage = None;

	// v1.2
	if !r.is_empty() {
		raw_analog_x = Some(r.read_u8()?);

		// v1.4
		if !r.is_empty() {
			damage = Some(r.read_f32::<BigEndian>()?);
		}
	}

	Ok(FrameEvent {
		id: id,
		event: Pre {
			index: id.index,
			random_seed: random_seed,
			state: state,
			position: position,
			direction: direction,
			joystick: joystick,
			cstick: cstick,
			triggers: triggers,
			buttons: buttons,
			raw_analog_x: raw_analog_x,
			damage: damage,
		}
	})
}

fn parse_flags(buf:&[u8; 5]) -> frame::StateFlags {
	frame::StateFlags(
		((buf[0] as u64) << 00) +
		((buf[1] as u64) << 08) +
		((buf[2] as u64) << 16) +
		((buf[3] as u64) << 24) +
		((buf[4] as u64) << 32)
	)
}

fn update_last_char_state(id:FrameId, character:Internal, state:State, last_char_states:&mut [CharState; NUM_PORTS]) {
	let prev = last_char_states[id.port as usize];

	last_char_states[id.port as usize] = CharState {
		character: character,
		state: state,
		age: match state {
			s if s == prev.state => prev.age + 1,
			// `TRANSFORM_GROUND` and TRANSFORM_AIR can transition into each other without
			// interrupting the transformation, so treat them the same for age purposes
			State::Zelda(action_state::Zelda::TRANSFORM_GROUND) =>
				match prev.state {
					State::Zelda(action_state::Zelda::TRANSFORM_AIR) =>
						// If you land on the frame where you would have transitioned from
						// `TRANSFORM_AIR` to `TRANSFORM_AIR_ENDING`, you instead transition to
						// `TRANSFORM_GROUND` for one frame before going to
						// `TRANSFORM_GROUND_ENDING` on the next frame. This delays the character
						// switch by one frame, so we cap `age` at its previous value so as not to
						// confuse `predict_character`.
						min(ZELDA_TRANSFORM_FRAME - 1, prev.age + 1),
					_ => 0,
				},
			State::Zelda(action_state::Zelda::TRANSFORM_AIR) =>
				match prev.state {
					State::Zelda(action_state::Zelda::TRANSFORM_GROUND) =>
						min(ZELDA_TRANSFORM_FRAME - 1, prev.age + 1),
					_ => 0,
				},
			State::Sheik(action_state::Sheik::TRANSFORM_GROUND) =>
				match prev.state {
					State::Sheik(action_state::Sheik::TRANSFORM_AIR) =>
						min(SHEIK_TRANSFORM_FRAME - 1, prev.age + 1),
					_ => 0,
				},
			State::Sheik(action_state::Sheik::TRANSFORM_AIR) =>
				match prev.state {
					State::Sheik(action_state::Sheik::TRANSFORM_GROUND) =>
						min(SHEIK_TRANSFORM_FRAME - 1, prev.age + 1),
					_ => 0,
				},
			_ => 0,
		},
	};
}

fn parse_frame_post(mut r:&[u8], last_char_states:&mut [CharState; NUM_PORTS]) -> Result<FrameEvent<Post>> {
	let id = FrameId {
		index: r.read_i32::<BigEndian>()?,
		port: r.read_u8()?,
		is_follower: r.read_u8()? != 0,
	};
	debug!("frame::Post: {:?}", id);

	let character = Internal(r.read_u8()?);
	let state = State::from(r.read_u16::<BigEndian>()?, character);
	let position = Position {
		x: r.read_f32::<BigEndian>()?,
		y: r.read_f32::<BigEndian>()?,
	};
	let direction = direction(r.read_f32::<BigEndian>()?)?;
	let damage = r.read_f32::<BigEndian>()?;
	let shield = r.read_f32::<BigEndian>()?;
	let last_attack_landed = {
		let attack = r.read_u8()?;
		match attack {
			0 => None,
			attack => Some(Attack(attack)),
		}
	};
	let combo_count = r.read_u8()?;
	let last_hit_by = r.read_u8()?;
	let stocks = r.read_u8()?;

	let mut state_age = None;
	let mut flags = None;
	let mut misc_as = None;
	let mut ground = None;
	let mut jumps = None;
	let mut l_cancel = None;
	let mut airborne = None;
	let mut hurtbox_state =  None;

	// v0.2
	if !r.is_empty() {
		state_age = Some(r.read_f32::<BigEndian>()?);

		// v2.0
		if !r.is_empty() {
			flags = {
				let mut buf = [0; 5];
				r.read_exact(&mut buf)?;
				Some(parse_flags(&buf))
			};
			misc_as = Some(r.read_f32::<BigEndian>()?);
			ground = Some(r.read_u16::<BigEndian>()?);
			jumps = Some(r.read_u8()?);
			l_cancel = Some(match r.read_u8()? {
				0 => None,
				l_cancel => Some(frame::LCancel(l_cancel)),
			});
			airborne = Some(r.read_u8()? != 0);

			// v2.1
			if !r.is_empty() {
				hurtbox_state = Some(frame::HurtboxState(r.read_u8()?));
			}
		}
	}

	update_last_char_state(id, character, state, last_char_states);

	Ok(FrameEvent {
		id: id,
		event: Post {
			index: id.index,
			character: character,
			state: state,
			position: position,
			direction: direction,
			damage: damage,
			shield: shield,
			last_attack_landed: last_attack_landed,
			combo_count: combo_count,
			last_hit_by: last_hit_by,
			stocks: stocks,
			state_age: state_age,
			flags: flags,
			misc_as: misc_as,
			ground: ground,
			jumps: jumps,
			l_cancel: l_cancel,
			airborne: airborne,
			hurtbox_state: hurtbox_state,
		},
	})
}

pub trait Handlers {
	fn game_start(&mut self, _:Start) -> Result<()> { Ok(()) }
	fn game_end(&mut self, _:End) -> Result<()> { Ok(()) }
	fn frame_pre(&mut self, _:FrameEvent<Pre>) -> Result<()> { Ok(()) }
	fn frame_post(&mut self, _:FrameEvent<Post>) -> Result<()> { Ok(()) }
	fn metadata(&mut self, _:HashMap<String, ubjson::Object>) -> Result<()> { Ok(()) }
}

fn expect_bytes<R:Read>(r:&mut R, expected:&[u8]) -> Result<()> {
	let mut actual = vec![0; expected.len()];
	r.read_exact(&mut actual)?;
	if expected == actual.as_slice() {
		Ok(())
	} else {
		Err(err!("expected: {:?}, got: {:?}", expected, actual))
	}
}

fn parse_event<R:Read, H:Handlers>(mut r:R, payload_sizes:&HashMap<u8, u16>, last_char_states:&mut [CharState; NUM_PORTS], handlers:&mut H) -> Result<Option<Event>> {
	let code = r.read_u8()?;
	let size = payload_sizes.get(&code).ok_or_else(|| err!("unknown event: {}", code))?;

	let mut buf = vec![0; *size as usize];
	r.read_exact(&mut *buf)?;

	match Event::try_from(code) {
		Ok(event) => match event {
			Event::Payloads => Err(err!("unexpected event: {}", code)),
			Event::GameStart => {
				handlers.game_start(parse_game_start(&*buf)?)?;
				Ok(Some(event))
			},
			Event::GameEnd => {
				handlers.game_end(parse_game_end(&*buf)?)?;
				Ok(Some(event))
			},
			Event::FramePre => {
				handlers.frame_pre(parse_frame_pre(&*buf, last_char_states)?)?;
				Ok(Some(event))
			},
			Event::FramePost => {
				handlers.frame_post(parse_frame_post(&*buf, last_char_states)?)?;
				Ok(Some(event))
			},
		},
		Err(_) => Ok(None),
	}
}

pub fn parse<R:Read + Seek, H:Handlers>(mut r:R, handlers:&mut H) -> Result<()> {
	// header ("{U\x03raw[$U#l")
	expect_bytes(&mut r, &[0x7b, 0x55, 0x03, 0x72, 0x61, 0x77, 0x5b, 0x24, 0x55, 0x23, 0x6c])?;

	r.read_u32::<BigEndian>()?; // length of "raw" element (currently unused)

	let payload_sizes = parse_event_payloads(&mut r)?;
	let mut last_char_states = [DEFAULT_CHAR_STATE; NUM_PORTS];

	while parse_event(r.by_ref(), &payload_sizes, &mut last_char_states, handlers)? != Some(Event::GameEnd) {}

	// metadata key & start ("U\x08metadata{")
	expect_bytes(&mut r, &[0x55, 0x08, 0x6d, 0x65, 0x74, 0x61, 0x64, 0x61, 0x74, 0x61, 0x7b])?;

	handlers.metadata(ubjson::parse_map(&mut r)?)?;

	// closing UBJSON brace & end of file ("}")
	expect_bytes(&mut r, &[0x7d])?;

	Ok(())
}
