use std::cmp::{min};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::{Read, Seek, Result, Error, ErrorKind};

use byteorder::{BigEndian, ReadBytesExt};
use encoding_rs::SHIFT_JIS;
use log::{debug, warn};

use super::action_state;
use super::action_state::{ActionState, Common};
use super::attack::{Attack};
use super::character;
use super::character::{Character};
use super::frame;
use super::frame::{Frames, FramePre, FramePost, Direction, Position};
use super::game;
use super::game::{Game, GameStart, GameEnd, Player, PlayerType, Port, Slippi};
use super::stage;
use super::ubjson;

const FIRST_FRAME_INDEX:i32 = -123;

const ZELDA_TRANSFORM_FRAME:u32 = 43;
const SHEIK_TRANSFORM_FRAME:u32 = 36;

const DEFAULT_CHAR_STATE:CharState = CharState {
	character: Character { value: 255 },
	state: ActionState::Common(Common::WAIT),
	age: 0
};

#[derive(Debug, PartialEq, Copy, Clone)]
struct CharState {
	character: Character,
	state: ActionState,
	age: u32,
}

static mut LAST_CHAR_STATES:[CharState; 4] = [
	DEFAULT_CHAR_STATE,
	DEFAULT_CHAR_STATE,
	DEFAULT_CHAR_STATE,
	DEFAULT_CHAR_STATE,
];

#[derive(Debug, PartialEq, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum Event {
	Payloads = 0x35,
	GameStart = 0x36,
	FramePre = 0x37,
	FramePost = 0x38,
	GameEnd = 0x39,
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct FrameId {
	index: i32,
	port: u8,
	is_follower: bool,
}

#[derive(Debug)]
struct FrameEvent<F> {
	id: FrameId,
	event: F,
}

#[derive(Debug)]
struct GameParser {
	start: Option<GameStart>,
	end: Option<GameEnd>,
	ports: [Option<Port>; 4],
	metadata: Option<HashMap<String, ubjson::Object>>,
}

macro_rules! err {
	($( $arg:expr ),*) => {
		Error::new(ErrorKind::InvalidData, format!($( $arg ),*))
	}
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

fn parse_player<R:Read>(mut r:R) -> Result<Option<Player>> {
	let character = character::CSSCharacter { value: r.read_u8()? };
	let r#type = game::PlayerType { value: r.read_u8()? };
	let stocks = r.read_u8()?;
	let costume = r.read_u8()?;
	r.read_exact(&mut [0; 3])?; // ???
	let team_shade = game::TeamShade { value: r.read_u8()? };
	let handicap = r.read_u8()?;
	let team = game::Team { value: r.read_u8()? };
	r.read_u16::<BigEndian>()?; // ???
	let bitfield = r.read_u8()?;
	r.read_u16::<BigEndian>()?; // ???
	let cpu_level = r.read_u8()?;
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
			team_shade: team_shade,
			handicap: handicap,
			team: team,
			bitfield: bitfield,
			cpu_level: cpu_level,
			offense_ratio: offense_ratio,
			defense_ratio: defense_ratio,
			model_scale: model_scale,
			dash_back: None,
			shield_drop: None,
			name_tag: None,
		}),
		_ => None
	})
}

