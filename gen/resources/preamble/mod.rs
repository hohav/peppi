//! Frame data representation.
//!
//! Peppi represents frame data using Arrow arrays (i.e. "struct-of-arrays").
//! This allows us to efficiently share frame data with other languages,
//! and enables simple serialization into a highly-compressible disk format.

#![allow(unused_variables)]

#[cfg(feature = "arrow")] mod peppi;
mod slippi;
pub mod transpose;

use std::{
	cmp::max,
	fmt,
	io::Result,
};

use byteorder::ReadBytesExt;

use crate::{
	game::Port,
	io::slippi::Version,
};

type BE = byteorder::BigEndian;

/// Frame indexes start at -123, and reach 0 at "Go!".
pub const FIRST_INDEX: i32 = -123;

/// Port number plus ICs-specific discriminant.
#[derive(Clone, Copy, Debug)]
pub struct PortOccupancy {
	pub port: Port,
	/// For ICs, distinguishes between Nana and Popo.
	pub follower: bool,
}

/// Rollback-aware processing typically ignores all but the first or last rollback for a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rollbacks {
	ExceptFirst,
	ExceptLast,
}

/// TODO: docs, or switch to BitVec?
#[derive(Debug)]
pub struct Validity {
	values: Vec<bool>,
	len: usize,
	capacity: usize,
}

impl Validity {
	pub fn with_capacity(capacity: usize) -> Self {
		Validity {
			values: Vec::new(),
			len: 0,
			capacity: capacity,
		}
	}

	pub fn len(&self) -> usize {
		if self.values.capacity() > 0 {
			self.values.len()
		} else {
			self.len
		}
	}

	pub fn is_valid(&self, idx: usize) -> bool {
		if self.values.capacity() > 0 {
			self.values[idx]
		} else {
			true
		}
	}

	pub fn push(&mut self, value: bool) {
		if self.values.capacity() > 0 {
			self.values.push(value);
		} else if !value {
			self.values.reserve(max(self.len + 1, self.capacity));
			self.values.append(&mut vec![true; self.len]);
			self.values.push(false);
		} else {
			self.len += 1;
		}
	}

	pub fn into_vec(self) -> Option<Vec<bool>> {
		(self.values.capacity() > 0).then(|| self.values)
	}

	pub fn null_count(&self) -> usize {
		if self.values.capacity() > 0 {
			let mut count = 0;
			for v in &self.values {
				if !v {
					count += 1;
				}
			}
			count
		} else {
			0
		}
	}
}

/// Frame data for a single character (ICs are two characters).
#[derive(Debug)]
pub struct Data {
	pub pre: Pre,
	pub post: Post,
	pub validity: Validity,
}

impl Data {
	pub fn with_capacity(capacity: usize, version: Version) -> Self {
		Self {
			pre: Pre::with_capacity(capacity, version),
			post: Post::with_capacity(capacity, version),
			validity: Validity::with_capacity(capacity),
		}
	}

	pub fn len(&self) -> usize {
		self.pre.len()
	}

	pub fn append_null(&mut self, version: Version) {
		self.validity.push(false);
		self.pre.append_default(version);
		self.post.append_default(version);
	}

	pub fn append_default(&mut self, version: Version) {
		self.validity.push(true);
		self.pre.append_default(version);
		self.post.append_default(version);
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> Option<transpose::Data> {
		self.validity.is_valid(i).then(|| transpose::Data {
			pre: self.pre.transpose_one(i, version),
			post: self.post.transpose_one(i, version),
		})
	}
}

/// Frame data for a single port.
#[derive(Debug)]
pub struct PortData {
	pub port: Port,
	pub leader: Data,
	/// The "backup" ICs character
	pub follower: Option<Data>,
	pub validity: Validity,
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
			validity: Validity::with_capacity(capacity),
		}
	}

	pub fn len(&self) -> usize {
		self.leader.len()
	}

	pub fn append_null(&mut self, version: Version) {
		self.validity.push(false);
		self.leader.append_default(version);
		self.follower.as_mut().map(|f| f.append_default(version));
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> Option<transpose::PortData> {
		self.validity.is_valid(i).then(|| transpose::PortData {
			port: self.port,
			leader: self.leader.transpose_one(i, version).unwrap(),
			follower: self
				.follower
				.as_ref()
				.and_then(|f| f.transpose_one(i, version)),
		})
	}
}

/// All frame data for a single game, in struct-of-arrays format.
pub struct Frame {
	/// Frame IDs start at `-123` and increment each frame. May repeat in case of rollbacks
	pub id: Vec<i32>,
	/// Port-specific data
	pub ports: Vec<PortData>,
	/// Start-of-frame data
	pub start: Option<Start>,
	/// End-of-frame data
	pub end: Option<End>,

