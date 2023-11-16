#![allow(unused_variables)]

use std::{
	fmt,
	io::{Result, Write},
};

use byteorder::WriteBytesExt;
use arrow2::{
	array::{ListArray, PrimitiveArray, StructArray},
	buffer::Buffer,
	datatypes::{DataType, Field},
	offset::OffsetsBuffer,
};

use crate::{
	model::{
		frame::{self, mutable, transpose, PortOccupancy},
		game::{Port, NUM_PORTS},
		slippi::Version,
	},
	serde::de::Event,
};

type BE = byteorder::BigEndian;

#[derive(Debug)]
pub struct Data {
	pub pre: Pre,
	pub post: Post,
}

impl Data {
	pub fn data_type(version: Version) -> DataType {
		DataType::Struct(vec![
			Field::new("pre", Pre::data_type(version).clone(), false),
			Field::new("post", Post::data_type(version).clone(), false),
		])
	}

	pub fn into_struct_array(self, version: Version) -> StructArray {
		let values = vec![
			self.pre.into_struct_array(version).boxed(),
			self.post.into_struct_array(version).boxed(),
		];
		StructArray::new(Self::data_type(version), values, None)
	}

	pub fn from_struct_array(array: StructArray, version: Version) -> Self {
		let (_, values, _) = array.into_data();
		Self {
			pre: Pre::from_struct_array(
				values[0]
					.as_any()
					.downcast_ref::<StructArray>()
					.unwrap()
					.clone(),
				version,
			),
			post: Post::from_struct_array(
				values[1]
					.as_any()
					.downcast_ref::<StructArray>()
					.unwrap()
					.clone(),
				version,
			),
		}
	}

	pub fn write_pre<W: Write>(
		&self,
		w: &mut W,
		version: Version,
		idx: usize,
		frame_id: i32,
		port: PortOccupancy,
	) -> Result<()> {
		w.write_u8(Event::FramePre as u8)?;
		w.write_i32::<BE>(frame_id)?;
		w.write_u8(port.port as u8)?;
		w.write_u8(match port.follower {
			true => 1,
			_ => 0,
		})?;
		self.pre.write(w, version, idx)?;
		Ok(())
	}

	pub fn write_post<W: Write>(
		&self,
		w: &mut W,
		version: Version,
		idx: usize,
		frame_id: i32,
		port: PortOccupancy,
	) -> Result<()> {
		w.write_u8(Event::FramePost as u8)?;
		w.write_i32::<BE>(frame_id)?;
		w.write_u8(port.port as u8)?;
		w.write_u8(match port.follower {
			true => 1,
			_ => 0,
		})?;
		self.post.write(w, version, idx)?;
		Ok(())
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::Data {
		transpose::Data {
			pre: self.pre.transpose_one(i, version),
			post: self.post.transpose_one(i, version),
		}
	}
}

impl From<mutable::Data> for Data {
	fn from(d: mutable::Data) -> Self {
		Self {
			pre: d.pre.into(),
			post: d.post.into(),
		}
	}
}

#[derive(Debug)]
pub struct PortData {
	pub port: Port,
	pub leader: Data,
	pub follower: Option<Data>,
}

impl PortData {
	pub fn data_type(version: Version, port: PortOccupancy) -> DataType {
		let mut fields = vec![Field::new(
			"leader",
			Data::data_type(version).clone(),
			false,
		)];
		if port.follower {
			fields.push(Field::new(
				"follower",
				Data::data_type(version).clone(),
				false,
			));
		}
		DataType::Struct(fields)
	}

	pub fn into_struct_array(self, version: Version, port: PortOccupancy) -> StructArray {
		let mut values = vec![self.leader.into_struct_array(version).boxed()];
		if let Some(follower) = self.follower {
			values.push(follower.into_struct_array(version).boxed());
		}
		StructArray::new(Self::data_type(version, port), values, None)
	}

