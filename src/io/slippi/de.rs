use std::{
	collections::HashMap,
	fs::{self, File},
	io::{self, Read, Seek, SeekFrom, Write},
	num::NonZeroU16,
	path::PathBuf,
};

use arrow2::{array::MutableArray, offset::Offsets};
use byteorder::ReadBytesExt;
use log::{debug, info, trace, warn};

type BE = byteorder::BigEndian;

use crate::{
	frame::{self, mutable::Frame as MutableFrame, transpose},
	game::{
		self, MAX_PLAYERS, Match, NUM_PORTS, Netplay, Player, PlayerType, Port, Quirks,
		immutable::Game, port_occupancy, shift_jis::MeleeString,
	},
	io::{HashingReader, Result, expect_bytes, slippi, ubjson},
};

type PayloadSizes = [Option<NonZeroU16>; 256];

#[derive(Clone, Debug)]
pub struct Debug {
	/// Output the each event's payload to `{dir}/{event_code}/{event_num}`.
	pub dir: PathBuf,
}

/// Options for parsing replays.
#[derive(Clone, Debug, Default)]
pub struct Opts {
	/// Skip all frame data (faster when you only need start/end/metadata).
	pub skip_frames: bool,
	/// Compute a hash of the replay's contents.
	pub compute_hash: bool,
	/// Debug options.
	pub debug: Option<Debug>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum Event {
	MessageSplitter = 0x10,
	Payloads = 0x35,
	GameStart = 0x36,
	FramePre = 0x37,
	FramePost = 0x38,
	GameEnd = 0x39,
	FrameStart = 0x3A,
	Item = 0x3B,
	FrameEnd = 0x3C,
	GeckoCodes = 0x3D,
	FodPlatform = 0x3F,
	DreamlandWhispy = 0x40,
	StadiumTransformation = 0x41,
}

#[derive(Debug, Default)]
struct SplitAccumulator {
	raw: Vec<u8>,
	actual_size: u32,
}

pub struct PartialGame {
	pub start: game::Start,
	pub end: Option<game::End>,
	pub frames: MutableFrame,
	pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
	pub gecko_codes: Option<game::GeckoCodes>,
	pub hash: Option<String>,
	pub quirks: Option<Quirks>,
}

impl From<PartialGame> for Game {
	fn from(game: PartialGame) -> Game {
		Game {
			start: game.start,
			end: game.end,
			frames: game.frames.into(),
			metadata: game.metadata,
			gecko_codes: game.gecko_codes,
			hash: game.hash,
			quirks: game.quirks,
		}
	}
}

pub struct ParseState {
	payload_sizes: PayloadSizes,
	bytes_read: usize,
	event_counts: HashMap<u8, usize>,
	split_accumulator: SplitAccumulator,
	port_indexes: [usize; 4],
	skipping_frames: bool,
	game: PartialGame,
}

impl game::Game for ParseState {
	fn start(&self) -> &game::Start {
		&self.game.start
	}

	fn end(&self) -> &Option<game::End> {
		&self.game.end
	}

	fn metadata(&self) -> &Option<serde_json::Map<String, serde_json::Value>> {
		&self.game.metadata
	}

	fn gecko_codes(&self) -> &Option<game::GeckoCodes> {
		&self.game.gecko_codes
	}

	fn len(&self) -> usize {
		self.game.frames.len()
	}

	fn frame(&self, idx: usize) -> transpose::Frame {
		self.game
			.frames
			.transpose_one(idx, self.game.start.slippi.version)
	}
}

impl ParseState {
	pub fn frames(&self) -> &MutableFrame {
		&self.game.frames
	}

	pub fn bytes_read(&self) -> usize {
		self.bytes_read
	}

	fn last_id(&self) -> Option<i32> {
		self.game.frames.id.values().last().map(|id| *id)
	}

	fn frame_open(&mut self, id: i32) {
		self.game.frames.id.push(Some(id));
	}

	fn frame_close(&mut self) {
		let len = self.game.frames.len();
		for p in &mut self.game.frames.ports {
			while p.leader.len() < len {
				p.leader.push_null(self.game.start.slippi.version);
			}
			if let Some(f) = &mut p.follower {
				while f.len() < len {
					f.push_null(self.game.start.slippi.version);
				}
			}
		}
	}
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

#[allow(clippy::too_many_arguments)]
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
	let mut unmapped = [0; 11];

