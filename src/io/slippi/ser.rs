use std::io::Write;

use byteorder::WriteBytesExt;

use crate::{
	frame::immutable::{
		DreamlandWhispy, End, FodPlatform, Frame, Item, Post, Pre, StadiumTransformation, Start,
	},
	game::{self, GeckoCodes, MAX_PLAYERS, NUM_PORTS, Player, PlayerType, Port, immutable::Game},
	io::{
		Result,
		slippi::{self, STAGE_EVENTS_VERSION, de::Event},
		ubjson,
	},
};

type BE = byteorder::BigEndian;

struct PayloadSizes {
	/// Order matters for round-tripping, hence Vec rather than HashMap.
	sizes: Vec<(u8, u16)>,
}

impl PayloadSizes {
	fn new() -> Self {
		Self { sizes: Vec::new() }
	}

	fn push(&mut self, event: Event, size: usize) {
		self.sizes.push((event as u8, size.try_into().unwrap()))
	}

	fn raw_size(&self, game: &Game) -> Result<u32> {
		use Event::*;

		let counts = frame_counts(&game.frames);
		let sizes: std::collections::HashMap<u8, u16> =
			self.sizes.iter().map(|(k, v)| (*k, *v)).collect();
		Ok(1 + 1 + (3 * self.sizes.len() as u32) // Payload sizes
			+ 1 + sizes[&(GameStart as u8)] as u32 // GameStart
			+ (1 + sizes[&(GameEnd as u8)] as u32) * game_end_count(game)? // GameEnd
			+ counts.frame_data * (1 + sizes[&(FramePre as u8)] as u32) // FramePre
			+ counts.frame_data * (1 + sizes[&(FramePost as u8)] as u32) // FramePost
			+ sizes.get(&(FrameStart as u8)).map_or(0, |s| counts.frame * (1 + *s as u32)) // FrameStart
			+ sizes.get(&(FrameEnd as u8)).map_or(0, |s| counts.frame * (1 + *s as u32)) // FrameEnd
			+ sizes.get(&(Item as u8)).map_or(0, |s| counts.item * (1 + *s as u32)) // Item
			+ sizes.get(&(FodPlatform as u8)).map_or(0, |s| counts.fod_platform * (1 + *s as u32)) // FodPlatform
			+ sizes.get(&(DreamlandWhispy as u8)).map_or(0, |s| counts.dreamland_whispy * (1 + *s as u32)) // DreamlandWhispy
			+ sizes.get(&(StadiumTransformation as u8)).map_or(0, |s| counts.stadium_transformation * (1 + *s as u32)) // StadiumTransformation
			+ game.gecko_codes.as_ref().map_or(0, gecko_codes_size))
	}
}

fn game_end_count(game: &Game) -> Result<u32> {
	match (
		game.end.is_some(),
		game.quirks.map_or(false, |q| q.double_game_end),
	) {
		(false, false) => Ok(0),
		(true, false) => Ok(1),
		(true, true) => Ok(2),
		_ => Err(err!("`double_game_end` quirk is on, but no GameEnd event")),
	}
}

fn payload_sizes(game: &Game) -> PayloadSizes {
	let mut sizes = PayloadSizes::new();
	let ver = game.start.slippi.version.clone();

	const FRAME_NUMBER: usize = std::mem::size_of::<i32>();
	const PORT: usize = 2 * std::mem::size_of::<u8>(); // port number + is_follower

	sizes.push(Event::GameStart, game.start.bytes.0.len());
	sizes.push(Event::FramePre, FRAME_NUMBER + PORT + Pre::size(ver));
	sizes.push(Event::FramePost, FRAME_NUMBER + PORT + Post::size(ver));
	sizes.push(
		Event::GameEnd,
		game.end
			.as_ref()
			.map_or(game::End::size(ver), |e| e.bytes.0.len()),
	);

	if ver.gte(2, 2) {
		sizes.push(Event::FrameStart, FRAME_NUMBER + Start::size(ver));
		if ver.gte(3, 0) {
			sizes.push(Event::Item, FRAME_NUMBER + Item::size(ver));
			if ver.gte(3, 0) {
				sizes.push(Event::FrameEnd, FRAME_NUMBER + End::size(ver));
				if ver.gte(3, 3) {
					if let Some(codes) = &game.gecko_codes {
						// discard higher-order bits of actual_size, matching Slippi's behavior
						sizes.push(Event::GeckoCodes, codes.actual_size as u16 as usize);
						sizes.push(Event::MessageSplitter, 516);
					}
					if ver >= STAGE_EVENTS_VERSION {
						sizes.push(Event::FodPlatform, FRAME_NUMBER + FodPlatform::size(ver));
						sizes.push(
							Event::DreamlandWhispy,
							FRAME_NUMBER + DreamlandWhispy::size(ver),
						);
						sizes.push(
							Event::StadiumTransformation,
							FRAME_NUMBER + StadiumTransformation::size(ver),
						);
					}
				}
			}
		}
	}

	sizes
}