	pub fn from_struct_array(array: StructArray, version: Version, port: Port) -> Self {
		let (_, values, _) = array.into_data();
		Self {
			port: port,
			leader: Data::from_struct_array(
				values[0]
					.as_any()
					.downcast_ref::<StructArray>()
					.unwrap()
					.clone(),
				version,
			),
			follower: values.get(1).map(|x| {
				Data::from_struct_array(
					x.as_any().downcast_ref::<StructArray>().unwrap().clone(),
					version,
				)
			}),
		}
	}

	pub fn write_pre<W: Write>(
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
			.map(|f| {
				if f.pre.random_seed.validity().map(|v| v.get_bit(idx)).unwrap_or(true) {
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
			.unwrap_or(Ok(()))
	}

	pub fn write_post<W: Write>(
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
			.map(|f| {
				if f.pre.random_seed.validity().map(|v| v.get_bit(idx)).unwrap_or(true) {
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
			.unwrap_or(Ok(()))
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::PortData {
		transpose::PortData {
			port: self.port,
			leader: self.leader.transpose_one(i, version),
			follower: self.follower.as_ref().map(|f| f.transpose_one(i, version)),
		}
	}
}

impl From<mutable::PortData> for PortData {
	fn from(p: mutable::PortData) -> Self {
		Self {
			port: p.port,
			leader: p.leader.into(),
			follower: p.follower.map(|f| f.into()),
		}
	}
}

pub struct Frame {
	pub id: PrimitiveArray<i32>,
	pub start: Start,
	pub end: End,
	pub ports: Vec<PortData>,
	pub item_offset: OffsetsBuffer<i32>,
	pub item: Item,
}

impl Frame {
	pub fn port_data_type(version: Version, ports: &[PortOccupancy]) -> DataType {
		DataType::Struct(
			ports
				.iter()
				.enumerate()
				.map(|(i, p)| {
					Field::new(
						format!("{}", i),
						PortData::data_type(version, *p).clone(),
						false,
					)
				})
				.collect(),
		)
	}

	pub fn port_data_from_struct_array(array: StructArray, version: Version) -> Vec<PortData> {
		let (_, values, _) = array.into_data();
		let mut ports = vec![];
		for i in 0 .. NUM_PORTS {
			if let Some(a) = values.get(i as usize) {
				ports.push(PortData::from_struct_array(
					a.as_any().downcast_ref::<StructArray>().unwrap().clone(),
					version,
					Port::try_from(i).unwrap(),
				));
			}
		}
		ports
	}

	pub fn item_data_type(version: Version) -> DataType {
		DataType::List(Box::new(Field::new(
			"item",
			Item::data_type(version),
			false,
		)))
	}

	pub fn data_type(version: Version, ports: &[PortOccupancy]) -> DataType {
		DataType::Struct(vec![
			Field::new("id", DataType::Int32, false),
			Field::new("start", Start::data_type(version).clone(), false),
			Field::new("end", End::data_type(version).clone(), false),
			Field::new("port", Self::port_data_type(version, ports).clone(), false),
			Field::new("item", Self::item_data_type(version).clone(), false),
		])
	}

	pub fn into_struct_array(self, version: Version, ports: &[PortOccupancy]) -> StructArray {
		let start = self.start.into_struct_array(version).boxed();
		let end = self.end.into_struct_array(version).boxed();

		let port = {
			let values: Vec<_> = std::iter::zip(ports, self.ports)
				.map(|(occupancy, data)| data.into_struct_array(version, *occupancy).boxed())
				.collect();
			StructArray::new(Self::port_data_type(version, ports), values, None).boxed()
		};

		let item = {
			let values = self.item.into_struct_array(version).boxed();
			ListArray::new(
				Self::item_data_type(version),
				self.item_offset,
				values,
				None,
			)
			.boxed()
		};

		StructArray::new(
			Self::data_type(version, ports),
			vec![self.id.boxed(), start, end, port, item],
			None,
		)
	}

	pub fn from_struct_array(array: StructArray, version: Version) -> Self {
		let (_, values, _) = array.into_data();
		let item_arrays = values[4]
			.as_any()
			.downcast_ref::<ListArray<i32>>()
			.unwrap()
			.clone();
		let item = item_arrays.values();
		let item_offset = item_arrays.offsets();

		Self {
			id: values[0]
				.as_any()
				.downcast_ref::<PrimitiveArray<i32>>()
				.unwrap()
				.clone(),
			start: Start::from_struct_array(
				values[1]
					.as_any()
					.downcast_ref::<StructArray>()
					.unwrap()
					.clone(),
				version,
			),
			end: End::from_struct_array(
				values[2]
					.as_any()
					.downcast_ref::<StructArray>()
					.unwrap()
					.clone(),
				version,
			),
			ports: Self::port_data_from_struct_array(
				values[3]
					.as_any()
					.downcast_ref::<StructArray>()
					.unwrap()
					.clone(),
				version,
			),
			item_offset: item_offset.clone(),
			item: Item::from_struct_array(
				item.as_any().downcast_ref::<StructArray>().unwrap().clone(),
				version,
			),
		}
	}

	pub fn write<W: Write>(&self, w: &mut W, version: Version) -> Result<()> {
		for (idx, &frame_id) in self.id.values().iter().enumerate() {
			if version.gte(2, 2) {
				w.write_u8(Event::FrameStart as u8)?;
				w.write_i32::<BE>(frame_id)?;
				self.start.write(w, version, idx)?;
			}
			for port in &self.ports {
				port.write_pre(w, version, idx, frame_id)?;
			}
			if version.gte(3, 0) {
				for item_idx in self.item_offset[idx] as usize .. self.item_offset[idx + 1] as usize {
					w.write_u8(Event::Item as u8)?;
					w.write_i32::<BE>(frame_id)?;
					self.item.write(w, version, item_idx)?;
				}
			}
			for port in &self.ports {
				port.write_post(w, version, idx, frame_id)?;
			}
			if version.gte(3, 0) {
				w.write_u8(Event::FrameEnd as u8)?;
				w.write_i32::<BE>(frame_id)?;
				self.end.write(w, version, idx)?;
			}
		}
		Ok(())
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::Frame {
		transpose::Frame {
			id: self.id.values()[i],
			start: self.start.transpose_one(i, version),
			end: self.end.transpose_one(i, version),
			ports: self.ports.iter().map(|p| p.transpose_one(i, version)).collect(),
		}
	}

	pub fn rollback_indexes_initial(&self) -> Vec<usize> {
		self.rollback_indexes(self.id.values().as_slice().iter().enumerate())
	}

	pub fn rollback_indexes_final(&self) -> Vec<usize> {
		let mut result = self.rollback_indexes(self.id.values().as_slice().iter().enumerate().rev());
		result.reverse();
		result
	}

	fn rollback_indexes<'a>(&self, ids: impl Iterator<Item=(usize, &'a i32)>) -> Vec<usize> {
		let mut result = vec![];
		let mut seen_ids = vec![false; self.id.len()];
		for (idx, id) in ids {
			let zero_based_id = usize::try_from(id - frame::FIRST_INDEX).unwrap();
			if !seen_ids[zero_based_id] {
				seen_ids[zero_based_id] = true;
				result.push(idx);
			}
		}
		result
	}
}

impl From<mutable::Frame> for Frame {
	fn from(f: mutable::Frame) -> Self {
		Self {
			id: f.id.into(),
			start: f.start.into(),
			end: f.end.into(),
			ports: f.ports.into_iter().map(|p| p.into()).collect(),
			item_offset: OffsetsBuffer::try_from(Buffer::from(f.item_offset.into_inner())).unwrap(),
			item: f.item.into(),
		}
	}
}

impl fmt::Debug for Frame {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
		write!(f, "Frame {{ len: {} }}", self.id.len())
	}
}
