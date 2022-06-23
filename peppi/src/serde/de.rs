use std::{
	cmp::min,
	collections::HashMap,
	io::{self, Read, Result},
};

use byteorder::ReadBytesExt;
use encoding_rs::SHIFT_JIS;
use log::{debug, info, trace};
use serde_json;

type BE = byteorder::BigEndian;

use crate::{
	model::{
		buttons,
		enums::{
			action_state::{self, Common, State},
			attack::Attack,
			character::{self, Internal},
			ground, item, stage,
		},
		frame::{self, Post, Pre},
		game::{self, Netplay, Player, PlayerType, MAX_PLAYERS, NUM_PORTS},
		item::Item,
		primitives::{Port, Position, Velocity},
		slippi, triggers,
	},
	ubjson,
};

const ZELDA_TRANSFORM_FRAME: u32 = 43;
const SHEIK_TRANSFORM_FRAME: u32 = 36;

// We only track this for Sheik/Zelda transformations, which can't happen on
// the first frame. So we can initialize with any arbitrary character value.
const DEFAULT_CHAR_STATE: CharState = CharState {
	character: Internal(255),
	state: State::Common(Common::WAIT),
	age: 0,
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct CharState {
	character: Internal,
	state: State,
	age: u32,
}

pub(super) const PAYLOADS_EVENT_CODE: u8 = 0x35;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub(super) enum Event {
	GameStart = 0x36,
	FramePre = 0x37,
	FramePost = 0x38,
	GameEnd = 0x39,
	FrameStart = 0x3A,
	Item = 0x3B,
	FrameEnd = 0x3C,
	GeckoCodes = 0x3D,
}

pub trait Indexed {
	fn index(&self) -> i32;
	fn array_index(&self) -> usize;
}

/// Just a frame index, with no port number.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameId {
	pub index: i32,
}

impl FrameId {
	fn new(index: i32) -> FrameId {
		FrameId { index: index }
	}
}

impl Indexed for FrameId {
	fn index(&self) -> i32 {
		self.index
	}

	fn array_index(&self) -> usize {
		(self.index - game::FIRST_FRAME_INDEX).try_into().unwrap()
	}
}

/// Frame index plus port number and `is_follower` flag (for ICs).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PortId {
	pub index: i32,
	pub port: Port,
	pub is_follower: bool,
}

impl PortId {
	pub fn new(index: i32, port: u8, is_follower: bool) -> Result<PortId> {
		Ok(PortId {
			index: index,
			port: Port::try_from(port).map_err(|e| err!("invalid port: {:?}", e))?,
			is_follower: is_follower,
		})
	}
}

impl Indexed for PortId {
	fn index(&self) -> i32 {
		self.index
	}

	fn array_index(&self) -> usize {
		(self.index - game::FIRST_FRAME_INDEX).try_into().unwrap()
	}
}

/// Wrapper for a frame event. Contains the event ID (`PortId` for per-port events,
/// `FrameId` for other events like items).
#[derive(Debug)]
pub struct FrameEvent<Id, Event> {
	pub id: Id,
	pub event: Event,
}

fn if_more<F, T>(r: &mut &[u8], f: F) -> Result<Option<T>>
where
	F: FnOnce(&mut &[u8]) -> Result<T>,
{
	Ok(match r.is_empty() {
		true => None,
		_ => Some(f(r)?),
	})
}

/// Reads the Event Payloads event, which must come first in the raw stream
/// and tells us the sizes for all other events to follow.
/// Returns the number of bytes read by this function, plus a map of event
/// codes to payload sizes. This map uses raw event codes as keys (as opposed
/// to `Event` enum values) for forwards compatibility, as it allows us to
/// skip unknown event types.
fn payload_sizes<R: Read>(r: &mut R) -> Result<(usize, HashMap<u8, u16>)> {
	let code = r.read_u8()?;
	if code != PAYLOADS_EVENT_CODE {
		return Err(err!("expected event payloads, but got: {}", code));
	}

	// Size in bytes of the subsequent list of payload-size kv pairs.
	// Each pair is 3 bytes, so this size should be divisible by 3.
	// However the value includes this size byte itself, so it's off-by-one.
	let size = r.read_u8()?;
	if size % 3 != 1 {
		return Err(err!("invalid payload size: {}", size));
	}

	let mut sizes = HashMap::new();
	for _ in (0..size - 1).step_by(3) {
		let code = r.read_u8()?;
		let size = r.read_u16::<BE>()?;
		sizes.insert(code, size);
	}

	info!(
		"Event payload sizes: {{{}}}",
		sizes
			.iter()
			.map(|(k, v)| format!("0x{:x}: {}", k, v))
			.collect::<Vec<_>>()
			.join(", ")
	);

	Ok((1 + size as usize, sizes)) // +1 byte for the event code
}

