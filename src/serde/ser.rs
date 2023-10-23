use std::io::{Result, Write};

use byteorder::WriteBytesExt;

use crate::{
	model::{
		frame,
		game::{self, Game, GeckoCodes},
		slippi::Version,
	},
	serde::de::{Event, PAYLOADS_EVENT_CODE},
	ubjson,
};

type BE = byteorder::BigEndian;

fn payload_sizes(game: &Game) -> Vec<(u8, u16)> {
	let start = &game.start;
	let ver = start.slippi.version;
	let mut sizes = Vec::new();

	sizes.push((Event::GameStart as u8, start.raw_bytes.len() as u16));

	sizes.push((
		Event::FramePre as u8,
		if ver.gte(1, 4) {
			63
		} else if ver.gte(1, 2) {
			59
		} else {
			58
		},
	));

	sizes.push((
		Event::FramePost as u8,
		if ver.gte(3, 11) {
			80
		} else if ver.gte(3, 8) {
			76
		} else if ver.gte(3, 5) {
			72
		} else if ver.gte(2, 1) {
			52
		} else if ver.gte(2, 0) {
			51
		} else if ver.gte(0, 2) {
			37
		} else {
			33
		},
	));

	sizes.push((Event::GameEnd as u8, if ver.gte(2, 0) { 2 } else { 1 }));

	if ver.gte(2, 2) {
		sizes.push((Event::FrameStart as u8, if ver.gte(3, 10) { 12 } else { 8 }));
	}

	if ver.gte(3, 0) {
		sizes.push((
			Event::Item as u8,
			if ver.gte(3, 6) {
				42
			} else if ver.gte(3, 2) {
				41
			} else {
				37
			},
		));
	}

	if ver.gte(3, 0) {
		sizes.push((Event::FrameEnd as u8, if ver.gte(3, 7) { 8 } else { 4 }));
	}

	if let Some(codes) = &game.gecko_codes {
		// Higher-order bits of actual_size are lost matching slippi-behavior
		sizes.push((Event::GeckoCodes as u8, codes.actual_size as u16));
	}

	if ver.gte(3, 3) {
		sizes.push((0x10, 516)); // Message Splitter
	}

	sizes
}

fn gecko_codes<W: Write>(w: &mut W, codes: &GeckoCodes) -> Result<()> {
	let mut pos = 0;
	let actual_size = codes.actual_size as usize;
	while pos < actual_size {
		w.write_u8(0x10)?; // Message Splitter
		w.write_all(&codes.bytes[pos..pos + 512])?;
		w.write_u16::<BE>(std::cmp::min(512, actual_size - pos) as u16)?;
		w.write_u8(Event::GeckoCodes as u8)?;
		pos += 512;
		w.write_u8(u8::from(pos >= actual_size))?;
	}
	Ok(())
}

fn game_start<W: Write>(w: &mut W, s: &game::Start, ver: Version) -> Result<()> {
	assert_eq!(ver, s.slippi.version);
	w.write_u8(Event::GameStart as u8)?;
	w.write_all(&s.raw_bytes)
}

fn game_end<W: Write>(w: &mut W, e: &game::End, ver: Version) -> Result<()> {
	w.write_u8(Event::GameEnd as u8)?;
	w.write_u8(e.method)?;
	if ver.gte(2, 0) {
		w.write_u8(
			e.lras_initiator
				.unwrap()
				.map(|p| p.into())
				.unwrap_or(u8::MAX),
		)?;
	}
	Ok(())
}

#[derive(Debug)]
struct FrameCounts {
	frames: u32,
	frame_data: u32,
	items: u32,
}

fn frame_counts(frames: &frame::Frame) -> FrameCounts {
	let len = frames.id.len();
	FrameCounts {
		frames: len.try_into().unwrap(),
		frame_data: frames.port.iter().map(|p|
			len + p.follower.as_ref().map(|f|
				len - f.pre.random_seed.validity()
					.map(|v| v.unset_bits())
					.unwrap_or(0)
			).unwrap_or(0)
		).sum::<usize>().try_into().unwrap(),
		items: frames.item.id.len() as u32,
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
		+ sizes.get(&(FrameStart as u8)).map(|s| counts.frames * (1 + *s as u32)).unwrap_or(0) // FrameStart
		+ sizes.get(&(FrameEnd as u8)).map(|s| counts.frames * (1 + *s as u32)).unwrap_or(0) // FrameEnd
		+ sizes.get(&(Item as u8)).map(|s| counts.items * (1 + *s as u32)).unwrap_or(0) // Item
		+ game.gecko_codes.as_ref().map(gecko_codes_size).unwrap_or(0)
}

pub fn serialize<W: Write>(w: &mut W, game: &Game) -> Result<()> {
	let payload_sizes = payload_sizes(game);
	let raw_size = raw_size(game, &payload_sizes);

	w.write_all(&[
		0x7b, 0x55, 0x03, 0x72, 0x61, 0x77, 0x5b, 0x24, 0x55, 0x23, 0x6c,
	])?;
	w.write_u32::<BE>(raw_size)?;

	w.write_u8(PAYLOADS_EVENT_CODE)?;
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
		ubjson::ser::from_map(w, metadata)?;
		w.write_all(&[0x7d])?; // closing brace for `metadata`
	}

	w.write_all(&[0x7d])?; // closing brace for top-level map

	Ok(())
}
