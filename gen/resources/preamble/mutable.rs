//! Mutable (in-progress) frame data.
//!
//! You’ll only encounter mutable frame data if you’re parsing live games.

#![allow(unused_variables)]
#![allow(dead_code)]

use arrow::array::{
	types::{Float32Type, Int32Type, Int8Type, UInt16Type, UInt32Type, UInt8Type},
	ArrayBuilder, ArrowPrimitiveType, PrimitiveBuilder,
};
use arrow_buffer::builder::{NullBufferBuilder, OffsetBufferBuilder};

use byteorder::ReadBytesExt;
use std::io::Result;

use crate::{
	frame::{immutable, transpose, PortOccupancy},
	game::Port,
	io::slippi::Version,
};

type BE = byteorder::BigEndian;

trait Valued<T> {
	fn value(&self, i: usize) -> T;
}

impl<T: ArrowPrimitiveType> Valued<<T as ArrowPrimitiveType>::Native> for PrimitiveBuilder<T> {
	fn value(&self, i: usize) -> <T as ArrowPrimitiveType>::Native {
		self.values_slice()[i]
	}
}

/// Frame data for a single character (ICs are two characters).
pub struct Data {
	pub pre: Pre,
	pub post: Post,
	pub validity: NullBufferBuilder,
}

impl Data {
	pub fn with_capacity(capacity: usize, version: Version) -> Self {
		Self {
			pre: Pre::with_capacity(capacity, version),
			post: Post::with_capacity(capacity, version),
			validity: NullBufferBuilder::new(capacity),
		}
	}

	pub fn len(&self) -> usize {
		self.pre.len()
	}

	pub fn push_null(&mut self, version: Version) {
		let len = self.len();
		self.validity.append(false);
		self.pre.push_default(version);
		self.post.push_default(version);
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::Data {
		transpose::Data {
			pre: self.pre.transpose_one(i, version),
			post: self.post.transpose_one(i, version),
		}
	}

	pub fn finish(&mut self) -> immutable::Data {
		immutable::Data {
			pre: self.pre.finish(),
			post: self.post.finish(),
			validity: self.validity.finish(),
		}
	}
}

/// Frame data for a single port.
pub struct PortData {
	pub port: Port,
	pub leader: Data,
	/// The "backup" ICs character
	pub follower: Option<Data>,
}

impl PortData {
	pub fn with_capacity(capacity: usize, version: Version, port: PortOccupancy) -> Self {
		Self {
			port: port.port,
			leader: Data::with_capacity(capacity, version),
			follower: match port.follower {
				true => Some(Data::with_capacity(capacity, version)),
				_ => None,
			},
		}
	}

	pub fn len(&self) -> usize {
		self.leader.len()
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::PortData {
		transpose::PortData {
			port: self.port,
			leader: self.leader.transpose_one(i, version),
			follower: self.follower.as_ref().map(|f| f.transpose_one(i, version)),
		}
	}

	pub fn finish(&mut self) -> immutable::PortData {
		immutable::PortData {
			port: self.port,
			leader: self.leader.finish(),
			follower: self.follower.as_mut().map(|f| f.finish()),
		}
	}
}

/// All frame data for a single game, in struct-of-arrays format.
pub struct Frame {
	/// Frame IDs start at `-123` and increment each frame. May repeat in case of rollbacks
	pub id: PrimitiveBuilder<Int32Type>,
	/// Port-specific data
	pub ports: Vec<PortData>,
	/// Start-of-frame data
	pub start: Option<Start>,
	/// End-of-frame data
	pub end: Option<End>,
	/// Item data. Logically, each frame has its own array of items. But we represent all item data in a flat array, with `item_offset` indicating the start of each frame's sub-array
	pub item: Option<Item>,
	/// Item array offsets (see `item`)
	pub item_offset: Option<OffsetBufferBuilder<i32>>,
}

impl Frame {
	pub fn with_capacity(capacity: usize, version: Version, ports: &[PortOccupancy]) -> Self {
		Self {
			id: PrimitiveBuilder::with_capacity(capacity),
			ports: ports
				.iter()
				.map(|p| PortData::with_capacity(capacity, version, *p))
				.collect(),
			start: version
				.gte(2, 2)
				.then(|| Start::with_capacity(capacity, version)),
			end: version
				.gte(3, 0)
				.then(|| End::with_capacity(capacity, version)),
			item_offset: version
				.gte(3, 0)
				.then(|| OffsetBufferBuilder::<i32>::new(capacity)),
			item: version.gte(3, 0).then(|| Item::with_capacity(0, version)),
		}
	}

	pub fn len(&self) -> usize {
		self.id.len()
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::Frame {
		transpose::Frame {
			id: self.id.values_slice()[i],
			ports: self
				.ports
				.iter()
				.map(|p| p.transpose_one(i, version))
				.collect(),
			start: version
				.gte(2, 2)
				.then(|| self.start.as_ref().unwrap().transpose_one(i, version)),
			end: version
				.gte(3, 0)
				.then(|| self.end.as_ref().unwrap().transpose_one(i, version)),
			items: version.gte(3, 0).then(|| {
				let offsets = self.item_offset.as_ref().unwrap();
				let [start, end] = (*offsets)[i..i + 2] else {
					panic!("not enough item offsets: {}/{}", i, offsets.len());
				};
				(usize::try_from(start).unwrap()..usize::try_from(end).unwrap())
					.map(|i| self.item.as_ref().unwrap().transpose_one(i, version))
					.collect()
			}),
		}
	}

	/// Builds an `immutable::Frame`, resetting self.
	pub fn finish(&mut self) -> immutable::Frame {
		let item_offset = self.item_offset.take();
		if item_offset.is_some() {
			self.item_offset = Some(OffsetBufferBuilder::new(0));
		}
		immutable::Frame {
			id: self.id.finish(),
			ports: self.ports.iter_mut().map(|p| p.finish()).collect(),
			start: self.start.as_mut().map(|x| x.finish()),
			end: self.end.as_mut().map(|x| x.finish()),
			item: self.item.as_mut().map(|x| x.finish()),
			item_offset: item_offset.map(|x| x.finish()),
		}
	}
}