fn player(
	port: Port,
	v0: &[u8; 36],
	is_teams: bool,
	v1_0: Option<[u8; 8]>,
	v1_3: Option<[u8; 16]>,
	v3_9_name: Option<[u8; 31]>,
	v3_9_code: Option<[u8; 10]>,
	v3_11: Option<[u8; 29]>,
) -> Result<Option<Player>> {
	let mut r = &v0[..];
	let mut unmapped = [0; 15];

	let character = character::External(r.read_u8()?);
	let r#type = game::PlayerType(r.read_u8()?);
	let stocks = r.read_u8()?;
	let costume = r.read_u8()?;
	r.read_exact(&mut unmapped[0..3])?;
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
	r.read_exact(&mut unmapped[3..5])?;
	let bitfield = r.read_u8()?;
	r.read_exact(&mut unmapped[5..7])?;
	let cpu_level = {
		let cpu_level = r.read_u8()?;
		match r#type {
			PlayerType::CPU => Some(cpu_level),
			_ => None,
		}
	};
	r.read_exact(&mut unmapped[7..11])?;
	let offense_ratio = r.read_f32::<BE>()?;
	let defense_ratio = r.read_f32::<BE>()?;
	let model_scale = r.read_f32::<BE>()?;
	r.read_exact(&mut unmapped[11..15])?;
	// total bytes: 0x24

	// v1.0
	let ucf = match v1_0 {
		Some(v1_0) => {
			let mut r = &v1_0[..];
			Some(game::Ucf {
				dash_back: match r.read_u32::<BE>()? {
					0 => None,
					db => Some(game::DashBack(db)),
				},
				shield_drop: match r.read_u32::<BE>()? {
					0 => None,
					sd => Some(game::ShieldDrop(sd)),
				},
			})
		}
		_ => None,
	};

	// v1_3
	let name_tag = v1_3.map(|v1_3| {
		let first_null = v1_3.iter().position(|&x| x == 0).unwrap_or(16);
		SHIFT_JIS
			.decode_without_bom_handling(&v1_3[0..first_null])
			.0
			.to_string()
	});

	// v3.9
	let netplay = v3_9_name.zip(v3_9_code).map(|(name, code)| {
		Netplay {
			name: {
				let first_null = name.iter().position(|&x| x == 0).unwrap_or(31);
				SHIFT_JIS
					.decode_without_bom_handling(&name[0..first_null])
					.0
					.to_string()
			},
			code: {
				let first_null = code.iter().position(|&x| x == 0).unwrap_or(10);
				SHIFT_JIS
					.decode_without_bom_handling(&code[0..first_null])
					.0
					.to_string()
			},
			// v3.11
			suid: v3_11.map(|v3_11| {
				let first_null = v3_11.iter().position(|&x| x == 0).unwrap_or(28);
				String::from_utf8_lossy(&v3_11[0..first_null]).to_string()
			}),
		}
	});

	Ok(match r#type {
		PlayerType::HUMAN | PlayerType::CPU | PlayerType::DEMO => Some(Player {
			port: port,
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
			// v1_0
			ucf: ucf,
			// v1_3
			name_tag: name_tag,
			// v3.9
			netplay: netplay,
		}),
		_ => None,
	})
}

fn player_bytes_v3_11(r: &mut &[u8]) -> Result<[u8; 29]> {
	let mut buf = [0; 29];
	r.read_exact(&mut buf)?;
	Ok(buf)
}

fn player_bytes_v3_9_name(r: &mut &[u8]) -> Result<[u8; 31]> {
	let mut buf = [0; 31];
	r.read_exact(&mut buf)?;
	Ok(buf)
}

