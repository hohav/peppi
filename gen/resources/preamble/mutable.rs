//! Mutable (in-progress) frame data.
//!
//! You’ll only encounter mutable frame data if you’re parsing live games.

#![allow(unused_variables)]
#![allow(dead_code)]

use arrow2::{
	array::{MutableArray, MutablePrimitiveArray},
	bitmap::MutableBitmap,
	offset::Offsets,
};

use byteorder::ReadBytesExt;
use std::io::Result;

use crate::{
	io::slippi::{Version, STAGE_EVENTS_VERSION},
	frame::{transpose, PortOccupancy},
	game::Port,
};

type BE = byteorder::BigEndian;

/// Frame data for a single character (ICs are two characters).
pub struct Data {
	pub pre: Pre,
	pub post: Post,
	pub validity: Option<MutableBitmap>,
}

impl Data {
	pub fn with_capacity(capacity: usize, version: Version) -> Self {
		Self {
			pre: Pre::with_capacity(capacity, version),
			post: Post::with_capacity(capacity, version),
			validity: None,
		}
	}

	pub fn len(&self) -> usize {
		self.pre.len()
	}

	pub fn push_null(&mut self, version: Version) {
		let len = self.len();
		self.validity
			.get_or_insert_with(|| MutableBitmap::from_len_set(len))
			.push(false);
		self.pre.push_null(version);
		self.post.push_null(version);
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::Data {
		transpose::Data {
			pre: self.pre.transpose_one(i, version),
			post: self.post.transpose_one(i, version),
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
}

/// All frame data for a single game, in struct-of-arrays format.
pub struct Frame {
	/// Frame IDs start at `-123` and increment each frame. May repeat in case of rollbacks
	pub id: MutablePrimitiveArray<i32>,
	/// Port-specific data
	pub ports: Vec<PortData>,
	/// Start-of-frame data
	pub start: Option<Start>,
	/// End-of-frame data
	pub end: Option<End>,

	/// Item data
	pub item: Option<Item>,
	/// Logically, each frame has its own array of items. But we represent all item data in a flat array, with this field indicating the start of each sub-array
	pub item_offset: Option<Offsets<i32>>,

	pub fod_platform: Option<FodPlatform>,
	pub fod_platform_offset: Option<Offsets<i32>>,

	pub dreamland_whispy: Option<DreamlandWhispy>,
	pub dreamland_whispy_offset: Option<Offsets<i32>>,

	pub stadium_transformation: Option<StadiumTransformation>,
	pub stadium_transformation_offset: Option<Offsets<i32>>,
}

impl Frame {
	pub fn with_capacity(capacity: usize, version: Version, ports: &[PortOccupancy]) -> Self {
		Self {
			id: MutablePrimitiveArray::<i32>::with_capacity(capacity),
			ports: ports
				.iter()
				.map(|p| PortData::with_capacity(capacity, version, *p))
				.collect(),
			start: version.gte(2, 2).then(|| Start::with_capacity(capacity, version)),
			end: version.gte(3, 0).then(|| End::with_capacity(capacity, version)),
			item: version.gte(3, 0).then(|| Item::with_capacity(0, version)),
			item_offset: version.gte(3, 0).then(|| Offsets::<i32>::with_capacity(capacity)),
			fod_platform: (version >= STAGE_EVENTS_VERSION).then(|| FodPlatform::with_capacity(0, version)),
			fod_platform_offset: (version >= STAGE_EVENTS_VERSION).then(|| Offsets::<i32>::with_capacity(capacity)),
			dreamland_whispy: (version >= STAGE_EVENTS_VERSION).then(|| DreamlandWhispy::with_capacity(0, version)),
			dreamland_whispy_offset: (version >= STAGE_EVENTS_VERSION).then(|| Offsets::<i32>::with_capacity(capacity)),
			stadium_transformation: (version >= STAGE_EVENTS_VERSION).then(|| StadiumTransformation::with_capacity(0, version)),
			stadium_transformation_offset: (version >= STAGE_EVENTS_VERSION).then(|| Offsets::<i32>::with_capacity(capacity)),
		}
	}

	pub fn len(&self) -> usize {
		self.id.len()
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::Frame {
		transpose::Frame {
			id: self.id.values()[i],
			ports: self.ports.iter().map(|p| p.transpose_one(i, version)).collect(),
			start: version.gte(2, 2).then(|| self.start.as_ref().unwrap().transpose_one(i, version)),
			end: version.gte(3, 0).then(|| self.end.as_ref().unwrap().transpose_one(i, version)),
			items: version.gte(3, 0).then(|| {
				let (start, end) = self.item_offset.as_ref().unwrap().start_end(i);
				(start..end)
					.map(|i| self.item.as_ref().unwrap().transpose_one(i, version))
					.collect()
			}),
			fod_platforms: (version >= STAGE_EVENTS_VERSION).then(|| {
				let (start, end) = self.fod_platform_offset.as_ref().unwrap().start_end(i);
				(start..end)
					.map(|i| self.fod_platform.as_ref().unwrap().transpose_one(i, version))
					.collect()
			}),
			dreamland_whispys: (version >= STAGE_EVENTS_VERSION).then(|| {
				let (start, end) = self.dreamland_whispy_offset.as_ref().unwrap().start_end(i);
				(start..end)
					.map(|i| self.dreamland_whispy.as_ref().unwrap().transpose_one(i, version))
					.collect()
			}),
			stadium_transformations: (version >= STAGE_EVENTS_VERSION).then(|| {
				let (start, end) = self.stadium_transformation_offset.as_ref().unwrap().start_end(i);
				(start..end)
					.map(|i| self.stadium_transformation.as_ref().unwrap().transpose_one(i, version))
					.collect()
			}),
		}
	}
}
