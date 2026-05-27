#![allow(unused_variables)]

use std::{
	io::{Result, Write},
	mem::size_of,
};

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::{
	frame::{Data, Frame, Frames, Port, PortData, PortOccupancy, Reader, Writer},
	io::slippi::{de::Event, Version},
};

type BE = byteorder::BigEndian;

impl Data {
	fn write_pre<W: Write>(
		&self,
		w: &mut W,
		version: Version,
		idx: usize,
		frame_id: i32,
		occ: PortOccupancy,
	) -> Result<()> {
		if self.validity.is_valid(idx) {
			w.write_u8(Event::FramePre as u8)?;
			w.write_i32::<BE>(frame_id)?;
			w.write_u8(occ.port as u8)?;
			w.write_u8(match occ.follower {
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
		occ: PortOccupancy,
	) -> Result<()> {
		if self.validity.is_valid(idx) {
			w.write_u8(Event::FramePost as u8)?;
			w.write_i32::<BE>(frame_id)?;
			w.write_u8(occ.port as u8)?;
			w.write_u8(match occ.follower {
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
		if self.validity.is_valid(idx) {
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
			self.follower.as_ref().map_or(Ok(()), |f| {
				if f.validity.is_valid(idx) {
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
		} else {
			Ok(())
		}
	}

	fn write_post<W: Write>(
		&self,
		w: &mut W,
		version: Version,
		idx: usize,
		frame_id: i32,
	) -> Result<()> {
		if self.validity.is_valid(idx) {
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
			self.follower.as_ref().map_or(Ok(()), |f| {
				if f.validity.is_valid(idx) {
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
		} else {
			Ok(())
		}
	}
}

impl Writer for Frame {
	fn write<W: Write>(&self, w: &mut W, version: Version) -> Result<()> {
		for (idx, &frame_id) in self.id.iter().enumerate() {
			if version.gte(2, 2) {
				w.write_u8(Event::FrameStart as u8)?;
				w.write_i32::<BE>(frame_id)?;
				self.start.as_ref().unwrap().write(w, version, idx)?;
			}
			for port in &self.ports {
				port.write_pre(w, version, idx, frame_id)?;
			}
			if version.gte(3, 18) {
				// FOD platform
				let offset = self.fod_platform_offset.as_ref().unwrap();
				for evt_idx in (offset[idx] as usize)..(offset[idx + 1] as usize) {
					w.write_u8(Event::FodPlatform as u8)?;
					w.write_i32::<BE>(frame_id)?;
					self.fod_platform
						.as_ref()
						.unwrap()
						.write(w, version, evt_idx)?;
				}

				// Dreamland Whispy
				let offset = self.dreamland_whispy_offset.as_ref().unwrap();
				for evt_idx in (offset[idx] as usize)..(offset[idx + 1] as usize) {
					w.write_u8(Event::DreamlandWhispy as u8)?;
					w.write_i32::<BE>(frame_id)?;
					self.dreamland_whispy
						.as_ref()
						.unwrap()
						.write(w, version, evt_idx)?;
				}

				// Stadium transformation
				let offset = self.stadium_transformation_offset.as_ref().unwrap();
				for evt_idx in (offset[idx] as usize)..(offset[idx + 1] as usize) {
					w.write_u8(Event::StadiumTransformation as u8)?;
					w.write_i32::<BE>(frame_id)?;
					self.stadium_transformation
						.as_ref()
						.unwrap()
						.write(w, version, evt_idx)?;
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

impl Reader for Frame {
	fn open(&mut self, version: Version, id: i32, port_count: usize) {
		self.id.push(id);
	}

	fn close(&mut self, version: Version, port_count: usize) {
		let len = self.len();
		for p in &mut self.ports {
			while p.len() < len {
				p.append_null(version);
			}
			if let Some(f) = &mut p.follower {
				while f.len() < len {
					f.append_null(version);
				}
			}
		}

		if version.gte(3, 0) {
			self.item_offset.as_mut().unwrap().push(
				self.item.as_ref().unwrap().id.len().try_into().unwrap(),
			);

			if version.gte(3, 18) {
				self.fod_platform_offset.as_mut().unwrap().push(
					self.fod_platform.as_ref().unwrap().platform.len().try_into().unwrap()
				);
				self.dreamland_whispy_offset.as_mut().unwrap().push(
					self.dreamland_whispy.as_ref().unwrap().direction.len().try_into().unwrap()
				);
				self.stadium_transformation_offset.as_mut().unwrap().push(
					self.stadium_transformation.as_ref().unwrap().event.len().try_into().unwrap()
				);
			}
		}
	}

	fn read_start(&mut self, r: &mut &[u8], version: Version) -> Result<()> {
		self.start.as_mut().unwrap().read_append(r, version)
	}

	fn read_pre(&mut self, r: &mut &[u8], version: Version, id: i32, port_index: u8, port: Port, follower: bool) -> Result<()> {
		let port_data = &mut self.ports[port_index as usize];
		if !follower {
			port_data.validity.push(true);
		}
		let character = match follower {
			true => port_data.follower.as_mut().unwrap(),
			_ => &mut port_data.leader,
		};
		character.validity.push(true);
		character.pre.read_append(r, version)
	}

	fn read_post(&mut self, r: &mut &[u8], version: Version, id: i32, port_index: u8, port: Port, follower: bool) -> Result<()> {
		let port_data = &mut self.ports[port_index as usize];
		let character = match follower {
			true => port_data.follower.as_mut().unwrap(),
			_ => &mut port_data.leader,
		};
		character.post.read_append(r, version)
	}

	fn read_item(&mut self, r: &mut &[u8], version: Version) -> Result<()> {
		self.item.as_mut().unwrap().read_append(r, version)
	}

	fn read_fod_platform(&mut self, r: &mut &[u8], version: Version) -> Result<()> {
		self.fod_platform.as_mut().unwrap().read_append(r, version)
	}

	fn read_dreamland_whispy(&mut self, r: &mut &[u8], version: Version) -> Result<()> {
		self.dreamland_whispy.as_mut().unwrap().read_append(r, version)
	}

	fn read_stadium_transformation(&mut self, r: &mut &[u8], version: Version) -> Result<()> {
		self.stadium_transformation.as_mut().unwrap().read_append(r, version)
	}

	fn read_end(&mut self, r: &mut &[u8], version: Version) -> Result<()> {
		self.end.as_mut().unwrap().read_append(r, version)
	}
}