fn player_bytes_v3_9_code(r: &mut &[u8]) -> Result<[u8; 10]> {
	let mut buf = [0; 10];
	r.read_exact(&mut buf)?;
	Ok(buf)
}

fn player_bytes_v1_3(r: &mut &[u8]) -> Result<[u8; 16]> {
	let mut buf = [0; 16];
	r.read_exact(&mut buf)?;
	Ok(buf)
}

fn player_bytes_v1_0(r: &mut &[u8]) -> Result<[u8; 8]> {
	let mut buf = [0; 8];
	r.read_exact(&mut buf)?;
	Ok(buf)
}

fn game_start(mut r: &mut &[u8]) -> Result<game::Start> {
	let raw_bytes = r.to_vec();
	let slippi = slippi::Slippi {
		version: slippi::Version(r.read_u8()?, r.read_u8()?, r.read_u8()?),
	};
	r.read_u8()?; // unused (build number)

	let mut unmapped = [0; 73];
	let bitfield = {
		let mut buf = [0; 4];
		r.read_exact(&mut buf)?;
		buf
	};
	r.read_exact(&mut unmapped[0..2])?;
	let is_raining_bombs = r.read_u8()? != 0;
	r.read_exact(&mut unmapped[2..3])?;
	let is_teams = r.read_u8()? != 0;
	r.read_exact(&mut unmapped[3..5])?;
	let item_spawn_frequency = r.read_i8()?;
	let self_destruct_score = r.read_i8()?;
	r.read_exact(&mut unmapped[5..6])?;
	let stage = stage::Stage(r.read_u16::<BE>()?);
	let timer = r.read_u32::<BE>()?;
	r.read_exact(&mut unmapped[6..21])?;
	let item_spawn_bitfield = {
		let mut buf = [0; 5];
		r.read_exact(&mut buf)?;
		buf
	};
	r.read_exact(&mut unmapped[21..29])?;
	let damage_ratio = r.read_f32::<BE>()?;
	r.read_exact(&mut unmapped[29..73])?;
	// @0x65
	let mut players_v0 = [[0; 36]; MAX_PLAYERS];
	for p in &mut players_v0 {
		r.read_exact(p)?;
	}
	// @0x13d
	let random_seed = r.read_u32::<BE>()?;

	let players_v1_0 = match r.is_empty() {
		true => [None, None, None, None],
		_ => [
			Some(player_bytes_v1_0(&mut r)?),
			Some(player_bytes_v1_0(&mut r)?),
			Some(player_bytes_v1_0(&mut r)?),
			Some(player_bytes_v1_0(&mut r)?),
		],
	};

	let players_v1_3 = match r.is_empty() {
		true => [None, None, None, None],
		_ => [
			Some(player_bytes_v1_3(&mut r)?),
			Some(player_bytes_v1_3(&mut r)?),
			Some(player_bytes_v1_3(&mut r)?),
			Some(player_bytes_v1_3(&mut r)?),
		],
	};

	let is_pal = if_more(r, |r| Ok(r.read_u8()? != 0))?;
	let is_frozen_ps = if_more(r, |r| Ok(r.read_u8()? != 0))?;
	let scene = if_more(r, |r| {
		Ok(game::Scene {
			minor: r.read_u8()?,
			major: r.read_u8()?,
		})
	})?;

	let players_v3_9 = match r.is_empty() {
		true => ([None, None, None, None], [None, None, None, None]),
		_ => (
			[
				Some(player_bytes_v3_9_name(&mut r)?),
				Some(player_bytes_v3_9_name(&mut r)?),
				Some(player_bytes_v3_9_name(&mut r)?),
				Some(player_bytes_v3_9_name(&mut r)?),
			],
			[
				Some(player_bytes_v3_9_code(&mut r)?),
				Some(player_bytes_v3_9_code(&mut r)?),
				Some(player_bytes_v3_9_code(&mut r)?),
				Some(player_bytes_v3_9_code(&mut r)?),
			],
		),
	};

	let players_v3_11 = match r.is_empty() {
		true => [None, None, None, None],
		_ => [
			Some(player_bytes_v3_11(&mut r)?),
			Some(player_bytes_v3_11(&mut r)?),
			Some(player_bytes_v3_11(&mut r)?),
			Some(player_bytes_v3_11(&mut r)?),
		],
	};

	let mut players = Vec::<Player>::new();
	for n in 0..NUM_PORTS {
		if let Some(p) = player(
			Port::try_from(n as u8).unwrap(),
			&players_v0[n],
			is_teams,
			players_v1_0[n],
			players_v1_3[n],
			players_v3_9.0[n],
			players_v3_9.1[n],
			players_v3_11[n],
		)? {
			players.push(p);
		}
	}

	let lang = if_more(r, |r| Ok(game::Language(r.read_u8()?)))?;

	Ok(game::Start {
		slippi: slippi,
		bitfield: bitfield,
		is_raining_bombs: is_raining_bombs,
		is_teams: is_teams,
		item_spawn_frequency: item_spawn_frequency,
		self_destruct_score: self_destruct_score,
		stage: stage,
		timer: timer,
		item_spawn_bitfield: item_spawn_bitfield,
		damage_ratio: damage_ratio,
		players: players,
		random_seed: random_seed,
		raw_bytes: raw_bytes,
		// v1.5
		is_pal: is_pal,
		// v2.0
		is_frozen_ps: is_frozen_ps,
		// v3.7
		scene: scene,
		// v3.12
		language: lang,
	})
}

