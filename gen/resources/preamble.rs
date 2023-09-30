#![allow(unused_parens)]
#![allow(unused_variables)]

use arrow2::{
	array::{ListArray, MutablePrimitiveArray, PrimitiveArray, StructArray},
	buffer::Buffer,
	datatypes::{DataType, Field},
	offset::{Offsets, OffsetsBuffer},
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Result, Write};

use crate::model::{primitives::Port, slippi::Version};

type BE = byteorder::BigEndian;

#[derive(Clone, Copy, Debug)]
pub struct PortOccupancy {
	pub port: Port,
	pub follower: bool,
}

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
		w.write_u8(0x37)?; // FIXME: don't hard-code
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
		w.write_u8(0x38)?; // FIXME: don't hard-code
		w.write_i32::<BE>(frame_id)?;
		w.write_u8(port.port as u8)?;
		w.write_u8(match port.follower {
			true => 1,
			_ => 0,
		})?;
		self.post.write(w, version, idx)?;
		Ok(())
	}
}

pub struct MutableData {
	pub pre: MutablePre,
	pub post: MutablePost,
}

impl MutableData {
	pub fn with_capacity(capacity: usize, version: Version) -> Self {
		Self {
			pre: MutablePre::with_capacity(capacity, version),
			post: MutablePost::with_capacity(capacity, version),
		}
	}
}

impl From<MutableData> for Data {
	fn from(d: MutableData) -> Self {
		Self {
			pre: d.pre.into(),
			post: d.post.into(),
		}
	}
}

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
				f.write_pre(
					w,
					version,
					idx,
					frame_id,
					PortOccupancy {
						port: self.port,
						follower: false,
					},
				)
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
				f.write_post(
					w,
					version,
					idx,
					frame_id,
					PortOccupancy {
						port: self.port,
						follower: false,
					},
				)
			})
			.unwrap_or(Ok(()))
	}

}

pub struct MutablePortData {
	pub port: Port,
	pub leader: MutableData,
	pub follower: Option<MutableData>,
}

impl MutablePortData {
	pub fn with_capacity(capacity: usize, version: Version, port: PortOccupancy) -> Self {
		Self {
			port: port.port,
			leader: MutableData::with_capacity(capacity, version),
			follower: match port.follower {
				true => Some(MutableData::with_capacity(capacity, version)),
				_ => None,
			},
		}
	}
}

impl From<MutablePortData> for PortData {
	fn from(p: MutablePortData) -> Self {
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
	pub port: Vec<PortData>,
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
						format!("_{}", i),
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
		for i in 0..4u8 {
			// FIXME: don't hard-code
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
			let values: Vec<_> = std::iter::zip(ports, self.port)
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
			port: Self::port_data_from_struct_array(
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
			if version > Version(2, 2, 0) {
				w.write_u8(0x3A)?;
				w.write_i32::<BE>(frame_id)?;
				self.start.write(w, version, idx)?;
			}
			for port in &self.port {
				port.write_pre(w, version, idx, frame_id)?;
			}
			for item_idx in self.item_offset[idx] as usize..self.item_offset[idx + 1] as usize {
				w.write_u8(0x3B)?;
				w.write_i32::<BE>(frame_id)?;
				self.item.write(w, version, item_idx)?;
			}
			for port in &self.port {
				port.write_post(w, version, idx, frame_id)?;
			}
			if version > Version(3, 0, 0) {
				w.write_u8(0x3C)?;
				w.write_i32::<BE>(frame_id)?;
				self.end.write(w, version, idx)?;
			}
		}
		Ok(())
	}
}

pub struct MutableFrame {
	pub id: MutablePrimitiveArray<i32>,
	pub start: MutableStart,
	pub end: MutableEnd,
	pub port: Vec<MutablePortData>,
	pub item_offset: Offsets<i32>,
	pub item: MutableItem,
}

impl MutableFrame {
	pub fn with_capacity(capacity: usize, version: Version, ports: &[PortOccupancy]) -> Self {
		Self {
			id: MutablePrimitiveArray::<i32>::with_capacity(capacity),
			start: MutableStart::with_capacity(capacity, version),
			end: MutableEnd::with_capacity(capacity, version),
			port: ports
				.iter()
				.map(|p| MutablePortData::with_capacity(capacity, version, *p))
				.collect(),
			item_offset: Offsets::<i32>::with_capacity(capacity),
			item: MutableItem::with_capacity(0, version),
		}
	}
}

impl From<MutableFrame> for Frame {
	fn from(f: MutableFrame) -> Self {
		Self {
			id: f.id.into(),
			start: f.start.into(),
			end: f.end.into(),
			port: f.port.into_iter().map(|p| p.into()).collect(),
			item_offset: OffsetsBuffer::try_from(Buffer::from(f.item_offset.into_inner())).unwrap(),
			item: f.item.into(),
		}
	}
}
