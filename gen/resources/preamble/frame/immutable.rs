#![allow(unused_variables)]

use std::{
	fmt,
	io::{Result, Write},
};

use byteorder::WriteBytesExt;
use arrow2::{
	array::{ListArray, PrimitiveArray, StructArray},
	bitmap::Bitmap,
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
	pub validity: Option<Bitmap>,
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
		StructArray::new(Self::data_type(version), values, self.validity)
	}

	pub fn from_struct_array(array: StructArray, version: Version) -> Self {
		let (_, values, validity) = array.into_data();
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
			validity: validity,
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
		if self.validity.as_ref().map(|v| v.get_bit(idx)).unwrap_or(true) {
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

	pub fn write_post<W: Write>(
		&self,
		w: &mut W,
		version: Version,
		idx: usize,
		frame_id: i32,
		port: PortOccupancy,
	) -> Result<()> {
		if self.validity.as_ref().map(|v| v.get_bit(idx)).unwrap_or(true) {
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
			validity: d.validity.map(|v| v.into()),
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
		let (fields, values, _) = array.into_data();
		assert_eq!("leader", fields[0].name);
		fields.get(1).map(|f| assert_eq!("follower", f.name));
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
	pub ports: Vec<PortData>,
	pub start: Option<Start>,
	pub end: Option<End>,
	pub item_offset: Option<OffsetsBuffer<i32>>,
	pub item: Option<Item>,
}

impl Frame {
	pub fn len(&self) -> usize {
		self.id.len()
	}

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
		let mut fields = vec![
			Field::new("id", DataType::Int32, false),
			Field::new("port", Self::port_data_type(version, ports).clone(), false),
		];
		if version.gte(2, 2) {
			fields.push(Field::new("start", Start::data_type(version).clone(), false));
			if version.gte(3, 0) {
				fields.push(Field::new("end", End::data_type(version).clone(), false));
				fields.push(Field::new("item", Self::item_data_type(version).clone(), false));
			}
		}
		DataType::Struct(fields)
	}

	pub fn into_struct_array(self, version: Version, ports: &[PortOccupancy]) -> StructArray {
		let port = {
			let values: Vec<_> = std::iter::zip(ports, self.ports)
				.map(|(occupancy, data)| data.into_struct_array(version, *occupancy).boxed())
				.collect();
			StructArray::new(Self::port_data_type(version, ports), values, None).boxed()
		};

		let mut arrays = vec![self.id.boxed(), port];

		if version.gte(2, 2) {
			arrays.push(self.start.unwrap().into_struct_array(version).boxed());
			if version.gte(3, 0) {
				arrays.push(self.end.unwrap().into_struct_array(version).boxed());
				let item_values = self.item.unwrap().into_struct_array(version).boxed();
				arrays.push(ListArray::new(
					Self::item_data_type(version),
					self.item_offset.unwrap(),
					item_values,
					None,
				).boxed());
			}
		}

		StructArray::new(Self::data_type(version, ports), arrays, None)
	}

	pub fn from_struct_array(array: StructArray, version: Version) -> Self {
		let (fields, values, _) = array.into_data();
		assert_eq!("id", fields[0].name);
		assert_eq!("port", fields[1].name);
		if version.gte(2, 2) {
			assert_eq!("start", fields[2].name);
			if version.gte(3, 0) {
				assert_eq!("end", fields[3].name);
				assert_eq!("item", fields[4].name);
			}
		}

		let (item, item_offset) = values.get(4).map(|v| {
			let arrays = v.as_any()
				.downcast_ref::<ListArray<i32>>()
				.unwrap()
				.clone();
			let item_offset = arrays.offsets().clone();
			let item = Item::from_struct_array(
				arrays.values()
					.as_any()
					.downcast_ref::<StructArray>()
					.unwrap()
					.clone(),
				version,
			);
			(Some(item), Some(item_offset))
		}).unwrap_or((None, None));

		Self {
			id: values[0]
				.as_any()
				.downcast_ref::<PrimitiveArray<i32>>()
				.unwrap()
				.clone(),
			ports: Self::port_data_from_struct_array(
				values[1]
					.as_any()
					.downcast_ref::<StructArray>()
					.unwrap()
					.clone(),
				version,
			),
			start: values.get(2).map(|v|
				Start::from_struct_array(
					v.as_any()
						.downcast_ref::<StructArray>()
						.unwrap()
						.clone(),
						version,
				)
			),
			end: values.get(3).map(|v|
				End::from_struct_array(
					v.as_any()
						.downcast_ref::<StructArray>()
						.unwrap()
						.clone(),
						version,
				)
			),
			item_offset: item_offset,
			item: item,
		}
	}

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

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::Frame {
		transpose::Frame {
			id: self.id.values()[i],
			ports: self.ports.iter().map(|p| p.transpose_one(i, version)).collect(),
			start: version.gte(2, 2).then(||
				self.start.as_ref().unwrap().transpose_one(i, version),
			),
			end: version.gte(3, 0).then(||
				self.end.as_ref().unwrap().transpose_one(i, version),
			),
			items: version.gte(3, 0).then(|| {
				let (start, end) = self.item_offset.as_ref().unwrap().start_end(i);
				(start..end)
					.map(|i| self.item.as_ref().unwrap().transpose_one(i, version))
					.collect()
			}),
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
			ports: f.ports.into_iter().map(|p| p.into()).collect(),
			start: f.start.map(|x| x.into()),
			end: f.end.map(|x| x.into()),
			item_offset: f.item_offset.map(|x|
				OffsetsBuffer::try_from(Buffer::from(x.into_inner())).unwrap()
			),
			item: f.item.map(|x| x.into()),
		}
	}
}

impl fmt::Debug for Frame {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
		write!(f, "Frame {{ len: {} }}", self.id.len())
	}
}