fn game_end(r: &mut &[u8]) -> Result<game::End> {
	Ok(game::End {
		method: game::EndMethod(r.read_u8()?),
		// v2.0
		lras_initiator: if_more(r, |r| Ok(Port::try_from(r.read_u8()?).ok()))?,
	})
}

fn frame_start(r: &mut &[u8]) -> Result<FrameEvent<FrameId, frame::Start>> {
	let id = FrameId::new(r.read_i32::<BE>()?);
	trace!("Frame Start: {:?}", id);
	Ok(FrameEvent {
		id: id,
		event: frame::Start {
			random_seed: r.read_u32::<BE>()?,
			scene_frame_counter: if_more(r, |r| r.read_u32::<BE>())?,
		},
	})
}

fn frame_end(r: &mut &[u8]) -> Result<FrameEvent<FrameId, frame::End>> {
	let id = FrameId::new(r.read_i32::<BE>()?);
	trace!("Frame End: {:?}", id);
	Ok(FrameEvent {
		id: id,
		event: frame::End {
			latest_finalized_frame: if_more(r, |r| r.read_i32::<BE>())?,
		},
	})
}

fn item(r: &mut &[u8]) -> Result<FrameEvent<FrameId, Item>> {
	let id = FrameId::new(r.read_i32::<BE>()?);
	trace!("Item Update: {:?}", id);
	let r#type = item::Type(r.read_u16::<BE>()?);
	Ok(FrameEvent {
		id: id,
		event: Item {
			r#type: r#type,
			state: item::State(r.read_u8()?),
			direction: {
				let x = r.read_f32::<BE>()?;
				if x == 0.0 {
					None
				} else {
					Some(x.try_into()?)
				}
			},
			velocity: Velocity {
				x: r.read_f32::<BE>()?,
				y: r.read_f32::<BE>()?,
			},
			position: Position {
				x: r.read_f32::<BE>()?,
				y: r.read_f32::<BE>()?,
			},
			damage: r.read_u16::<BE>()?,
			timer: r.read_f32::<BE>()?,
			id: r.read_u32::<BE>()?,
			// v3.2
			misc: if_more(r, |r| {
				Ok([r.read_u8()?, r.read_u8()?, r.read_u8()?, r.read_u8()?])
			})?,
			// v3.6
			owner: if_more(r, |r| Ok(Port::try_from(r.read_u8()?).ok()))?,
		},
	})
}

