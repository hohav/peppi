use std::{
	io::{Result, Write},
};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::{
	model::{
		frame,
		game::{self, Frames, Game, GeckoCodes},
		item,
		slippi::{self, version as ver},
	},
	serde::de::{PAYLOADS_EVENT_CODE, Event, PortId},
	ubjson,
};

type BE = byteorder::BigEndian;

fn payload_sizes(game: &Game) -> Vec<(u8, u16)> {
	let start = &game.start;
	let v = start.slippi.version;
	let mut sizes = Vec::new();

	sizes.push((Event::GameStart as u8, start.raw_bytes.len() as u16));

	sizes.push((Event::FramePre as u8,
		if v >= ver(1, 4) {
			63
		} else if v >= ver(1, 2) {
			59
		} else {
			58
		}
	));

	sizes.push((Event::FramePost as u8,
		if v >= ver(3, 11) {
			80
		} else if v >= ver(3, 8) {
			76
		} else if v >= ver(3, 5) {
			72
		} else if v >= ver(2, 1) {
			52
		} else if v >= ver(2, 0) {
			51
		} else if v >= ver(0, 2) {
			37
		} else {
			33
		}
	));

	sizes.push((Event::GameEnd as u8, if v >= ver(2, 0) { 2 } else { 1 }));

	if v >= ver(2, 2) {
		sizes.push((Event::FrameStart as u8,
			if v >= ver(3, 10) {
				12
			} else {
				8
			}
		));
	}

	if v >= ver(3, 0) {
		sizes.push((Event::Item as u8,
			if v >= ver(3, 6) {
				42
			} else if v >= ver(3, 2) {
				41
			} else {
				37
			}
		));
	}

	if v >= ver(3, 0) {
		sizes.push((Event::FrameEnd as u8,
			if v >= ver(3, 7) {
				8
			} else {
				4
			}
		));
	}

	if let Some(codes) = &game.gecko_codes {
		sizes.push((Event::GeckoCodes as u8, codes.actual_size));
	}

	if v >= ver(3, 3) {
		sizes.push((0x10, 516)); // Message Splitter
	}

	sizes
}

fn gecko_codes<W: Write>(w: &mut W, codes: &GeckoCodes) -> Result<()> {
	let mut pos = 0;
	let actual_size = codes.actual_size as usize;
	while pos < actual_size {
		w.write_u8(0x10)?; // Message Splitter
		w.write_all(&codes.bytes[pos .. pos + 512])?;
		w.write_u16::<BE>(std::cmp::min(512, actual_size - pos) as u16)?;
		w.write_u8(Event::GeckoCodes as u8)?;
		pos += 512;
		w.write_u8(if pos < actual_size { 0 } else { 1 })?;
	}
	Ok(())
}

fn game_start<W: Write>(w: &mut W, s: &game::Start, v: slippi::Version) -> Result<()> {
	assert_eq!(v, s.slippi.version);
	w.write_u8(Event::GameStart as u8)?;
	w.write_all(&s.raw_bytes)
}

fn game_end<W: Write>(w: &mut W, e: &game::End, v: slippi::Version) -> Result<()> {
	w.write_u8(Event::GameEnd as u8)?;
	w.write_u8(e.method.0)?;
	if v >= ver(2, 0) {
		w.write_u8(e.lras_initiator.unwrap().map(|p| p.into()).unwrap_or(u8::MAX))?;
	}
	Ok(())
}

fn frame_start<W: Write>(w: &mut W, s: &frame::Start, v: slippi::Version, frame_idx: i32) -> Result<()> {
	w.write_u8(Event::FrameStart as u8)?;
	w.write_i32::<BE>(frame_idx)?;
	w.write_u32::<BE>(s.random_seed)?;
	if v >= ver(3, 10) {
		w.write_u32::<BE>(s.scene_frame_counter.unwrap())?;
	}
	Ok(())
}