fn gecko_codes<W: Write>(w: &mut W, codes: &GeckoCodes) -> Result<()> {
	let mut pos = 0;
	let actual_size = codes.actual_size as usize;
	while pos < actual_size {
		w.write_u8(Event::MessageSplitter as u8)?;
		w.write_all(&codes.bytes[pos..pos + 512])?;
		w.write_u16::<BE>(std::cmp::min(512, actual_size - pos) as u16)?;
		w.write_u8(Event::GeckoCodes as u8)?;
		pos += 512;
		w.write_u8(u8::from(pos >= actual_size))?;
	}
	Ok(())
}

fn bool(b: bool) -> u8 {
	if b { 1 } else { 0 }
}

fn player(start: &game::Start, port: usize) -> Option<&Player> {
	if let Some(port) = Port::try_from(port as u8).ok() {
		start.players.iter().find(|p| p.port == port)
	} else {
		None
	}
}

fn _game_start(s: &game::Start) -> Result<Vec<u8>> {
	let mut buf = s.bytes.0.clone();
	let mut b = &mut buf[..];

	let ver = s.slippi.version;
	b.write_u8(ver.0)?;
	b.write_u8(ver.1)?;
	b.write_u8(ver.2)?;
	b = &mut b[1..]; // unused (build number)

	// Game Info block

	b.write_all(&s.bitfield)?;
	b = &mut b[2..]; // 0x04..0x06 (unmapped)
	b.write_u8(bool(s.is_raining_bombs))?; // 0x06
	b = &mut b[1..]; // 0x07 (unmapped)
	b.write_u8(bool(s.is_teams))?; // 0x08
	b = &mut b[2..]; // 0x09..0x0B (unmapped)
	b.write_i8(s.item_spawn_frequency)?; // 0x0B
	b.write_i8(s.self_destruct_score)?; // 0x0C
	b = &mut b[1..]; // 0x0D (unmapped)
	b.write_u16::<BE>(s.stage)?; // 0x0E
	b.write_u32::<BE>(s.timer)?; // 0x10..0x14
	b = &mut b[15..]; // 0x14..0x23 (unmapped)
	b.write_all(&s.item_spawn_bitfield)?; // 0x23..0x28
	b = &mut b[8..]; // 0x28..0x30 (unmapped)
	b.write_f32::<BE>(s.damage_ratio)?; // 0x30..0x34
	b = &mut b[44..]; // 0x34..0x60 (unmapped)
	for n in 0..MAX_PLAYERS {
		if let Some(p) = player(s, n) {
			b.write_u8(p.character)?; // 0x24n + 0x60
			b.write_u8(p.r#type as u8)?; // 0x24n + 0x61
			b.write_u8(p.stocks)?; // 0x24n + 0x62
			b.write_u8(p.costume)?; // 0x24n + 0x63
			b = &mut b[3..]; // 0x24n + 0x64..0x67 (unmapped)
			match p.team {
				// 0x24n + 0x67
				Some(t) => b.write_u8(t.shade)?,
				_ => b = &mut b[1..],
			}
			b.write_u8(p.handicap)?; // 0x24n + 0x68
			match p.team {
				// 0x24n + 0x69
				Some(t) => b.write_u8(t.color)?,
				_ => b = &mut b[1..],
			}
			b = &mut b[2..]; // 0x24n + 0x6A..0x6C (unmapped)
			b.write_u8(p.bitfield)?; // 0x24n + 0x6C
			b = &mut b[2..]; // 0x24n + 0x6D..0x6F (unmapped)
			match p.r#type {
				// 0x24n + 0x6F
				PlayerType::Cpu => b.write_u8(p.cpu_level.unwrap())?,
				_ => b = &mut b[1..],
			}
			b.write_u16::<BE>(p.damage_start)?; // 0x24n + 0x70..0x72
			b.write_u16::<BE>(p.damage_spawn)?; // 0x24n + 0x72..0x74
			b = &mut b[4..]; // 0x24n + 0x74..0x78 (unmapped)
			b.write_f32::<BE>(p.offense_ratio)?; // 0x24n + 0x78..0x7C
			b.write_f32::<BE>(p.defense_ratio)?; // 0x24n + 0x7C..0x80
			b.write_f32::<BE>(p.model_scale)?; // 0x24n + 0x80..0x84
		} else {
			b = &mut b[36..];
		}
	}
	b.write_u32::<BE>(s.random_seed)?; // 0x13D

	if ver.gte(1, 0) {
		for n in 0..NUM_PORTS {
			if let Some(p) = player(s, n) {
				let ucf = p.ucf.unwrap();
				b.write_u32::<BE>(ucf.dash_back.map_or(0, |x| x as u32))?; // 0x08n + 0x141
				b.write_u32::<BE>(ucf.shield_drop.map_or(0, |x| x as u32))?; // 0x08n + 0x145
			} else {
				b = &mut b[8..];
			}
		}
	}

	if ver.gte(1, 3) {
		for n in 0..NUM_PORTS {
			if let Some(p) = player(s, n) {
				let name_tag = p.name_tag.as_ref().unwrap().bytes();
				b.write_all(&name_tag.to_owned())?; // 0x10n + 0x161
				b.write_all(&vec![0; 16 - name_tag.len()])?;
			} else {
				b = &mut b[16..];
			}
		}
	}

	if ver.gte(1, 5) {
		b.write_u8(match s.is_pal.unwrap() {
			true => 1,
			false => 0,
		})?; // 0x1A1
	}

	if ver.gte(2, 0) {
		b.write_u8(match s.is_frozen_ps.unwrap() {
			true => 1,
			false => 0,
		})?; // 0x1A2
	}

	if ver.gte(3, 7) {
		let scene = s.scene.unwrap();
		b.write_u8(scene.minor)?; // 0x1A3
		b.write_u8(scene.major)?; // 0x1A4
	}

	if ver.gte(3, 9) {
		for n in 0..NUM_PORTS {
			// 0x1Fn + 0x1A5
			if let Some(p) = player(s, n) {
				let bytes = p.netplay.as_ref().unwrap().name.bytes();
				if bytes.len() > 30 {
					return Err(err!("netplay name must be no more than 30 bytes"));
				}
				b.write_all(&bytes.to_owned())?;
				b.write_all(&vec![0; 31 - bytes.len()])?;
			} else {
				b = &mut b[31..];
			}
		}
		for n in 0..NUM_PORTS {
			// 0xAn + 0x221
			if let Some(p) = player(s, n) {
				let bytes = p.netplay.as_ref().unwrap().code.bytes();
				if bytes.len() > 9 {
					return Err(err!("netplay code must be no more than 9 bytes"));
				}
				b.write_all(&bytes.to_owned())?;
				b.write_all(&vec![0; 10 - bytes.len()])?;
			} else {
				b = &mut b[10..];
			}
		}
	}

	if ver.gte(3, 11) {
		for n in 0..NUM_PORTS {
			// 0x1Dn + 0x249
			if let Some(p) = player(s, n) {
				let bytes = p
					.netplay
					.as_ref()
					.unwrap()
					.suid
					.as_ref()
					.unwrap()
					.as_bytes();
				if bytes.len() > 28 {
					return Err(err!("netplay SUID must be no more than 28 bytes"));
				}
				b.write_all(&bytes)?;
				b.write_all(&vec![0; 29 - bytes.len()])?;
			} else {
				b = &mut b[29..];
			}
		}
	}

	if ver.gte(3, 12) {
		b.write_u8(s.language.unwrap() as u8)?;
	}

	if ver.gte(3, 14) {
		let m = s.r#match.as_ref().unwrap();
		let bytes = m.id.as_bytes();
		if bytes.len() > 50 {
			return Err(err!("match ID must be no more than 50 bytes"));
		}
		b.write_all(&bytes)?;
		b.write_u8(0)?; // null terminator
		b = &mut b[(50 - bytes.len())..];
		b.write_u32::<BE>(m.game)?;
		b.write_u32::<BE>(m.tiebreaker)?;
	}

	Ok(buf)
}