/// We need to know the character to interpret the action state properly,
/// but for Sheik/Zelda we don't know whether they transformed this frame
/// until we get the corresponding `frame::Post` event. So we predict based
/// on whether we were on the last frame of `TRANSFORM_AIR` or
/// `TRANSFORM_GROUND` during the *previous* frame.
fn predict_character(id: PortId, last_char_states: &[CharState; NUM_PORTS]) -> Internal {
	let prev = last_char_states[id.port as usize];
	match prev.state {
		State::Zelda(action_state::Zelda::TRANSFORM_GROUND)
		| State::Zelda(action_state::Zelda::TRANSFORM_AIR)
			if prev.age >= ZELDA_TRANSFORM_FRAME =>
		{
			Internal::SHEIK
		}
		State::Sheik(action_state::Sheik::TRANSFORM_GROUND)
		| State::Sheik(action_state::Sheik::TRANSFORM_AIR)
			if prev.age >= SHEIK_TRANSFORM_FRAME =>
		{
			Internal::ZELDA
		}
		_ => prev.character,
	}
}

fn frame_pre(
	r: &mut &[u8],
	last_char_states: &[CharState; NUM_PORTS],
) -> Result<FrameEvent<PortId, Pre>> {
	let id = PortId::new(r.read_i32::<BE>()?, r.read_u8()?, r.read_u8()? != 0)?;
	trace!("Pre-Frame Update: {:?}", id);

	let character = predict_character(id, last_char_states);

	let random_seed = r.read_u32::<BE>()?;
	let state = State::from(r.read_u16::<BE>()?, character);

	let position = Position {
		x: r.read_f32::<BE>()?,
		y: r.read_f32::<BE>()?,
	};
	let direction = r.read_f32::<BE>()?.try_into()?;
	let joystick = Position {
		x: r.read_f32::<BE>()?,
		y: r.read_f32::<BE>()?,
	};
	let cstick = Position {
		x: r.read_f32::<BE>()?,
		y: r.read_f32::<BE>()?,
	};
	let trigger_logical = r.read_f32::<BE>()?;
	let buttons = frame::Buttons {
		logical: buttons::Logical(r.read_u32::<BE>()?),
		physical: buttons::Physical(r.read_u16::<BE>()?),
	};
	let triggers = frame::Triggers {
		logical: trigger_logical,
		physical: triggers::Physical {
			l: r.read_f32::<BE>()?,
			r: r.read_f32::<BE>()?,
		},
	};

	Ok(FrameEvent {
		id: id,
		event: Pre {
			random_seed: random_seed,
			state: state,
			position: position,
			direction: direction,
			joystick: joystick,
			cstick: cstick,
			triggers: triggers,
			buttons: buttons,
			// v1.2
			raw_analog_x: if_more(r, |r| r.read_u8())?,
			// v1.4
			damage: if_more(r, |r| r.read_f32::<BE>())?,
		},
	})
}

fn flags(buf: &[u8; 5]) -> frame::StateFlags {
	frame::StateFlags(
		(buf[0] as u64)
			+ ((buf[1] as u64) << 08)
			+ ((buf[2] as u64) << 16)
			+ ((buf[3] as u64) << 24)
			+ ((buf[4] as u64) << 32),
	)
}

fn update_last_char_state(
	id: PortId,
	character: Internal,
	state: State,
	last_char_states: &mut [CharState; NUM_PORTS],
) {
	const Z_AIR: State = State::Zelda(action_state::Zelda::TRANSFORM_AIR);
	const Z_GROUND: State = State::Zelda(action_state::Zelda::TRANSFORM_GROUND);
	const S_AIR: State = State::Sheik(action_state::Sheik::TRANSFORM_AIR);
	const S_GROUND: State = State::Sheik(action_state::Sheik::TRANSFORM_GROUND);
	let prev = last_char_states[id.port as usize];
	last_char_states[id.port as usize] = CharState {
		character: character,
		state: state,
		age: match (prev.state, state) {
			(s0, s1) if s0 == s1 => prev.age + 1,
			// `TRANSFORM_AIR` can transition into `TRANSFORM_GROUND`
			// without interruption, so conflate them for age purposes.
			// Note: if you land on the frame where you would have transitioned from
			// `TRANSFORM_AIR` to `TRANSFORM_AIR_ENDING`, you instead transition to
			// `TRANSFORM_GROUND` for one frame before going to
			// `TRANSFORM_GROUND_ENDING` on the next frame. This delays the character
			// switch by one frame, so we cap `age` at its previous value so as not to
			// confuse `predict_character`.
			(Z_AIR, Z_GROUND) | (Z_GROUND, Z_AIR) => min(ZELDA_TRANSFORM_FRAME - 1, prev.age + 1),
			(S_AIR, S_GROUND) | (S_GROUND, S_AIR) => min(SHEIK_TRANSFORM_FRAME - 1, prev.age + 1),
			_ => 0,
		},
	};
}