	/// Item data
	pub item: Option<Item>,
	/// Logically, each frame has its own array of items. But we represent all item data in a flat array, with this field indicating the start of each sub-array
	pub item_offset: Option<Vec<i32>>,

	pub fod_platform: Option<FodPlatform>,
	pub fod_platform_offset: Option<Vec<i32>>,

	pub dreamland_whispy: Option<DreamlandWhispy>,
	pub dreamland_whispy_offset: Option<Vec<i32>>,

	pub stadium_transformation: Option<StadiumTransformation>,
	pub stadium_transformation_offset: Option<Vec<i32>>,
}

fn make_offsets(capacity: usize) -> Vec<i32> {
	let mut offsets = Vec::with_capacity(capacity+1);
	offsets.push(0);
	offsets
}

impl Frame {
	pub fn with_capacity(capacity: usize, version: Version, ports: &[PortOccupancy]) -> Self {
		Self {
			id: Vec::with_capacity(capacity),
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
			item: version.gte(3, 0).then(|| Item::with_capacity(0, version)),
			item_offset: version
				.gte(3, 0)
				.then(|| make_offsets(capacity)),
			fod_platform: version
				.gte(3, 18)
				.then(|| FodPlatform::with_capacity(0, version)),
			fod_platform_offset: version
				.gte(3, 18)
				.then(|| make_offsets(capacity)),
			dreamland_whispy: version
				.gte(3, 18)
				.then(|| DreamlandWhispy::with_capacity(0, version)),
			dreamland_whispy_offset: version
				.gte(3, 18)
				.then(|| make_offsets(capacity)),
			stadium_transformation: version
				.gte(3, 18)
				.then(|| StadiumTransformation::with_capacity(0, version)),
			stadium_transformation_offset: version
				.gte(3, 18)
				.then(|| make_offsets(capacity)),
		}
	}

	pub fn len(&self) -> usize {
		self.id.len()
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::Frame {
		transpose::Frame {
			id: self.id[i],
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
				let range = &self.item_offset.as_ref().unwrap()[i..i+2];
				(usize::try_from(range[0]).unwrap() .. usize::try_from(range[1]).unwrap())
					.map(|i| self.item.as_ref().unwrap().transpose_one(i, version))
					.collect()
			}),
			fod_platforms: version.gte(3, 18).then(|| {
				let range = &self.fod_platform_offset.as_ref().unwrap()[i..i+2];
				(usize::try_from(range[0]).unwrap() .. usize::try_from(range[1]).unwrap())
					.map(|i| self.fod_platform.as_ref().unwrap().transpose_one(i, version)
					)
					.collect()
			}),
			dreamland_whispys: version.gte(3, 18).then(|| {
				let range = &self.dreamland_whispy_offset.as_ref().unwrap()[i..i+2];
				(usize::try_from(range[0]).unwrap() .. usize::try_from(range[1]).unwrap())
					.map(|i| self.dreamland_whispy.as_ref().unwrap().transpose_one(i, version))
					.collect()
			}),
			stadium_transformations: version.gte(3, 18).then(|| {
				let range = &self.stadium_transformation_offset.as_ref().unwrap()[i..i+2];
				(usize::try_from(range[0]).unwrap() .. usize::try_from(range[1]).unwrap())
					.map(|i| self.stadium_transformation.as_ref().unwrap().transpose_one(i, version))
					.collect()
			}),
		}
	}

	/// Frames IDs may appear multiple times due to rollbacks. This fn lets you
	/// "dedupe" rollbacks, by returning `true` for all but one of each unique
	/// frame ID. The value returned at index `i` corresponds to `self.id[i]`.
	pub fn rollbacks(&self, keep: Rollbacks) -> Vec<bool> {
		use Rollbacks::*;
		match keep {
			ExceptFirst => self.rollbacks_(self.id.iter().enumerate()),
			ExceptLast => self.rollbacks_(self.id.iter().enumerate().rev()),
		}
	}

	fn rollbacks_<'a>(&self, ids: impl Iterator<Item = (usize, &'a i32)>) -> Vec<bool> {
		let mut result = vec![false; self.len()];
		let unique_id_count = self.id.iter().max().map_or(0, |idx| {
			1 + usize::try_from(idx - FIRST_INDEX).unwrap()
		});
		let mut seen = vec![false; unique_id_count];
		for (idx, id) in ids {
			let zero_based_id = usize::try_from(id - FIRST_INDEX).unwrap();
			if !seen[zero_based_id] {
				seen[zero_based_id] = true;
				result[idx] = false;
			} else {
				result[idx] = true;
			}
		}
		result
	}
}

impl fmt::Debug for Frame {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
		write!(f, "Frame {{ len: {} }}", self.id.len())
	}
}
