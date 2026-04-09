//! Mutable (in-progress) frame data.
//!
//! You’ll only encounter mutable frame data if you’re parsing live games.

#![allow(unused_variables)]
#![allow(dead_code)]

use arrow::{
	array::{ArrayBuilder, NullBufferBuilder, OffsetBufferBuilder, PrimitiveBuilder},
	datatypes::{Float32Type, Int8Type, Int32Type, UInt8Type, UInt16Type, UInt32Type},
};

use byteorder::ReadBytesExt;
use std::io::Result;

use crate::{
	io::slippi::Version,
	frame::{immutable, transpose, PortOccupancy},
	game::Port,
};

type BE = byteorder::BigEndian;

fn start_end(buf: &Option<OffsetBufferBuilder<i32>>, i: usize) -> (usize, usize) {
	let b = buf.as_ref().unwrap();
	(b[i].try_into().unwrap(), b[i+1].try_into().unwrap())
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

	pub fn append_null(&mut self, version: Version) {
		self.validity.append_n_non_nulls(self.len());
		self.validity.append_null();
		self.pre.append_null(version);
		self.post.append_null(version);
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
		let Self {
			port,
			leader,
			follower,
		} = self;
		immutable::PortData {
			port: *port,
			leader: leader.finish(),
			follower: follower.as_mut().map(|f| f.finish()),
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

	/// Item data
	pub item: Option<Item>,
	/// Logically, each frame has its own array of items. But we represent all item data in a flat array, with this field indicating the start of each sub-array
	pub item_offset: Option<OffsetBufferBuilder<i32>>,

	pub fod_platform: Option<FodPlatform>,
	pub fod_platform_offset: Option<OffsetBufferBuilder<i32>>,

	pub dreamland_whispy: Option<DreamlandWhispy>,
	pub dreamland_whispy_offset: Option<OffsetBufferBuilder<i32>>,

	pub stadium_transformation: Option<StadiumTransformation>,
	pub stadium_transformation_offset: Option<OffsetBufferBuilder<i32>>,
}

impl Frame {
	pub fn with_capacity(capacity: usize, version: Version, ports: &[PortOccupancy]) -> Self {
		Self {
			id: PrimitiveBuilder::<Int32Type>::with_capacity(capacity),
			ports: ports
				.iter()
				.map(|p| PortData::with_capacity(capacity, version, *p))
				.collect(),
			start: version.gte(2, 2).then(|| Start::with_capacity(capacity, version)),
			end: version.gte(3, 0).then(|| End::with_capacity(capacity, version)),
			item: version.gte(3, 0).then(|| Item::with_capacity(0, version)),
			item_offset: version.gte(3, 0).then(|| OffsetBufferBuilder::<i32>::new(capacity)),
			fod_platform: version.gte(3, 18).then(|| FodPlatform::with_capacity(0, version)),
			fod_platform_offset: version.gte(3, 18).then(|| OffsetBufferBuilder::<i32>::new(capacity)),
			dreamland_whispy: version.gte(3, 18).then(|| DreamlandWhispy::with_capacity(0, version)),
			dreamland_whispy_offset: version.gte(3, 18).then(|| OffsetBufferBuilder::<i32>::new(capacity)),
			stadium_transformation: version.gte(3, 18).then(|| StadiumTransformation::with_capacity(0, version)),
			stadium_transformation_offset: version.gte(3, 18).then(|| OffsetBufferBuilder::<i32>::new(capacity)),
		}
	}

	pub fn len(&self) -> usize {
		self.id.len()
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::Frame {
		transpose::Frame {
			id: self.id.values_slice()[i],
			ports: self.ports.iter().map(|p| p.transpose_one(i, version)).collect(),
			start: version.gte(2, 2).then(|| self.start.as_ref().unwrap().transpose_one(i, version)),
			end: version.gte(3, 0).then(|| self.end.as_ref().unwrap().transpose_one(i, version)),
			items: version.gte(3, 0).then(|| {
				let (start, end) = start_end(&self.item_offset, i);
				(start..end)
					.map(|i| self.item.as_ref().unwrap().transpose_one(i, version))
					.collect()
			}),
			fod_platforms: version.gte(3, 18).then(|| {
				let (start, end) = start_end(&self.fod_platform_offset, i);
				(start..end)
					.map(|i| self.fod_platform.as_ref().unwrap().transpose_one(i, version))
					.collect()
			}),
			dreamland_whispys: version.gte(3, 18).then(|| {
				let (start, end) = start_end(&self.dreamland_whispy_offset, i);
				(start..end)
					.map(|i| self.dreamland_whispy.as_ref().unwrap().transpose_one(i, version))
					.collect()
			}),
			stadium_transformations: version.gte(3, 18).then(|| {
				let (start, end) = start_end(&self.stadium_transformation_offset, i);
				(start..end)
					.map(|i| self.stadium_transformation.as_ref().unwrap().transpose_one(i, version))
					.collect()
			}),
		}
	}

	pub fn finish(&mut self) -> immutable::Frame {
		let Self {
			id,
			ports,
			start,
			end,
			item,
			item_offset,
			fod_platform,
			fod_platform_offset,
			dreamland_whispy,
			dreamland_whispy_offset,
			stadium_transformation,
			stadium_transformation_offset,
		} = self;
		immutable::Frame {
			id: id.finish(),
			ports: ports.into_iter().map(|p| p.finish()).collect(),
			start: start.as_mut().map(|x| x.finish()),
			end: end.as_mut().map(|x| x.finish()),
			item: item.as_mut().map(|x| x.finish()),
			item_offset: item_offset.take().map(|x| x.finish()),
			fod_platform: fod_platform.as_mut().map(|x| x.finish()),
			fod_platform_offset: fod_platform_offset.take().map(|x| x.finish()),
			dreamland_whispy: dreamland_whispy.as_mut().map(|x| x.finish()),
			dreamland_whispy_offset: dreamland_whispy_offset.take().map(|x| x.finish()),
			stadium_transformation: stadium_transformation.as_mut().map(|x| x.finish()),
			stadium_transformation_offset: stadium_transformation_offset.take().map(|x| x.finish()),
		}
	}
}