fn parse_game_start(mut r:&[u8]) -> Result<GameStart> {
	let slippi = Slippi {version: (r.read_u8()?, r.read_u8()?, r.read_u8()?)};
	debug!("GameStart: {:?}", slippi);

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
	let stage = stage::Stage { value: r.read_u16::<BigEndian>()? };
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
	let mut players = [parse_player(&mut r)?, parse_player(&mut r)?, parse_player(&mut r)?, parse_player(&mut r)?];
	// @0xf5
	r.read_exact(&mut [0; 72])?; // ???
	// @0x13d
	let random_seed = r.read_u32::<BigEndian>()?;

	let mut is_pal = None;
	let mut is_frozen_ps = None;
	if !r.is_empty() { // v1.0
		for p in &mut players {
			let dash_back = Some(game::DashBack { value: r.read_u32::<BigEndian>()? });
			let shield_drop = Some(game::ShieldDrop { value: r.read_u32::<BigEndian>()? });
			if let Some(p) = p {
				p.dash_back = dash_back;
				p.shield_drop = shield_drop;
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
		random_seed: random_seed,
		is_pal: is_pal,
		is_frozen_ps: is_frozen_ps,
	})
}

fn parse_game_end<R:Read>(mut r:R) -> Result<GameEnd> {
	debug!("GameEnd");
	Ok(GameEnd {
		method: r.read_u8()?,
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

fn predict_character(id:FrameId) -> Character {
	let prev = unsafe { LAST_CHAR_STATES[id.port as usize] };
	match prev.state {
		ActionState::Zelda(action_state::Zelda::TRANSFORM_GROUND) |
		ActionState::Zelda(action_state::Zelda::TRANSFORM_AIR)
			if prev.age >= ZELDA_TRANSFORM_FRAME => Character::SHEIK,
		ActionState::Sheik(action_state::Sheik::TRANSFORM_GROUND) |
		ActionState::Sheik(action_state::Sheik::TRANSFORM_AIR)
			if prev.age >= SHEIK_TRANSFORM_FRAME => Character::ZELDA,
		_ => prev.character,
	}
}

fn parse_frame_pre(mut r:&[u8]) -> Result<FrameEvent<FramePre>> {
	let id = FrameId {
		index: r.read_i32::<BigEndian>()?,
		port: r.read_u8()?,
		is_follower: r.read_u8()? != 0,
	};
	debug!("FramePre: {:?}", id);

	// We need to know the character to interpret the action state properly, but for Sheik/Zelda we
	// don't know whether they transformed this frame untilwe get the corresponding FramePost
	// event. So we predict based on whether we were on the last frame of `TRANSFORM_AIR` or
	// `TRANSFORM_GROUND` during the *previous* frame.
	let character = predict_character(id);

	let random_seed = r.read_u32::<BigEndian>()?;
	let state = ActionState::from(r.read_u16::<BigEndian>()?, character);

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
		logical: frame::ButtonsLogical { value: r.read_u32::<BigEndian>()? },
		physical: frame::ButtonsPhysical { value: r.read_u16::<BigEndian>()? },
	};
	let triggers = frame::Triggers {
		logical: trigger_logical,
		physical: frame::TriggersPhysical {
			l: r.read_f32::<BigEndian>()?,
			r: r.read_f32::<BigEndian>()?,
		},
	};

	// v1.2
	let raw_analog_x = r.read_u8().ok();

	// v1.4
	let damage = r.read_f32::<BigEndian>().ok();

	Ok(FrameEvent {
		id: id,
		event: FramePre {
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

fn flags(buf:&[u8; 5]) -> frame::StateFlags {
	frame::StateFlags { value:
		((buf[0] as u64) << 00) +
		((buf[1] as u64) << 08) +
		((buf[2] as u64) << 16) +
		((buf[3] as u64) << 24) +
		((buf[4] as u64) << 32)
	}
}

unsafe fn update_last_char_state(id:FrameId, character:Character, state:ActionState) {
	let prev = LAST_CHAR_STATES[id.port as usize];

	LAST_CHAR_STATES[id.port as usize] = CharState {
		character: character,
		state: state,
		age: match state {
			s if s == prev.state => prev.age + 1,
			// `TRANSFORM_GROUND` and TRANSFORM_AIR can transition into each other without
			// interrupting the transformation, so treat them the same for age purposes
			ActionState::Zelda(action_state::Zelda::TRANSFORM_GROUND) =>
				match prev.state {
					ActionState::Zelda(action_state::Zelda::TRANSFORM_AIR) =>
						// If you land on the frame where you would have transitioned from
						// `TRANSFORM_AIR` to `TRANSFORM_AIR_ENDING`, you instead transition to
						// `TRANSFORM_GROUND` for one frame before going to
						// `TRANSFORM_GROUND_ENDING` on the next frame. This delays the character
						// switch by one frame, so we cap `age` at its previous value so as not to
						// confuse `predict_character`.
						min(ZELDA_TRANSFORM_FRAME - 1, prev.age + 1),
					_ => 0,
				},
			ActionState::Zelda(action_state::Zelda::TRANSFORM_AIR) =>
				match prev.state {
					ActionState::Zelda(action_state::Zelda::TRANSFORM_GROUND) =>
						min(ZELDA_TRANSFORM_FRAME - 1, prev.age + 1),
					_ => 0,
				},
			ActionState::Sheik(action_state::Sheik::TRANSFORM_GROUND) =>
				match prev.state {
					ActionState::Sheik(action_state::Sheik::TRANSFORM_AIR) =>
						min(SHEIK_TRANSFORM_FRAME - 1, prev.age + 1),
					_ => 0,
				},
			ActionState::Sheik(action_state::Sheik::TRANSFORM_AIR) =>
				match prev.state {
					ActionState::Sheik(action_state::Sheik::TRANSFORM_GROUND) =>
						min(SHEIK_TRANSFORM_FRAME - 1, prev.age + 1),
					_ => 0,
				},
			_ => 0,
		},
	};
}

fn parse_frame_post(mut r:&[u8]) -> Result<FrameEvent<FramePost>> {
	let id = FrameId {
		index: r.read_i32::<BigEndian>()?,
		port: r.read_u8()?,
		is_follower: r.read_u8()? != 0,
	};
	debug!("FramePost: {:?}", id);

	let character = Character { value: r.read_u8()? };
	let state = ActionState::from(r.read_u16::<BigEndian>()?, character);
	let position = Position {
		x: r.read_f32::<BigEndian>()?,
		y: r.read_f32::<BigEndian>()?,
	};
	let direction = direction(r.read_f32::<BigEndian>()?)?;
	let damage = r.read_f32::<BigEndian>()?;
	let shield = r.read_f32::<BigEndian>()?;
	let last_attack_landed = Attack { value: r.read_u8()? };
	let combo_count = r.read_u8()?;
	let last_hit_by = r.read_u8()?;
	let stocks = r.read_u8()?;

	// v0.2
	let state_age = r.read_f32::<BigEndian>().ok();

	// v2.0
	let flags = {
		let mut buf = [0; 5];
		r.read_exact(&mut buf).ok().map(|_| flags(&buf))
	};
	let misc_as = r.read_f32::<BigEndian>().ok();
	let ground = r.read_u16::<BigEndian>().ok();
	let jumps = r.read_u8().ok();
	let l_cancel = r.read_u8().ok().map(|x| frame::LCancel { value: x });
	let airborne = r.read_u8().ok().map(|x| x != 0);

	// v2.1
	let hurtbox_state =  r.read_u8().ok().map(|x| frame::HurtboxState { value: x });

	unsafe {
		update_last_char_state(id, character, state);
	}

	Ok(FrameEvent {
		id: id,
		event: FramePost {
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
			start: self.start.ok_or_else(|| err!("missing start event"))?,
			end: self.end.ok_or_else(|| err!("missing end event"))?,
			ports: self.ports,
			metadata: self.metadata.unwrap_or_default(),
		})
	}
}

fn warn_ooo_frames<F:frame::Indexed>(cur:&F, prev:Option<&F>) {
	if let Some(prev) = prev {
		if prev.index() > cur.index() {
			warn!("out-of-order frame: {} -> {}", prev.index(), cur.index());
		}
	} else {
		if cur.index() != FIRST_FRAME_INDEX {
			warn!("unexpected first frame index: {}", cur.index());
		}
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
			self.ports[id.port as usize] = Some(Port {
				leader: Frames { pre: Vec::new(), post: Vec::new() },
				follower: None,
			});
		}

		let port = self.ports[id.port as usize].as_mut().unwrap();

		let frames = if id.is_follower {
			if port.follower.is_none() {
				port.follower = Some(Frames { pre: Vec::new(), post: Vec::new() });
			}
			&mut port.follower.as_mut().unwrap().pre
		} else {
			&mut port.leader.pre
		};

		warn_ooo_frames(&e.event, frames.last());
		frames.push(e.event);
	}

	fn frame_post(&mut self, e:FrameEvent<FramePost>) {
		let id = e.id;

		if self.ports[id.port as usize].is_none() {
			self.ports[id.port as usize] = Some(
				Port {
					leader: Frames { pre: Vec::new(), post: Vec::new() },
					follower: None
				}
			);
		}

		let port = self.ports[id.port as usize].as_mut().unwrap();

		let frames = if id.is_follower {
			if port.follower.is_none() {
				port.follower = Some(Frames { pre: Vec::new(), post: Vec::new() });
			}
			&mut port.follower.as_mut().unwrap().post
		} else {
			&mut port.leader.post
		};

		warn_ooo_frames(&e.event, frames.last());
		frames.push(e.event);
	}
}

pub fn parse<R:Read + Seek>(mut r:R) -> Result<Game> {
	// header ("{U\x03raw[$U#l")
	expect_bytes(r.by_ref(), &[0x7b, 0x55, 0x03, 0x72, 0x61, 0x77, 0x5b, 0x24, 0x55, 0x23, 0x6c])?;

	r.read_u32::<BigEndian>()?; // length of "raw" element (currently unused)

	let payload_sizes = parse_event_payloads(&mut r)?;

	let mut game_parser = GameParser {
		start: None,
		end: None,
		ports: [None, None, None, None],
		metadata: None,
	};

	unsafe {
		LAST_CHAR_STATES = [
			DEFAULT_CHAR_STATE,
			DEFAULT_CHAR_STATE,
			DEFAULT_CHAR_STATE,
			DEFAULT_CHAR_STATE,
		];
	}

	while parse_event(r.by_ref(), &payload_sizes, &mut game_parser)? != Some(Event::GameEnd) {
	}

	// metadata key & start ("U\x08metadata{")
	expect_bytes(r.by_ref(), &[0x55, 0x08, 0x6d, 0x65, 0x74, 0x61, 0x64, 0x61, 0x74, 0x61, 0x7b])?;

	game_parser.metadata = Some(ubjson::parse_map(&mut r)?);

	// closing UBJSON brace & end of file ("}")
	expect_bytes(r.by_ref(), &[0x7d])?;

	game_parser.into_game()
}
