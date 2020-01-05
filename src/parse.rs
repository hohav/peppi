use std::io::{Read, Seek, Result, Error, ErrorKind};

use std::collections::HashMap;

extern crate byteorder;
use byteorder::{BigEndian, ReadBytesExt};

extern crate num_enum;
use std::convert::TryFrom;

use super::game::{Frames, FramePre, FramePreV1_2, FramePreV1_4, FramePost, FramePostV0_2, FramePostV2_0, FramePostV2_1, Game, GameStart, GameEnd, Player, Port, Slippi};

use super::ubjson;

const FIRST_FRAME_INDEX:i32 = -123;

#[derive(Debug, PartialEq, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum Event {
    Payloads = 0x35,
    GameStart = 0x36,
    FramePre = 0x37,
    FramePost = 0x38,
    GameEnd = 0x39,
}

#[derive(Debug)]
struct FrameId {
	index:i32,
	port:u8,
	is_follower:bool,
}

#[derive(Debug)]
struct FrameEvent<F> {
	id:FrameId,
	event:F,
}

#[derive(Debug)]
struct GameParser {
	start:Option<GameStart>,
	end:Option<GameEnd>,
	ports:[Option<Port>; 4],
	metadata:Option<HashMap<String, ubjson::Object>>,
}

macro_rules! err {
	($( $arg:expr ),*) => {
		Error::new(ErrorKind::InvalidData, format!($( $arg ),*))
	}
}

fn parse_event_payloads<R:Read>(r:&mut R) -> Result<HashMap<u8, u16>> {
	let code = r.read_u8()?;
	assert_eq!(code, Event::Payloads as u8);

	let payload_size = r.read_u8()?;
	// includes size byte for some reason
	assert!(payload_size % 3 == 1);

	let mut payload_sizes = HashMap::new();
	for _ in (0..(payload_size - 1)).step_by(3) {
		payload_sizes.insert(r.read_u8()?, r.read_u16::<BigEndian>()?);
	}

	Ok(payload_sizes)
}

fn parse_player<R:Read>(mut r:R) -> Result<Option<Player>> {
	let character = r.read_u8()?;
	let r#type = r.read_u8()?;
	let stocks = r.read_u8()?;
	let costume = r.read_u8()?;
	r.read_exact(&mut [0; 3])?; // ???
	let team_shade = r.read_u8()?;
	let handicap = r.read_u8()?;
	let team = r.read_u8()?;
	r.read_u16::<BigEndian>()?; // ???
	let bitfield = r.read_u8()?;
	r.read_u16::<BigEndian>()?; // ???
	let cpu_level = r.read_u8()?;
	r.read_u32::<BigEndian>()?; // ???
	let offense_ratio = r.read_f32::<BigEndian>()?;
	let defense_ratio = r.read_f32::<BigEndian>()?;
	let model_scale = r.read_f32::<BigEndian>()?;
	r.read_u32::<BigEndian>()?; // ???
	Ok(match r#type {
		0 | 1 | 2 => Some(Player {
			character: character,
			r#type: r#type,
			stocks: stocks,
			costume: costume,
			team_shade: team_shade,
			handicap: handicap,
			team: team,
			bitfield: bitfield,
			cpu_level: cpu_level,
			offense_ratio: offense_ratio,
			defense_ratio: defense_ratio,
			model_scale: model_scale,
		}),
		_ => None
	})
}

fn parse_game_start<R:Read>(mut r:R) -> Result<GameStart> {
	let slippi = Slippi {version: (r.read_u8()?, r.read_u8()?, r.read_u8()?)};
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
	let stage = r.read_u16::<BigEndian>()?;
	let game_timer = r.read_u32::<BigEndian>()?;
	r.read_exact(&mut [0; 15])?; // ???
	let item_spawn_bitfield = {
		let mut buf = [0; 5];
		r.read_exact(&mut buf)?;
		buf
	};
	r.read_u64::<BigEndian>()?; // ???
	let damage_ratio = r.read_f32::<BigEndian>()?;
	r.read_exact(&mut [0; 44])?;
	let players = [parse_player(&mut r)?, parse_player(&mut r)?, parse_player(&mut r)?, parse_player(&mut r)?];
	Ok(GameStart {
		slippi: slippi,
		is_teams: is_teams,
		item_spawn_frequency: item_spawn_frequency,
		self_destruct_score: self_destruct_score,
		stage: stage,
		game_timer: game_timer,
		item_spawn_bitfield: item_spawn_bitfield,
		damage_ratio: damage_ratio,
		players: players,
	})
}