fn frame_pre<W: Write>(w: &mut W, p: &frame::Pre, v: slippi::Version, id: PortId) -> Result<()> {
	w.write_u8(Event::FramePre as u8)?;
	w.write_i32::<BE>(id.index)?;
	w.write_u8(id.port as u8)?;
	w.write_u8(id.is_follower as u8)?;

	w.write_u32::<BE>(p.random_seed)?;
	w.write_u16::<BE>(p.state.into())?;
	w.write_f32::<BE>(p.position.x)?;
	w.write_f32::<BE>(p.position.y)?;
	w.write_f32::<BE>(p.direction.into())?;
	w.write_f32::<BE>(p.joystick.x)?;
	w.write_f32::<BE>(p.joystick.y)?;
	w.write_f32::<BE>(p.cstick.x)?;
	w.write_f32::<BE>(p.cstick.y)?;
	w.write_f32::<BE>(p.triggers.logical)?;
	w.write_u32::<BE>(p.buttons.logical.0)?;
	w.write_u16::<BE>(p.buttons.physical.0)?;
	w.write_f32::<BE>(p.triggers.physical.l)?;
	w.write_f32::<BE>(p.triggers.physical.r)?;

	if v >= ver(1, 2) {
		w.write_u8(p.raw_analog_x.unwrap())?;
	}

	if v >= ver(1, 4) {
		w.write_f32::<BE>(p.damage.unwrap())?;
	}

	Ok(())
}

fn frame_post<W: Write>(w: &mut W, p: &frame::Post, v: slippi::Version, id: PortId) -> Result<()> {
	w.write_u8(Event::FramePost as u8)?;
	w.write_i32::<BE>(id.index)?;
	w.write_u8(id.port as u8)?;
	w.write_u8(id.is_follower as u8)?;

	w.write_u8(p.character.0)?;
	w.write_u16::<BE>(p.state.into())?;
	w.write_f32::<BE>(p.position.x)?;
	w.write_f32::<BE>(p.position.y)?;
	w.write_f32::<BE>(p.direction.into())?;
	w.write_f32::<BE>(p.damage)?;
	w.write_f32::<BE>(p.shield)?;
	w.write_u8(p.last_attack_landed.map(|a| a.0).unwrap_or(0))?;
	w.write_u8(p.combo_count)?;
	w.write_u8(p.last_hit_by.map(|p| p as u8).unwrap_or(6))?;
	w.write_u8(p.stocks)?;

	if v >= ver(0, 2) {
		w.write_f32::<BE>(p.state_age.unwrap())?;
	}

	if v >= ver(2, 0) {
		let mut buf = [0u8; 8];
		buf.as_mut().write_u64::<LittleEndian>(p.flags.unwrap().0)?;
		w.write_all(&buf[0..5])?;
		w.write_f32::<BE>(p.misc_as.unwrap())?;
		w.write_u8(p.airborne.unwrap() as u8)?;
		w.write_u16::<BE>(p.ground.unwrap().0)?;
		w.write_u8(p.jumps.unwrap())?;
		w.write_u8(match p.l_cancel.unwrap() { Some(true) => 1, Some(false) => 2, _ => 0 })?;
	}

	if v >= ver(2, 1) {
		w.write_u8(p.hurtbox_state.unwrap().0)?;
	}

	if v >= ver(3, 5) {
		let vel = p.velocities.unwrap();
		w.write_f32::<BE>(vel.autogenous_x.air)?;
		w.write_f32::<BE>(vel.autogenous.y)?;
		w.write_f32::<BE>(vel.knockback.x)?;
		w.write_f32::<BE>(vel.knockback.y)?;
		w.write_f32::<BE>(vel.autogenous_x.ground)?;
	}

	if v >= ver(3, 8) {
		w.write_f32::<BE>(p.hitlag.unwrap())?;
	}

	if v >= ver(3, 11) {
		w.write_u32::<BE>(p.animation_index.unwrap())?;
	}

	Ok(())
}