fn frame_post(
	r: &mut &[u8],
	last_char_states: &mut [CharState; NUM_PORTS],
) -> Result<FrameEvent<PortId, Post>> {
	let id = PortId::new(r.read_i32::<BE>()?, r.read_u8()?, r.read_u8()? != 0)?;
	trace!("Post-Frame Update: {:?}", id);

	let character = Internal(r.read_u8()?);
	let state = State::from(r.read_u16::<BE>()?, character);
	let position = Position {
		x: r.read_f32::<BE>()?,
		y: r.read_f32::<BE>()?,
	};
	let direction = r.read_f32::<BE>()?.try_into()?;
	let damage = r.read_f32::<BE>()?;
	let shield = r.read_f32::<BE>()?;
	let last_attack_landed = {
		let attack = r.read_u8()?;
		match attack {
			0 => None,
			attack => Some(Attack(attack)),
		}
	};
	let combo_count = r.read_u8()?;
	let last_hit_by = Port::try_from(r.read_u8()?).ok();
	let stocks = r.read_u8()?;

	// v0.2
	let state_age = if_more(r, |r| r.read_f32::<BE>())?;

	// v2.0
	let flags = if_more(r, |r| {
		let mut buf = [0; 5];
		r.read_exact(&mut buf)?;
		Ok(flags(&buf))
	})?;
	let misc_as = if_more(r, |r| r.read_f32::<BE>())?;
	let airborne = if_more(r, |r| Ok(r.read_u8()? != 0))?;
	let ground = if_more(r, |r| Ok(ground::Ground(r.read_u16::<BE>()?)))?;
	let jumps = if_more(r, |r| r.read_u8())?;
	let l_cancel = if_more(r, |r| {
		Ok(match r.read_u8()? {
			0 => None,
			1 => Some(true),
			2 => Some(false),
			i => return Err(err!("invalid L-Cancel value: {}", i)),
		})
	})?;

	// v2.1
	let hurtbox_state = if_more(r, |r| Ok(frame::HurtboxState(r.read_u8()?)))?;

	// v3.5
	let velocities = if_more(r, |r| {
		Ok({
			let autogenous_x_air = r.read_f32::<BE>()?;
			let autogenous_y = r.read_f32::<BE>()?;
			let knockback_x = r.read_f32::<BE>()?;
			let knockback_y = r.read_f32::<BE>()?;
			let autogenous_x_ground = r.read_f32::<BE>()?;
			frame::Velocities {
				autogenous: Velocity {
					x: match airborne.unwrap() {
						true => autogenous_x_air,
						_ => autogenous_x_ground,
					},
					y: autogenous_y,
				},
				autogenous_x: frame::AutogenousXVelocity {
					air: autogenous_x_air,
					ground: autogenous_x_ground,
				},
				knockback: Velocity {
					x: knockback_x,
					y: knockback_y,
				},
			}
		})
	})?;

	// v3.8
	let hitlag = if_more(r, |r| r.read_f32::<BE>())?;

	// v3.11
	let animation_index = if_more(r, |r| r.read_u32::<BE>())?;

	update_last_char_state(id, character, state, last_char_states);

	Ok(FrameEvent {
		id: id,
		event: Post {
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
			// v0.2
			state_age: state_age,
			// v2.0
			flags: flags,
			misc_as: misc_as,
			airborne: airborne,
			ground: ground,
			jumps: jumps,
			l_cancel: l_cancel,
			// v2.1
			hurtbox_state: hurtbox_state,
			// v3.5
			velocities: velocities,
			// v3.8
			hitlag: hitlag,
			// v3.11
			animation_index: animation_index,
		},
	})
}