	let character = r.read_u8()?;
	let r#type = PlayerType::try_from(r.read_u8()?).ok();
	let stocks = r.read_u8()?;
	let costume = r.read_u8()?;
	r.read_exact(&mut unmapped[0..3])?;
	let team_shade = r.read_u8()?;
	let handicap = r.read_u8()?;
	let team_color = r.read_u8()?;
	let team = {
		match is_teams {
			true => Some(game::Team {
				color: team_color,
				shade: team_shade,
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
			Some(PlayerType::Cpu) => Some(cpu_level),
			_ => None,
		}
	};
	let damage_start = r.read_u16::<BE>()?;
	let damage_spawn = r.read_u16::<BE>()?;
	r.read_exact(&mut unmapped[7..11])?;
	let offense_ratio = r.read_f32::<BE>()?;
	let defense_ratio = r.read_f32::<BE>()?;
	let model_scale = r.read_f32::<BE>()?;
	// total bytes: 0x24

	// v1.0
	let ucf = match v1_0 {
		Some(v1_0) => {
			let mut r = &v1_0[..];
			Some(game::Ucf {
				dash_back: match r.read_u32::<BE>()? {
					0 => None,
					x => Some(
						game::DashBack::try_from(x)
							.map_err(|_| err!("invalid UCF dashback: {}", x))?,
					),
				},
				shield_drop: match r.read_u32::<BE>()? {
					0 => None,
					x => Some(
						game::ShieldDrop::try_from(x)
							.map_err(|_| err!("invalid UCF shield drop: {}", x))?,
					),
				},
			})
		}
		_ => None,
	};

	// v1_3
	let name_tag = v1_3
		.map(|v1_3| MeleeString::try_from(v1_3.as_slice()))
		.transpose()?;

	// v3.9
	let netplay = v3_9_name
		.zip(v3_9_code)
		.map(|(name, code)| {
			let suid = v3_11
				.map(|v3_11| {
					let first_null = v3_11.iter().position(|&x| x == 0).unwrap_or(28);
					let result = std::str::from_utf8(&v3_11[0..first_null]);
					result
						.map(String::from)
						.map_err(|_| err!("invalid netplay SUID: {:?}", v3_11))
				})
				.transpose()?;
			Result::Ok(Netplay {
				name: MeleeString::try_from(name.as_slice())?,
				code: MeleeString::try_from(code.as_slice())?,
				suid,
			})
		})
		.transpose()?;

	Ok(r#type.map(|r#type| Player {
		port,
		character,
		r#type,
		stocks,
		costume,
		team,
		handicap,
		bitfield,
		cpu_level,
		damage_start,
		damage_spawn,
		offense_ratio,
		defense_ratio,
		model_scale,
		// v1_0
		ucf,
		// v1_3
		name_tag,
		// v3.9
		netplay,
	}))
}

fn player_bytes<const N: usize, const M: usize>(r: &mut &[u8]) -> Result<[[u8; N]; M]> {
	let mut arrs: [[u8; N]; M] = [[0; N]; M];
	arrs.iter_mut().try_for_each(|buf| r.read_exact(buf))?;
	Ok(arrs)
}

pub(crate) fn game_start(r: &mut &[u8]) -> Result<game::Start> {
	let bytes = game::Bytes(r.to_vec());
	let ver = slippi::Version(r.read_u8()?, r.read_u8()?, r.read_u8()?);
	let slippi = slippi::Slippi { version: ver };
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
	let stage = r.read_u16::<BE>()?;
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
	let players_v0 = player_bytes::<36, MAX_PLAYERS>(r)?;
	// @0x13d
	let random_seed = r.read_u32::<BE>()?;

	// note the shift from `MAX_PLAYERS` to `NUM_PORTS`: Slippi only supports 4 players!
	let players_v1_0 = match ver.gte(1, 0) {
		true => Some(player_bytes::<8, NUM_PORTS>(r)?),
		_ => None,
	};

	let players_v1_3 = match ver.gte(1, 3) {
		true => Some(player_bytes::<16, NUM_PORTS>(r)?),
		_ => None,
	};

	let is_pal = match ver.gte(1, 5) {
		true => Some(r.read_u8()? != 0),
		_ => None,
	};

	let is_frozen_ps = match ver.gte(2, 0) {
		true => Some(r.read_u8()? != 0),
		_ => None,
	};

	let scene = match ver.gte(3, 7) {
		true => Some(game::Scene {
			minor: r.read_u8()?,
			major: r.read_u8()?,
		}),
		_ => None,
	};

	let players_v3_9 = match ver.gte(3, 9) {
		true => Some((
			player_bytes::<31, NUM_PORTS>(r)?,
			player_bytes::<10, NUM_PORTS>(r)?,
		)),
		_ => None,
	};

	let players_v3_11 = match ver.gte(3, 11) {
		true => Some(player_bytes::<29, NUM_PORTS>(r)?),
		_ => None,
	};

	let language = if ver.gte(3, 12) {
		let b = r.read_u8()?;
		Some(game::Language::try_from(b).map_err(|_| err!("invalid language: {}", b))?)
	} else {
		None
	};

	let r#match = if ver.gte(3, 14) {
		let id = {
			let mut buf = [0u8; 51];
			r.read_exact(&mut buf)?;
			let first_null = buf.iter().position(|&x| x == 0).unwrap_or(50);
			let result = std::str::from_utf8(&buf[0..first_null]);
			result
				.map(String::from)
				.map_err(|_| err!("invalid match ID: {:?}", buf))
		}?;
		let game = r.read_u32::<BE>()?;
		let tiebreaker = r.read_u32::<BE>()?;
		Some(Match {
			id,
			game,
			tiebreaker,
		})
	} else {
		None
	};