fn parse_game_end<R:Read>(mut r:R) -> Result<GameEnd> {
	Ok(GameEnd {
		method: r.read_u8()?,
		lras_initiator: r.read_i8()?,
	})
}

fn parse_frame_pre(mut r:&[u8]) -> Result<FrameEvent<FramePre>> {
	Ok(FrameEvent {
		id: FrameId {
			index: r.read_i32::<BigEndian>()?,
			port: r.read_u8()?,
			is_follower: r.read_u8()? != 0,
		},
		event: FramePre {
			random_seed: r.read_u32::<BigEndian>()?,
			state: r.read_u16::<BigEndian>()?,
			position_x: r.read_f32::<BigEndian>()?,
			position_y: r.read_f32::<BigEndian>()?,
			direction: r.read_f32::<BigEndian>()?,
			joystick_x: r.read_f32::<BigEndian>()?,
			joystick_y: r.read_f32::<BigEndian>()?,
			cstick_x: r.read_f32::<BigEndian>()?,
			cstick_y: r.read_f32::<BigEndian>()?,
			trigger_logical: r.read_f32::<BigEndian>()?,
			buttons_logical: r.read_u32::<BigEndian>()?,
			buttons_physical: r.read_u16::<BigEndian>()?,
			trigger_physical_l: r.read_f32::<BigEndian>()?,
			trigger_physical_r: r.read_f32::<BigEndian>()?,
			v1_2: if r.is_empty() {
				None
			} else {
				Some(FramePreV1_2 {
					raw_analog_x: r.read_u8()?,
				})
			},
			v1_4: if r.is_empty() {
				None
			} else {
				Some(FramePreV1_4 {
					damage: r.read_f32::<BigEndian>()?,
				})
			},
		}
	})
}

fn parse_frame_post(mut r:&[u8]) -> Result<FrameEvent<FramePost>> {
	Ok(FrameEvent {
		id: FrameId {
			index: r.read_i32::<BigEndian>()?,
			port: r.read_u8()?,
			is_follower: r.read_u8()? != 0,
		},
		event: FramePost {
			character: r.read_u8()?,
			state: r.read_u16::<BigEndian>()?,
			position_x: r.read_f32::<BigEndian>()?,
			position_y: r.read_f32::<BigEndian>()?,
			direction: r.read_f32::<BigEndian>()?,
			damage: r.read_f32::<BigEndian>()?,
			shield: r.read_f32::<BigEndian>()?,
			last_attack_landed: r.read_u8()?,
			combo_count: r.read_u8()?,
			last_hit_by: r.read_u8()?,
			stocks: r.read_u8()?,
			v0_2: if r.is_empty() {
				None
			} else {
				Some(FramePostV0_2 {
					state_age: r.read_f32::<BigEndian>()?,
				})
			},
			v2_0: if r.is_empty() {
				None
			} else {
				Some(FramePostV2_0 {
					misc_as: r.read_f32::<BigEndian>()?,
					ground: r.read_u16::<BigEndian>()?,
					jumps: r.read_u8()?,
					l_cancel: r.read_u8()?,
					airborne: r.read_u8()? != 0,
					flags:{let mut buf = [0; 5]; r.read_exact(&mut buf)?; buf},
				})
			},
			v2_1: if r.is_empty() {
				None
			} else {
				Some(FramePostV2_1 {
					hurtbox_state: r.read_u8()?,
				})
			},
		}
	})
}

trait Handlers {
	fn game_start(&mut self, _:GameStart) {}
	fn game_end(&mut self, _:GameEnd) {}
	fn frame_pre(&mut self, _:FrameEvent<FramePre>) {}
	fn frame_post(&mut self, _:FrameEvent<FramePost>) {}
}