fn item<W: Write>(w: &mut W, i: &item::Item, v: slippi::Version, frame_idx: i32) -> Result<()> {
	w.write_u8(Event::Item as u8)?;
	w.write_i32::<BE>(frame_idx)?;

	w.write_u16::<BE>(i.r#type.0)?;
	w.write_u8(i.state.0)?;
	w.write_f32::<BE>(i.direction.map(|d| d.into()).unwrap_or(0.0))?;
	w.write_f32::<BE>(i.velocity.x)?;
	w.write_f32::<BE>(i.velocity.y)?;
	w.write_f32::<BE>(i.position.x)?;
	w.write_f32::<BE>(i.position.y)?;
	w.write_u16::<BE>(i.damage)?;
	w.write_f32::<BE>(i.timer)?;
	w.write_u32::<BE>(i.id)?;

	if v >= ver(3, 2) {
		w.write_all(&i.misc.unwrap())?;
	}

	if v >= ver(3, 6) {
		w.write_u8(i.owner.unwrap().map(|p| p as u8).unwrap_or(u8::MAX))?;
	}

	Ok(())
}

fn frame_end<W: Write>(w: &mut W, e: &frame::End, v: slippi::Version, frame_idx: i32) -> Result<()> {
	w.write_u8(Event::FrameEnd as u8)?;
	w.write_i32::<BE>(frame_idx)?;
	if v >= ver(3, 7) {
		w.write_i32::<BE>(e.latest_finalized_frame.unwrap())?;
	}
	Ok(())
}

fn frames<W: Write, const N: usize>(w: &mut W, frames: &[frame::Frame<N>], v: slippi::Version) -> Result<()> {
	for f in frames {
		if v >= ver(2, 2) {
			frame_start(w, f.start.as_ref().unwrap(), v, f.index)?;
		}

		let mut port_idx = 0u8;
		for p in &f.ports {
			frame_pre(w, &p.leader.pre, v, PortId::new(f.index, port_idx, false)?)?;
			if let Some(follower) = &p.follower {
				frame_pre(w, &follower.pre, v, PortId::new(f.index, port_idx, true)?)?;
			}
			port_idx += 1;
		}

		if v >= ver(3, 0) {
			for i in f.items.as_ref().unwrap() {
				item(w, i, v, f.index)?;
			}
		}

		port_idx = 0u8;
		for p in &f.ports {
			frame_post(w, &p.leader.post, v, PortId::new(f.index, port_idx, false)?)?;
			if let Some(follower) = &p.follower {
				frame_post(w, &follower.post, v, PortId::new(f.index, port_idx, true)?)?;
			}
			port_idx += 1;
		}

		if v >= ver(3, 0) {
			frame_end(w, f.end.as_ref().unwrap(), v, f.index)?;
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

fn frame_counts<const N: usize>(frames: &Vec<frame::Frame<N>>) -> FrameCounts {
	FrameCounts {
		frames: frames.len().try_into().unwrap(),
		frame_data: frames.iter().map(|f|
			f.ports.iter().map(|p| match p.follower {
				None => 1,
				_ => 2,
			}).sum::<u32>(),
		).sum::<u32>(),
		items: frames.iter().flat_map(|f| f.items.as_ref().map(|i| i.len() as u32)).sum(),
	}
}

fn gecko_codes_size(gecko_codes: &GeckoCodes) -> u32 {
	assert_eq!(gecko_codes.bytes.len() % 512, 0);
	let num_blocks = u32::try_from(gecko_codes.bytes.len()).unwrap() / 512;
	num_blocks * (512 + 5)
}

fn raw_size(game: &Game, payload_sizes: &Vec<(u8, u16)>) -> u32 {
	use Event::*;

	let counts = match &game.frames {
		Frames::P1(f) => frame_counts(f),
		Frames::P2(f) => frame_counts(f),
		Frames::P3(f) => frame_counts(f),
		Frames::P4(f) => frame_counts(f),
	};

	let sizes: std::collections::HashMap<u8, u16> = payload_sizes.iter().map(|(k, v)| (*k, *v)).collect();
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

	w.write_all(
		&[0x7b, 0x55, 0x03, 0x72, 0x61, 0x77, 0x5b, 0x24, 0x55, 0x23, 0x6c])?;
	w.write_u32::<BE>(raw_size)?;

	w.write_u8(PAYLOADS_EVENT_CODE)?;
	w.write_u8((payload_sizes.len() * 3 + 1).try_into().unwrap())?; // see note in `parse::payload_sizes`
	for (event, size) in payload_sizes {
		w.write_u8(event)?;
		w.write_u16::<BE>(size)?;
	}

	let v = game.start.slippi.version;
	game_start(w, &game.start, v)?;

	if let Some(codes) = &game.gecko_codes {
		gecko_codes(w, codes)?;
	}

	match &game.frames {
		Frames::P1(f) => frames(w, f, v)?,
		Frames::P2(f) => frames(w, f, v)?,
		Frames::P3(f) => frames(w, f, v)?,
		Frames::P4(f) => frames(w, f, v)?,
	};

	game_end(w, &game.end, v)?;

	w.write_all(
		&[0x55, 0x08, 0x6d, 0x65, 0x74, 0x61, 0x64, 0x61, 0x74, 0x61, 0x7b])?;
	ubjson::ser::from_map(w, &game.metadata.raw)?;
	w.write_all(&[0x7d])?; // closing brace for `metadata`
	w.write_all(&[0x7d])?; // closing brace for top-level map

	Ok(())
}