	let players = (0..NUM_PORTS)
		.filter_map(|n| {
			player(
				Port::try_from(n as u8).unwrap(),
				&players_v0[n],
				is_teams,
				players_v1_0.map(|p| p[n]),
				players_v1_3.map(|p| p[n]),
				players_v3_9.map(|p| p.0[n]),
				players_v3_9.map(|p| p.1[n]),
				players_v3_11.map(|p| p[n]),
			)
			.transpose()
		})
		.collect::<Result<Vec<_>>>()?;

	Ok(game::Start {
		slippi,
		bitfield,
		is_raining_bombs,
		is_teams,
		item_spawn_frequency,
		self_destruct_score,
		stage,
		timer,
		item_spawn_bitfield,
		damage_ratio,
		players,
		random_seed,
		bytes,
		// v1.5
		is_pal,
		// v2.0
		is_frozen_ps,
		// v3.7
		scene,
		// v3.12
		language,
		r#match,
	})
}

pub fn player_end(port: Port, placement: i8) -> Result<Option<game::PlayerEnd>> {
	match placement {
		-1 => Ok(None),
		0..=3 => Ok(Some(game::PlayerEnd {
			port,
			placement: placement as u8,
		})),
		p => Err(err!("Invalid player placement {}", p)),
	}
}

pub(crate) fn game_end(r: &mut &[u8]) -> Result<game::End> {
	let bytes = game::Bytes(r.to_vec());
	let method = {
		let b = r.read_u8()?;
		game::EndMethod::try_from(b).map_err(|_| err!("invalid game end method: {}", b))?
	};

	// v2.0
	let lras_initiator = if_more(r, |r| {
		Ok(match r.read_u8()? {
			255 => None,
			x => Some(Port::try_from(x).map_err(|_| err!("invalid LRAS initiator: {}", x))?),
		})
	})?;

	// v3.13
	let players = if_more(r, |r| {
		let placements = [r.read_i8()?, r.read_i8()?, r.read_i8()?, r.read_i8()?];
		(0..NUM_PORTS)
			.filter_map(|n| {
				player_end(Port::try_from(n as u8).unwrap(), placements[n as usize]).transpose()
			})
			.collect()
	})?;

	Ok(game::End {
		method,
		bytes,
		lras_initiator,
		players,
	})
}

fn handle_splitter_event(buf: &[u8], accumulator: &mut SplitAccumulator) -> Result<Option<u8>> {
	assert_eq!(buf.len(), 516);
	let actual_size = (&buf[512..514]).read_u16::<BE>()?;
	assert!(actual_size <= 512);
	let wrapped_event = buf[514];
	let is_final = buf[515] != 0;

	// bytes beyond `actual_size` are meaningless,
	// but save them anyway for lossless round-tripping
	accumulator.raw.extend_from_slice(&buf[0..512]);
	accumulator.actual_size += actual_size as u32;

	Ok(match is_final {
		true => Some(wrapped_event),
		_ => None,
	})
}