/// Callbacks for events encountered while parsing a replay.
///
/// For frame events, there will be one event per frame per character
/// (Ice Climbers are two characters).
pub trait Handlers {
	// Descriptions below partially copied from the Slippi spec:
	// https://github.com/project-slippi/slippi-wiki/blob/master/SPEC.md

	/// List of enabled Gecko codes. Currently unparsed.
	fn gecko_codes(&mut self, _codes: &[u8], _actual_size: u16) -> Result<()> {
		Ok(())
	}

	/// How the game is set up; also includes the version of the extraction code.
	fn game_start(&mut self, _: game::Start) -> Result<()> {
		Ok(())
	}
	/// The end of the game.
	fn game_end(&mut self, _: game::End) -> Result<()> {
		Ok(())
	}
	/// Miscellaneous data not directly provided by Melee.
	fn metadata(&mut self, _: serde_json::Map<String, serde_json::Value>) -> Result<()> {
		Ok(())
	}

	/// RNG seed and frame number at the start of a frame's processing.
	fn frame_start(&mut self, _: FrameEvent<FrameId, frame::Start>) -> Result<()> {
		Ok(())
	}
	/// Pre-frame update, collected right before controller inputs are used to figure out the character's next action. Used to reconstruct a replay.
	fn frame_pre(&mut self, _: FrameEvent<PortId, Pre>) -> Result<()> {
		Ok(())
	}
	/// Post-frame update, collected at the end of the Collision detection which is the last consideration of the game engine. Useful for making decisions about game states, such as computing stats.
	fn frame_post(&mut self, _: FrameEvent<PortId, Post>) -> Result<()> {
		Ok(())
	}
	/// Indicates an entire frame's worth of data has been transferred/processed.
	fn frame_end(&mut self, _: FrameEvent<FrameId, frame::End>) -> Result<()> {
		Ok(())
	}

	/// One event per frame per item, with a maximum of 15 updates per frame. Can be used for stats, training AIs, or visualization engines to handle items. Items include projectiles like lasers or needles.
	fn item(&mut self, _: FrameEvent<FrameId, Item>) -> Result<()> {
		Ok(())
	}

	/// Called after all parse events have been handled.
	fn finalize(&mut self) -> Result<()> {
		Ok(())
	}
}

fn expect_bytes<R: Read>(r: &mut R, expected: &[u8]) -> Result<()> {
	let mut actual = vec![0; expected.len()];
	r.read_exact(&mut actual)?;
	if expected == actual.as_slice() {
		Ok(())
	} else {
		Err(err!("expected: {:?}, got: {:?}", expected, actual))
	}
}

fn handle_splitter_event(buf: &[u8], accumulator: &mut Option<Vec<u8>>) -> Result<Option<u8>> {
	assert_eq!(buf.len(), 516);
	let actual_size = (&buf[512..514]).read_u16::<BE>()?;
	assert!(actual_size <= 512);
	let wrapped_event = buf[514];
	let is_final = buf[515] != 0;

	if accumulator.is_none() {
		*accumulator = Some(Vec::new());
	}
	let accumulator = accumulator.as_mut().unwrap();

	// bytes beyond `actual_size` are meaningless,
	// but save them anyway for lossless round-tripping
	accumulator.extend_from_slice(&buf[0..512]);

	Ok(match is_final {
		true => Some(wrapped_event),
		_ => None,
	})
}