fn game_start<W: Write>(w: &mut W, s: &game::Start, ver: slippi::Version) -> Result<()> {
	assert_eq!(ver, s.slippi.version);
	w.write_u8(Event::GameStart as u8)?;
	Ok(w.write_all(&_game_start(s)?)?)
}

fn game_end<W: Write>(w: &mut W, e: &game::End, ver: slippi::Version) -> Result<()> {
	w.write_u8(Event::GameEnd as u8)?;
	w.write_u8(e.method as u8)?;
	if ver.gte(2, 0) {
		w.write_i8(match e.lras_initiator.unwrap() {
			Some(x) => x as i8,
			None => -1,
		})?;
		if ver.gte(3, 13) {
			let players = e.players.as_ref().unwrap();
			for n in 0..NUM_PORTS {
				w.write_i8(
					players
						.iter()
						.find(|p| p.port == Port::try_from(n as u8).unwrap())
						.map_or(-1, |p| p.placement as i8),
				)?;
			}
		}
	}
	Ok(())
}

#[derive(Debug)]
struct FrameCounts {
	frame: u32,
	frame_data: u32,
	item: u32,
	fod_platform: u32,
	dreamland_whispy: u32,
	stadium_transformation: u32,
}

fn frame_counts(frames: &Frame) -> FrameCounts {
	let len = frames.len();
	FrameCounts {
		frame: len.try_into().unwrap(),
		frame_data: frames
			.ports
			.iter()
			.map(|p| {
				len - p.leader.validity.as_ref().map_or(0, |v| v.unset_bits())
					+ p.follower.as_ref().map_or(0, |f| {
						len - f.validity.as_ref().map_or(0, |v| v.unset_bits())
					})
			})
			.sum::<usize>()
			.try_into()
			.unwrap(),
		item: frames.item.as_ref().map_or(0, |i| i.id.len() as u32),
		fod_platform: frames
			.fod_platform
			.as_ref()
			.map_or(0, |i| i.platform.len() as u32),
		dreamland_whispy: frames
			.dreamland_whispy
			.as_ref()
			.map_or(0, |i| i.direction.len() as u32),
		stadium_transformation: frames
			.stadium_transformation
			.as_ref()
			.map_or(0, |i| i.event.len() as u32),
	}
}