fn debug_write_event(
	buf: &[u8],
	code: u8,
	state: Option<&ParseState>,
	debug: &Debug,
) -> Result<()> {
	// write the event's raw data to "{debug.dir}/{code}/{count}",
	// where `count` is how many of that event we've seen already
	let code_dir = debug.dir.join(format!("{}", code));
	fs::create_dir_all(&code_dir)?;
	let count = state.map_or(0, |s| *s.event_counts.get(&code).unwrap_or(&0));
	let mut f = File::create(code_dir.join(format!("{}", count)))?;
	f.write_all(buf)?;
	Ok(())
}

/// Parses an Event Payloads event from `r`, which must come first in the raw
/// stream and tells us the sizes for all other events to follow.
///
/// Returns the number of bytes read, and a map of event codes to payload sizes.
/// This map uses raw event codes as keys (as opposed to `Event` enum values)
/// for forwards compatibility, to allow skipping unknown events.
fn parse_payloads<R: Read>(mut r: R, opts: Option<&Opts>) -> Result<(usize, PayloadSizes)> {
	let code = r.read_u8()?;
	if code != Event::Payloads as u8 {
		return Err(err!("expected event payloads, but got: {:#02x}", code));
	}

	// Size in bytes of the subsequent list of payload-size kv pairs.
	// Each pair is 3 bytes, so this size should be divisible by 3.
	// However the value includes this size byte itself, so it's off-by-one.
	let size = r.read_u8()?;
	if size % 3 != 1 {
		return Err(err!("invalid payload size: {}", size));
	}

	let mut buf = vec![0; (size - 1) as usize];
	r.read_exact(&mut buf)?;
	let buf = &mut &buf[..];

	if let Some(ref d) = opts.as_ref().and_then(|o| o.debug.as_ref()) {
		debug_write_event(&buf, code, None, d)?;
	}

	let mut sizes: PayloadSizes = [None; 256];
	for _ in (0..size - 1).step_by(3) {
		let code = buf.read_u8()?;
		let size = buf.read_u16::<BE>()?;
		sizes[code as usize] =
			Some(NonZeroU16::new(size).ok_or_else(|| err!("zero-size event payload"))?);
	}

	sizes[Event::GameStart as usize].ok_or_else(|| err!("missing Game Start in payload sizes"))?;

	sizes[Event::GameEnd as usize].ok_or_else(|| err!("missing Game End in payload sizes"))?;

	debug!(
		"Event payload sizes: {{{}}}",
		sizes
			.iter()
			.enumerate()
			.filter_map(|(c, s)| s.map(|s| format!("0x{:x}: {}", c, s)))
			.collect::<Vec<_>>()
			.join(", ")
	);

	Ok((1 + size as usize, sizes)) // +1 byte for the event code
}

/// Parses a Game Start event from `r`, which must come immediately after the
/// Event Payloads.
///
/// Returns the number of bytes read, and a parsed `game::Start` event
/// (or Err if the event wasn't a Game Start).
fn parse_game_start<R: Read>(
	mut r: R,
	payload_sizes: &PayloadSizes,
	bytes_read: usize,
	opts: Option<&Opts>,
) -> Result<(usize, game::Start)> {
	let code = r.read_u8()?;
	debug!("Event {:#02x} @{:#x}", code, bytes_read);

	let size = payload_sizes[code as usize]
		.ok_or_else(|| err!("unknown event: {:#02x}", code))?
		.get() as usize;
	let mut buf = vec![0; size];
	r.read_exact(&mut buf)?;

	if let Some(ref d) = opts.as_ref().and_then(|o| o.debug.as_ref()) {
		debug_write_event(&buf, code, None, d)?;
	}

	match Event::try_from(code) {
		// +1 byte for the event code
		Ok(Event::GameStart) => Ok((bytes_read + size + 1, game_start(&mut &*buf)?)),
		_ => Err(err!("Invalid event before start: {:#02x}", code)),
	}
}

pub fn parse_header<R: Read>(mut r: R, _opts: Option<&Opts>) -> Result<u32> {
	// For speed, assume the `raw` element comes first and handle it manually.
	// The official JS parser does this too, so it should be reliable.
	expect_bytes(&mut r, &super::FILE_SIGNATURE)?;
	// `raw` content size in bytes
	Ok(r.read_u32::<BE>()?)
}

