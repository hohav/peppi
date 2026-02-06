#![allow(unused_variables)]

use std::{
	io::{Result, Write},
	mem::size_of,
};

use byteorder::WriteBytesExt;

use crate::{
	frame::{
		immutable::{Data, Frame, PortData},
		PortOccupancy,
	},
	io::slippi::{Version, STAGE_EVENTS_VERSION, de::Event},
};

type BE = byteorder::BigEndian;

impl Data {
	fn write_pre<W: Write>(
		&self,
		w: &mut W,
		version: Version,
		idx: usize,
		frame_id: i32,
		port: PortOccupancy,
	) -> Result<()> {
		if self.validity.as_ref().map_or(true, |v| v.get_bit(idx)) {
			w.write_u8(Event::FramePre as u8)?;
			w.write_i32::<BE>(frame_id)?;
			w.write_u8(port.port as u8)?;
			w.write_u8(match port.follower {
				true => 1,
				_ => 0,
			})?;
			self.pre.write(w, version, idx)?;
		}
		Ok(())
	}

	fn write_post<W: Write>(
		&self,
		w: &mut W,
		version: Version,
		idx: usize,
		frame_id: i32,
		port: PortOccupancy,
	) -> Result<()> {
		if self.validity.as_ref().map_or(true, |v| v.get_bit(idx)) {
			w.write_u8(Event::FramePost as u8)?;
			w.write_i32::<BE>(frame_id)?;
			w.write_u8(port.port as u8)?;
			w.write_u8(match port.follower {
				true => 1,
				_ => 0,
			})?;
			self.post.write(w, version, idx)?;
		}
		Ok(())
	}
}

impl PortData {
	fn write_pre<W: Write>(
		&self,
		w: &mut W,
		version: Version,
		idx: usize,
		frame_id: i32,
	) -> Result<()> {
		self.leader.write_pre(
			w,
			version,
			idx,
			frame_id,
			PortOccupancy {
				port: self.port,
				follower: false,
			},
		)?;
		self.follower
			.as_ref()
			.map_or(Ok(()), |f| {
				if f.validity.as_ref().map_or(true, |v| v.get_bit(idx)) {
					f.write_pre(
						w,
						version,
						idx,
						frame_id,
						PortOccupancy {
							port: self.port,
							follower: true,
						},
					)
				} else {
					Ok(())
				}
			})
	}

	fn write_post<W: Write>(
		&self,
		w: &mut W,
		version: Version,
		idx: usize,
		frame_id: i32,
	) -> Result<()> {
		self.leader.write_post(
			w,
			version,
			idx,
			frame_id,
			PortOccupancy {
				port: self.port,
				follower: false,
			},
		)?;
		self.follower
			.as_ref()
			.map_or(Ok(()), |f| {
				if f.validity.as_ref().map_or(true, |v| v.get_bit(idx)) {
					f.write_post(
						w,
						version,
						idx,
						frame_id,
						PortOccupancy {
							port: self.port,
							follower: true,
						},
					)
				} else {
					Ok(())
				}
			})
	}
}

impl Frame {
	pub fn write<W: Write>(&self, w: &mut W, version: Version) -> Result<()> {
		for (idx, &frame_id) in self.id.values().iter().enumerate() {
			if version.gte(2, 2) {
				w.write_u8(Event::FrameStart as u8)?;
				w.write_i32::<BE>(frame_id)?;
				self.start.as_ref().unwrap().write(w, version, idx)?;
			}
			for port in &self.ports {
				port.write_pre(w, version, idx, frame_id)?;
			}
			if version >= STAGE_EVENTS_VERSION {
				// FOD platform
				let offset = self.fod_platform_offset.as_ref().unwrap();
				for evt_idx in (offset[idx] as usize)..(offset[idx + 1] as usize) {
					w.write_u8(Event::FodPlatform as u8)?;
					w.write_i32::<BE>(frame_id)?;
					self.fod_platform.as_ref().unwrap().write(w, version, evt_idx)?;
				}

				// Dreamland Whispy
				let offset = self.dreamland_whispy_offset.as_ref().unwrap();
				for evt_idx in (offset[idx] as usize)..(offset[idx + 1] as usize) {
					w.write_u8(Event::DreamlandWhispy as u8)?;
					w.write_i32::<BE>(frame_id)?;
					self.dreamland_whispy.as_ref().unwrap().write(w, version, evt_idx)?;
				}

				// Stadium transformation
				let offset = self.stadium_transformation_offset.as_ref().unwrap();
				for evt_idx in (offset[idx] as usize)..(offset[idx + 1] as usize) {
					w.write_u8(Event::StadiumTransformation as u8)?;
					w.write_i32::<BE>(frame_id)?;
					self.stadium_transformation.as_ref().unwrap().write(w, version, evt_idx)?;
				}
			}
			if version.gte(3, 0) {
				let offset = self.item_offset.as_ref().unwrap();
				for item_idx in (offset[idx] as usize)..(offset[idx + 1] as usize) {
					w.write_u8(Event::Item as u8)?;
					w.write_i32::<BE>(frame_id)?;
					self.item.as_ref().unwrap().write(w, version, item_idx)?;
				}
			}
			for port in &self.ports {
				port.write_post(w, version, idx, frame_id)?;
			}
			if version.gte(3, 0) {
				w.write_u8(Event::FrameEnd as u8)?;
				w.write_i32::<BE>(frame_id)?;
				self.end.as_ref().unwrap().write(w, version, idx)?;
			}
		}
		Ok(())
	}
}