fn gecko_codes_size(gecko_codes: &GeckoCodes) -> u32 {
	assert_eq!(gecko_codes.bytes.len() % 512, 0);
	let num_blocks = u32::try_from(gecko_codes.bytes.len()).unwrap() / 512;
	num_blocks * (512 + 5)
}

/// Writes a replay to `w` in Slippi (`.slp`) format.
///
/// Returns an error if the game's version is higher than `MAX_SUPPORTED_VERSION`.
pub fn write<W: Write>(w: &mut W, game: &Game) -> Result<()> {
	slippi::assert_max_version(game.start.slippi.version)?;

	let payload_sizes = payload_sizes(game);

	w.write_all(&slippi::FILE_SIGNATURE)?;
	w.write_u32::<BE>(payload_sizes.raw_size(game)?)?;

	w.write_u8(Event::Payloads as u8)?;
	// see "off-by-one" note in `de::parse_payloads`
	w.write_u8((payload_sizes.sizes.len() * 3 + 1).try_into().unwrap())?;
	for (event, size) in payload_sizes.sizes {
		w.write_u8(event)?;
		w.write_u16::<BE>(size)?;
	}

	let ver = game.start.slippi.version;
	game_start(w, &game.start, ver)?;

	if let Some(codes) = &game.gecko_codes {
		gecko_codes(w, codes)?;
	}

	game.frames.write(w, ver)?;

	if let Some(end) = &game.end {
		game_end(w, end, ver)?;
		if game.quirks.map_or(false, |q| q.double_game_end) {
			game_end(w, end, ver)?;
		}
	}

	if let Some(metadata) = &game.metadata {
		w.write_all(&[
			0x55, 0x08, 0x6d, 0x65, 0x74, 0x61, 0x64, 0x61, 0x74, 0x61, 0x7b,
		])?;
		ubjson::write_map(w, metadata)?;
		w.write_all(&[0x7d])?; // closing brace for `metadata`
	}

	w.write_all(&[0x7d])?; // closing brace for top-level map

	Ok(())
}