pub fn parse_start<R: Read>(mut r: R, opts: Option<&Opts>) -> Result<ParseState> {
	let (bytes_read, payload_sizes) = parse_payloads(&mut r, opts)?;
	let (bytes_read, start) = parse_game_start(&mut r, &payload_sizes, bytes_read, opts)?;

	let ports = port_occupancy(&start);
	let version = start.slippi.version;
	let capacity = match opts.map_or(false, |o| o.skip_frames) {
		true => 0,
		false => 1024,
	};
	let game = PartialGame {
		start: start.clone(),
		end: None,
		frames: MutableFrame::with_capacity(capacity, version, &ports),
		metadata: None,
		gecko_codes: None,
		hash: None,
		quirks: None,
	};

	let port_indexes = {
		let mut result = [0, 0, 0, 0];
		for (i, p) in ports.into_iter().enumerate() {
			result[p.port as usize] = i;
		}
		result
	};

	let event_counts = HashMap::from([(Event::Payloads as u8, 1), (Event::GameStart as u8, 1)]);

	Ok(ParseState {
		payload_sizes,
		bytes_read,
		event_counts,
		game,
		port_indexes,
		split_accumulator: Default::default(),
		skipping_frames: false,
	})
}

fn push_offset<T>(offsets: &mut Option<Offsets<i32>>, new_len: i32) {
	let old_len = *offsets.as_ref().unwrap().last();
	offsets
		.as_mut()
		.unwrap()
		.try_push(new_len.checked_sub(old_len).unwrap())
		.unwrap();
}

