use std::io::{Result, Write};

use byteorder::WriteBytesExt;

use crate::{
	frame::immutable::{End, Frame, Item, Post, Pre, Start},
	game::{self, immutable::Game, GeckoCodes},
	io::{
		slippi::{self, de::Event},
		ubjson,
	},
};

type BE = byteorder::BigEndian;

fn payload_sizes(game: &Game) -> Vec<(u8, u16)> {
	// The order of the sizes list is important for round-tripping,
	// which is why we use a Vec rather than a HashMap.
	let mut sizes = Vec::new();
	let start = &game.start;
	let ver = start.slippi.version;

	// +4 bytes for frame number, or +6 for frame number / port / follower
	sizes.push((Event::GameStart as u8, start.bytes.0.len() as u16));
	sizes.push((Event::FramePre as u8, 6 + Pre::size(ver) as u16));
	sizes.push((Event::FramePost as u8, 6 + Post::size(ver) as u16));
	sizes.push((Event::GameEnd as u8, game::End::size(ver) as u16));

	if ver.gte(2, 2) {
		sizes.push((Event::FrameStart as u8, 4 + Start::size(ver) as u16));
		if ver.gte(3, 0) {
			sizes.push((Event::Item as u8, 4 + Item::size(ver) as u16));
			if ver.gte(3, 0) {
				sizes.push((Event::FrameEnd as u8, 4 + End::size(ver) as u16));
				if ver.gte(3, 3) {
					if let Some(codes) = &game.gecko_codes {
						// higher-order bits of actual_size are lost, matching Slippi's behavior
						sizes.push((Event::GeckoCodes as u8, codes.actual_size as u16));
						sizes.push((Event::MessageSplitter as u8, 516));
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
	w.write_all(&s.bytes.0)
}

fn game_end<W: Write>(w: &mut W, e: &game::End, ver: slippi::Version) -> Result<()> {
	w.write_u8(Event::GameEnd as u8)?;
	w.write_u8(e.method as u8)?;
	if ver.gte(2, 0) {
		w.write_u8(e.lras_initiator.unwrap().map_or(u8::MAX, |p| p.into()))?;
		if ver.gte(3, 13) {
			let players = e.players.as_ref().unwrap();
			w.write_all(&players.iter().fold([u8::MAX; 4], |mut acc, p| {
				acc[p.port as usize] = p.placement;
				acc
			}))?;
		}
	}
	Ok(())
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

fn raw_size(game: &Game, payload_sizes: &Vec<(u8, u16)>) -> u32 {
	use Event::*;

	let counts = frame_counts(&game.frames);
	let sizes: std::collections::HashMap<u8, u16> =
		payload_sizes.iter().map(|(k, v)| (*k, *v)).collect();
	1 + 1 + (3 * payload_sizes.len() as u32) // Payload sizes
		+ 1 + sizes[&(GameStart as u8)] as u32 // GameStart
		+ 1 + sizes[&(GameEnd as u8)] as u32 // GameEnd
		+ counts.frame_data * (1 + sizes[&(FramePre as u8)] as u32) // FramePre
		+ counts.frame_data * (1 + sizes[&(FramePost as u8)] as u32) // FramePost
		+ sizes.get(&(FrameStart as u8)).map_or(0, |s| counts.frames * (1 + *s as u32)) // FrameStart
		+ sizes.get(&(FrameEnd as u8)).map_or(0, |s| counts.frames * (1 + *s as u32)) // FrameEnd
		+ sizes.get(&(Item as u8)).map_or(0, |s| counts.items * (1 + *s as u32)) // Item
		+ game.gecko_codes.as_ref().map_or(0, gecko_codes_size)
}

/// Writes a Slippi-format game to `w`.
pub fn write<W: Write>(w: &mut W, game: &Game) -> Result<()> {
	slippi::assert_max_version(game.start.slippi.version)?;

	let payload_sizes = payload_sizes(game);
	let raw_size = raw_size(game, &payload_sizes);

	w.write_all(&slippi::FILE_SIGNATURE)?;
	w.write_u32::<BE>(raw_size)?;

	w.write_u8(Event::Payloads as u8)?;
	w.write_u8((payload_sizes.len() * 3 + 1).try_into().unwrap())?; // see note in `parse::payload_sizes`
	for (event, size) in payload_sizes {
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