fn expect_bytes<R:Read>(mut r:R, expected:&[u8]) -> Result<()> {
	let mut actual = vec![0; expected.len()];
	r.read_exact(&mut actual)?;
	if expected == actual.as_slice() {
		Ok(())
	} else {
		Err(err!("expected: {:?}, got: {:?}", expected, actual))
	}
}

fn parse_event<R:Read, H:Handlers>(mut r:R, payload_sizes:&HashMap<u8, u16>, handlers:&mut H) -> Result<Option<Event>> {
	let code = r.read_u8()?;
	let size = payload_sizes.get(&code).ok_or_else(|| err!("unknown event: {}", code))?;

	let mut buf = vec![0; *size as usize];
	r.read_exact(&mut *buf)?;

	match Event::try_from(code) {
		Ok(event) => match event {
			Event::Payloads => Err(err!("unexpected event: {}", code)),
			Event::GameStart => {
				handlers.game_start(parse_game_start(&*buf)?);
				Ok(Some(event))
			},
			Event::GameEnd => {
				handlers.game_end(parse_game_end(&*buf)?);
				Ok(Some(event))
			},
			Event::FramePre => {
				handlers.frame_pre(parse_frame_pre(&*buf)?);
				Ok(Some(event))
			},
			Event::FramePost => {
				handlers.frame_post(parse_frame_post(&*buf)?);
				Ok(Some(event))
			},
		},
		Err(_) => Ok(None),
	}
}

impl GameParser {
	fn into_game(self) -> Result<Game> {
		Ok(Game {
			start:self.start.ok_or_else(|| err!("missing start event"))?,
			end:self.end.ok_or_else(|| err!("missing end event"))?,
			ports:self.ports,
			metadata:self.metadata.unwrap_or_default(),
		})
	}
}

impl Handlers for GameParser {
	fn game_start(&mut self, s:GameStart) {
		self.start = Some(s);
	}

	fn game_end(&mut self, s:GameEnd) {
		self.end = Some(s);
	}

	fn frame_pre(&mut self, e:FrameEvent<FramePre>) {
		let id = e.id;

		if self.ports[id.port as usize].is_none() {
			self.ports[id.port as usize] = Some(
				Port {
					leader: Frames {pre:Vec::new(), post:Vec::new()},
					follower: None
				}
			);
		}

		let port = self.ports[id.port as usize].as_mut().unwrap();

		let frames = if id.is_follower {
			&mut port.leader.pre
		} else {
			&mut port.leader.pre
		};

		assert_eq!((id.index - FIRST_FRAME_INDEX) as usize, frames.len());
		frames.push(e.event);
	}

	fn frame_post(&mut self, e:FrameEvent<FramePost>) {
		let id = e.id;

		if self.ports[id.port as usize].is_none() {
			self.ports[id.port as usize] = Some(
				Port {
					leader: Frames {pre:Vec::new(), post:Vec::new()},
					follower: None
				}
			);
		}

		let port = self.ports[id.port as usize].as_mut().unwrap();

		let frames = if id.is_follower {
			&mut port.leader.post
		} else {
			&mut port.leader.post
		};

		assert_eq!((id.index - FIRST_FRAME_INDEX) as usize, frames.len());
		frames.push(e.event);
	}
}

pub fn parse<R:Read + Seek>(mut r:R) -> Result<Game> {
	expect_bytes(r.by_ref(), &[0x7b, 0x55, 0x03, 0x72, 0x61, 0x77, 0x5b, 0x24, 0x55, 0x23, 0x6c])?; // header ("{U\x03raw[$U#l")

	r.read_u32::<BigEndian>()?; // length (currently unused)

	let payload_sizes = parse_event_payloads(&mut r)?;

	let mut game_parser = GameParser {
		start:None,
		end:None,
		ports:[None, None, None, None],
		metadata:None,
	};

	while parse_event(r.by_ref(), &payload_sizes, &mut game_parser)? != Some(Event::GameEnd) {
	}

	expect_bytes(r.by_ref(), &[0x55, 0x08, 0x6d, 0x65, 0x74, 0x61, 0x64, 0x61, 0x74, 0x61, 0x7b])?; // "U\x08metadata{"
	game_parser.metadata = Some(ubjson::parse_map(&mut r)?);

	game_parser.into_game()
}