/// Parses a single event from `r`.
///
/// Returns the event code that was parsed.
pub fn parse_event<R: Read>(mut r: R, state: &mut ParseState, opts: Option<&Opts>) -> Result<u8> {
	let mut code = r.read_u8()?;

	if state.skipping_frames && code != Event::GameEnd as u8 {
		warn!("Missing end event");
		let size = state.payload_sizes[Event::GameEnd as usize]
			.expect("Missing GameEnd in playload sizes")
			.get() as usize;
		let mut buf = vec![0; size];
		r.read_exact(&mut buf)?;
		state.bytes_read += size + 1; // +1 byte for the event code
		return Ok(0);
	}

	debug!("Event {:#02x} @{:#x}", code, state.bytes_read);

	let size = state.payload_sizes[code as usize]
		.ok_or_else(|| err!("unknown event: {:#02x}", code))?
		.get() as usize;
	let mut buf = vec![0; size];
	r.read_exact(&mut buf)?;

	if code == Event::MessageSplitter as u8 {
		if let Some(wrapped_event) = handle_splitter_event(&buf, &mut state.split_accumulator)? {
			code = wrapped_event;
			buf.clear();
			buf.append(&mut state.split_accumulator.raw);
		}
	};

	if let Some(ref d) = opts.as_ref().and_then(|o| o.debug.as_ref()) {
		debug_write_event(&buf, code, Some(state), d)?;
	}

	*state.event_counts.entry(code).or_default() += 1;

	let event = Event::try_from(code).ok();
	if let Some(event) = event {
		use Event::*;
		match event {
			Payloads => return Err(err!("Duplicate payloads event")),
			MessageSplitter => {}
			GeckoCodes => {
				state.game.gecko_codes = Some(game::GeckoCodes {
					bytes: buf.to_vec(),
					actual_size: state.split_accumulator.actual_size,
				})
			}
			GameStart => return Err(err!("Duplicate start event")),
			GameEnd => state.game.end = Some(game_end(&mut &*buf)?),
			FrameStart => {
				// no FrameEnd events before v3.0, so simulate it
				if state.game.start.slippi.version.lt(3, 0) {
					state.frame_close();
				}
				let r = &mut &*buf;
				let id = r.read_i32::<BE>()?;
				trace!("Frame start: {}", id);
				state.frame_open(id);
				state
					.game
					.frames
					.start
					.as_mut()
					.unwrap()
					.read_push(r, state.game.start.slippi.version)?;
			}
			FramePre => {
				let r = &mut &*buf;
				let id = r.read_i32::<BE>()?;
				let port = r.read_u8()?;
				let is_follower = r.read_u8()? != 0;
				trace!("Frame pre: {}:{}", id, port);
				if state.game.start.slippi.version.gte(2, 2) {
					assert_eq!(id, state.last_id().unwrap());
				} else {
					// no Frame Start events before v2.2, but also no rollbacks
					let last_id = state.last_id().unwrap_or(frame::FIRST_INDEX - 1);
					if last_id + 1 == id {
						state.frame_open(id);
					} else {
						assert_eq!(id, last_id);
					}
				}
				let port_index = state.port_indexes[port as usize];
				if is_follower {
					state.game.frames.ports[port_index]
						.follower
						.as_mut()
						.unwrap()
						.validity
						.as_mut()
						.map(|v| v.push(true));
					state.game.frames.ports[port_index]
						.follower
						.as_mut()
						.unwrap()
						.pre
						.read_push(r, state.game.start.slippi.version)?;
				} else {
					state.game.frames.ports[port_index]
						.leader
						.validity
						.as_mut()
						.map(|v| v.push(true));
					state.game.frames.ports[port_index]
						.leader
						.pre
						.read_push(r, state.game.start.slippi.version)?;
				}
			}
			FramePost => {
				let r = &mut &*buf;
				let id = r.read_i32::<BE>()?;
				let port = r.read_u8()?;
				let is_follower = r.read_u8()? != 0;
				trace!("Frame post: {}:{}", id, port);
				assert_eq!(id, state.last_id().unwrap());
				match is_follower {
					true => state.game.frames.ports[state.port_indexes[port as usize]]
						.follower
						.as_mut()
						.unwrap()
						.post
						.read_push(r, state.game.start.slippi.version)?,
					_ => state.game.frames.ports[state.port_indexes[port as usize]]
						.leader
						.post
						.read_push(r, state.game.start.slippi.version)?,
				};
			}
			FrameEnd => {
				let r = &mut &*buf;
				let id = r.read_i32::<BE>()?;
				trace!("Frame end: {}", id);
				assert_eq!(id, state.last_id().unwrap());
				push_offset::<u32>(
					&mut state.game.frames.item_offset,
					state
						.game
						.frames
						.item
						.as_ref()
						.unwrap()
						.id
						.len()
						.try_into()
						.unwrap(),
				);
				if state.game.start.slippi.version.gte(3, 18) {
					push_offset::<u8>(
						&mut state.game.frames.fod_platform_offset,
						state
							.game
							.frames
							.fod_platform
							.as_ref()
							.unwrap()
							.platform
							.len()
							.try_into()
							.unwrap(),
					);
					push_offset::<u8>(
						&mut state.game.frames.dreamland_whispy_offset,
						state
							.game
							.frames
							.dreamland_whispy
							.as_ref()
							.unwrap()
							.direction
							.len()
							.try_into()
							.unwrap(),
					);
					push_offset::<u16>(
						&mut state.game.frames.stadium_transformation_offset,
						state
							.game
							.frames
							.stadium_transformation
							.as_ref()
							.unwrap()
							.event
							.len()
							.try_into()
							.unwrap(),
					);
				}
				state
					.game
					.frames
					.end
					.as_mut()
					.unwrap()
					.read_push(r, state.game.start.slippi.version)?;
				state.frame_close();
			}
			Item => {
				let r = &mut &*buf;
				let id = r.read_i32::<BE>()?;
				trace!("Frame item: {}", id);
				assert_eq!(id, state.last_id().unwrap());
				state
					.game
					.frames
					.item
					.as_mut()
					.unwrap()
					.read_push(r, state.game.start.slippi.version)?;
			}
			FodPlatform => {
				let r = &mut &*buf;
				let id = r.read_i32::<BE>()?;
				trace!("FOD platform: {}", id);
				assert_eq!(id, state.last_id().unwrap());
				state
					.game
					.frames
					.fod_platform
					.as_mut()
					.unwrap()
					.read_push(r, state.game.start.slippi.version)?;
			}
			DreamlandWhispy => {
				let r = &mut &*buf;
				let id = r.read_i32::<BE>()?;
				trace!("Dreamland Whispy: {}", id);
				assert_eq!(id, state.last_id().unwrap());
				state
					.game
					.frames
					.dreamland_whispy
					.as_mut()
					.unwrap()
					.read_push(r, state.game.start.slippi.version)?;
			}
			StadiumTransformation => {
				let r = &mut &*buf;
				let id = r.read_i32::<BE>()?;
				trace!("Stadium transformation: {}", id);
				assert_eq!(id, state.last_id().unwrap());
				state
					.game
					.frames
					.stadium_transformation
					.as_mut()
					.unwrap()
					.read_push(r, state.game.start.slippi.version)?;
			}
		};
	}

	state.bytes_read += size + 1; // +1 byte for the event code
	Ok(code)
}

