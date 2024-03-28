use std::io::Write;

use byteorder::WriteBytesExt;

use crate::{
	frame::immutable::{End, Frame, Item, Post, Pre, Start},
	game::{self, immutable::Game, GeckoCodes},
	io::{
		slippi::{self, de::Event},
		ubjson, Result,
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

	fn raw_size(&self, game: &Game) -> u32 {
		use Event::*;

		let counts = frame_counts(&game.frames);
		let sizes: std::collections::HashMap<u8, u16> =
			self.sizes.iter().map(|(k, v)| (*k, *v)).collect();
		1 + 1 + (3 * self.sizes.len() as u32) // Payload sizes
			+ 1 + sizes[&(GameStart as u8)] as u32 // GameStart
			+ 1 + sizes[&(GameEnd as u8)] as u32 // GameEnd
			+ match game.quirks.map_or(false, |q| q.double_game_end) {
				true => 1 + sizes[&(GameEnd as u8)] as u32, // ...and another for good measure
				_ => 0u32,
			}
			+ counts.frame_data * (1 + sizes[&(FramePre as u8)] as u32) // FramePre
			+ counts.frame_data * (1 + sizes[&(FramePost as u8)] as u32) // FramePost
			+ sizes.get(&(FrameStart as u8)).map_or(0, |s| counts.frames * (1 + *s as u32)) // FrameStart
			+ sizes.get(&(FrameEnd as u8)).map_or(0, |s| counts.frames * (1 + *s as u32)) // FrameEnd
			+ sizes.get(&(Item as u8)).map_or(0, |s| counts.items * (1 + *s as u32)) // Item
			+ game.gecko_codes.as_ref().map_or(0, gecko_codes_size)
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

fn game_start<W: Write>(w: &mut W, s: &game::Start, ver: slippi::Version) -> Result<()> {
	assert_eq!(ver, s.slippi.version);
	w.write_u8(Event::GameStart as u8)?;
	Ok(w.write_all(&s.bytes.0)?)
}

fn game_end<W: Write>(w: &mut W, e: &game::End, _ver: slippi::Version) -> Result<()> {
	w.write_u8(Event::GameEnd as u8)?;
	Ok(w.write_all(&e.bytes.0)?)
}

#[derive(Debug)]
struct FrameCounts {
	frames: u32,
	frame_data: u32,
	items: u32,
}

fn frame_counts(frames: &Frame) -> FrameCounts {
	let len = frames.len();
	FrameCounts {
		frames: len.try_into().unwrap(),
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
		items: frames.item.as_ref().map_or(0, |i| i.id.len() as u32),
	}
}

fn gecko_codes_size(gecko_codes: &GeckoCodes) -> u32 {
	assert_eq!(gecko_codes.bytes.len() % 512, 0);
	let num_blocks = u32::try_from(gecko_codes.bytes.len()).unwrap() / 512;
	num_blocks * (512 + 5)
}

/// Writes a Slippi-format game to `w`.
/// Returns an error if the game's version is higher than `MAX_SUPPORTED_VERSION`.
pub fn write<W: Write>(w: &mut W, game: &Game) -> Result<()> {
	slippi::assert_max_version(game.start.slippi.version)?;

	let payload_sizes = payload_sizes(game);

	w.write_all(&slippi::FILE_SIGNATURE)?;
	w.write_u32::<BE>(payload_sizes.raw_size(game))?;

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