/// Parses a single event from the raw stream. If the event is one of the
/// supported `Event` types, calls the corresponding `Handler` callback with
/// the parsed event.
///
/// Returns the number of bytes read by this function.
fn event<R: Read, H: Handlers>(
	mut r: R,
	payload_sizes: &HashMap<u8, u16>,
	last_char_states: &mut [CharState; NUM_PORTS],
	handlers: &mut H,
	splitter_accumulator: &mut Option<Vec<u8>>,
) -> Result<(usize, Option<Event>)> {
	let mut code = r.read_u8()?;
	debug!("Event: {:#x}", code);

	let size = *payload_sizes
		.get(&code)
		.ok_or_else(|| err!("unknown event: {}", code))? as usize;
	let mut buf = vec![0; size];
	r.read_exact(&mut *buf)?;

	if code == 0x10 {
		// message splitter
		if let Some(wrapped_event) = handle_splitter_event(&buf, splitter_accumulator)? {
			code = wrapped_event;
			buf.clear();
			buf.append(splitter_accumulator.as_mut().unwrap());
		}
	};

	let event = Event::try_from(code).ok();
	if let Some(event) = event {
		use Event::*;
		match event {
			GameStart => handlers.game_start(game_start(&mut &*buf)?)?,
			GameEnd => handlers.game_end(game_end(&mut &*buf)?)?,
			FrameStart => handlers.frame_start(frame_start(&mut &*buf)?)?,
			FramePre => handlers.frame_pre(frame_pre(&mut &*buf, last_char_states)?)?,
			FramePost => handlers.frame_post(frame_post(&mut &*buf, last_char_states)?)?,
			FrameEnd => handlers.frame_end(frame_end(&mut &*buf)?)?,
			Item => handlers.item(item(&mut &*buf)?)?,
			GeckoCodes => handlers.gecko_codes(&buf, payload_sizes[&(GeckoCodes as u8)])?,
		};
	}

	Ok((1 + size as usize, event)) // +1 byte for the event code
}

/// Options for parsing replays.
#[derive(Clone, Copy, Debug)]
pub struct Opts {
	/// Skip all frame data when parsing a replay for speed
	/// (when you only need start/end/metadata).
	pub skip_frames: bool,
}

/// Parses a Slippi replay from `r`, passing events to the callbacks in `handlers` as they occur.
pub fn deserialize<R: Read, H: Handlers>(
	mut r: &mut R,
	handlers: &mut H,
	opts: Option<Opts>,
) -> Result<()> {
	// For speed, assume the `raw` element comes first and handle it manually.
	// The official JS parser does this too, so it should be reliable.
	expect_bytes(
		&mut r,
		// top-level opening brace, `raw` key & type ("{U\x03raw[$U#l")
		&[
			0x7b, 0x55, 0x03, 0x72, 0x61, 0x77, 0x5b, 0x24, 0x55, 0x23, 0x6c,
		],
	)?;

	let raw_len = r.read_u32::<BE>()? as usize;
	let (mut bytes_read, payload_sizes) = payload_sizes(&mut r)?;
	let mut last_char_states = [DEFAULT_CHAR_STATE; NUM_PORTS];
	let mut last_event: Option<Event> = None;
	let skip_frames = opts.map(|o| o.skip_frames).unwrap_or(false);

	let mut splitter_accumulator = None;

	// `raw_len` will be 0 for an in-progress replay
	while (raw_len == 0 || bytes_read < raw_len) && last_event != Some(Event::GameEnd) {
		if skip_frames && last_event == Some(Event::GameStart) {
			// Skip to GameEnd, which we assume is the last event in the stream!
			let skip = raw_len - bytes_read - payload_sizes[&(Event::GameEnd as u8)] as usize - 1;
			// In theory we should seek() if `r` is Seekable, but it's not much
			// faster and is very awkward to implement without specialization.
			io::copy(&mut r.by_ref().take(skip as u64), &mut io::sink())?;
			bytes_read += skip;
		}
		let (bytes, event) = event(
			r.by_ref(),
			&payload_sizes,
			&mut last_char_states,
			handlers,
			&mut splitter_accumulator,
		)?;
		bytes_read += bytes;
		last_event = event;
	}

	if raw_len != 0 && bytes_read != raw_len {
		return Err(err!(
			"failed to consume expected number of bytes: {}, {}",
			raw_len,
			bytes_read
		));
	}

	expect_bytes(
		&mut r,
		// `metadata` key & type ("U\x08metadata{")
		&[
			0x55, 0x08, 0x6d, 0x65, 0x74, 0x61, 0x64, 0x61, 0x74, 0x61, 0x7b,
		],
	)?;

	// Since we already read the opening "{" from the `metadata` value,
	// we know it's a map. `parse_map` will consume the corresponding "}".
	let metadata = ubjson::de::to_map(&mut r)?;
	info!("Raw metadata: {}", serde_json::to_string(&metadata)?);
	handlers.metadata(metadata)?;

	expect_bytes(&mut r, &[0x7d])?; // top-level closing brace ("}")

	handlers.finalize()?;
	Ok(())
}