/// Assumes you already consumed the `U`, because that's how you know if there's metadata.
pub fn parse_metadata<R: Read>(
	mut r: R,
	state: &mut ParseState,
	_opts: Option<&Opts>,
) -> Result<()> {
	expect_bytes(
		&mut r,
		// `metadata` key & type ("U\x08metadata{", minus the `U`)
		&[0x08, 0x6d, 0x65, 0x74, 0x61, 0x64, 0x61, 0x74, 0x61, 0x7b],
	)?;

	// Since we already read the opening "{" from the `metadata` value,
	// we know it's a map. `parse_map` will consume the corresponding "}".
	let metadata = ubjson::read_map(&mut r)?;
	info!("Metadata: {}", serde_json::to_string(&metadata)?);
	state.game.metadata = Some(metadata);
	Ok(())
}

/// Reads a Slippi (`.slp`) replay from `r`.
pub fn read<R: Read + Seek>(r: R, opts: Option<&Opts>) -> Result<Game> {
	let hash = opts.map_or(false, |o| o.compute_hash);
	// Wrap so we can hash all the bytes we've read at the end.
	let mut r = HashingReader::new(r, hash);

	// Handle Event Payloads and Game Start
	let raw_len = parse_header(&mut r, opts)? as usize;
	info!("Raw length: {} bytes", raw_len);

	let mut state = parse_start(&mut r, opts)?;

	if opts.map_or(false, |o| o.skip_frames) {
		// Skip to GameEnd, which we assume is the last event in the stream!
		let end_offset = 1 + state.payload_sizes[Event::GameEnd as usize]
			.expect("Missing GameEnd in payload sizes")
			.get() as usize;
		if raw_len == 0 || raw_len - state.bytes_read < end_offset {
			return Err(err!(
				"Cannot skip to game end. Replay in-progress or corrupted."
			));
		}
		let skip = raw_len - state.bytes_read - end_offset;
		info!("Jumping to GameEnd (skipping {} bytes)", skip);
		if hash {
			io::copy(&mut r.by_ref().take(skip as u64), &mut io::sink())?;
		} else {
			r.seek(SeekFrom::Current(
				skip.try_into()
					.map_err(|_| err!("invalid skip value: {}", skip))?,
			))?;
		}
		state.bytes_read += skip;
		state.skipping_frames = true;
	}

	// Main event loop. `raw_len` will be 0 for an in-progress replay.
	while raw_len == 0 || state.bytes_read < raw_len {
		if parse_event(r.by_ref(), &mut state, opts)? == Event::GameEnd as u8 {
			break;
		}
	}

	// FrameEnd doesn't exist until v3.0, so we simulate it in FrameStart/FramePre.
	// But that means there can be a "dangling" frame that we need to close here.
	if state.game.start.slippi.version.lt(3, 0) {
		state.frame_close();
	}

	info!("Frames: {}", state.game.frames.len());

	// Some replays have duplicated Game End events, which are safe to ignore.
	if state.bytes_read < raw_len {
		let len = raw_len - state.bytes_read;
		let mut buf = vec![0; len];
		r.read_exact(&mut buf)?;
		if len == 1 + game::End::size(state.game.start.slippi.version)
			&& buf[0] == Event::GameEnd as u8
		{
			info!("Skipping duplicate Game End event");
			state
				.game
				.quirks
				.get_or_insert(Quirks::default())
				.double_game_end = true;
		} else {
			warn!("Extra content after Game End ({} bytes)", len);
		}
	} else if raw_len > 0 && state.bytes_read > raw_len {
		warn!(
			"Consumed more than expected ({} bytes)",
			state.bytes_read - raw_len
		);
	}

	// Some replays have no `metadata` (e.g. Fizzi's anonymized Ranked dataset),
	// in which case the next char should be the final UBSJON `}`.

	// Programs like slippi nintendont or rwing add custom fields after the metadata.
	// So sometimes we can't expect the closing '}' byte after the metadata.
	// As of 7.2.0, the official parser is robust to this and ignores bytes after metadata.
	// https://github.com/project-slippi/slippi-js/blob/v7.2.0/src/utils/slpReader.ts#L115
	// We can ignore them safely.
	match r.read_u8()? {
		0x55 => parse_metadata(r.by_ref(), &mut state, opts)?,
		0x7d => {} // top-level closing brace ("}")
		x => return Err(err!("expected: 0x55 or 0x7d, got: {:#02x}", x)),
	};

	state.game.hash = r.into_digest();
	Ok(Game::from(state.game))
}
