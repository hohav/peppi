//! Immutable (fully-parsed) frame data, as Arrow arrays.
//!
//! This is what you get when you parse a game in one shot using [`crate::io::slippi::read`] or
//! [`crate::io::peppi::read`].
//!
//! These arrays can be shared, and cloning them is `O(1)`. See the
//! [arrow_array docs](https://docs.rs/arrow-array/latest/arrow_array/index.html) for more.

#![allow(unused_variables)]

mod peppi;
mod slippi;

use std::fmt;

use arrow::{
	array::{
		types::{Float32Type, Int32Type, Int8Type, UInt16Type, UInt32Type, UInt8Type},
		PrimitiveArray,
	},
	buffer::{NullBuffer, OffsetBuffer},
};

use crate::{
	frame::{self, transpose, Rollbacks},
	game::Port,
	io::slippi::Version,
};

/// Frame data for a single character (ICs are two characters).
#[derive(Debug)]
pub struct Data {
	pub pre: Pre,
	pub post: Post,
	pub validity: Option<NullBuffer>,
}

impl Data {
	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::Data {
		transpose::Data {
			pre: self.pre.transpose_one(i, version),
			post: self.post.transpose_one(i, version),
		}
	}
}

/// Frame data for a single port.
#[derive(Debug)]
pub struct PortData {
	pub port: Port,
	pub leader: Data,
	/// The "backup" ICs character
	pub follower: Option<Data>,
}

impl PortData {
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
	pub id: PrimitiveArray<Int32Type>,
	/// Port-specific data
	pub ports: Vec<PortData>,
	/// Start-of-frame data
	pub start: Option<Start>,
	/// End-of-frame data
	pub end: Option<End>,
	/// Logically, each frame has its own array of items. But we represent all item data in a flat array, with this field indicating the start of each sub-array
	pub item_offset: Option<OffsetBuffer<i32>>,
	/// Item data
	pub item: Option<Item>,
}

impl Frame {
	pub fn len(&self) -> usize {
		self.id.len()
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::Frame {
		transpose::Frame {
			id: self.id.values()[i],
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

	/// Frames IDs may appear multiple times due to rollbacks. This fn lets you
	/// "dedupe" rollbacks, by returning `true` for all but one of each unique
	/// frame ID. The value returned at index `i` corresponds to `self.id[i]`.
	pub fn rollbacks(&self, keep: Rollbacks) -> Vec<bool> {
		use Rollbacks::*;
		match keep {
			ExceptFirst => self.rollbacks_(self.id.values().iter().cloned().enumerate()),
			ExceptLast => self.rollbacks_(self.id.values().iter().cloned().enumerate().rev()),
		}
	}

	fn rollbacks_<'a>(&self, ids: impl Iterator<Item = (usize, i32)>) -> Vec<bool> {
		let mut result = vec![false; self.len()];
		let unique_id_count = arrow::compute::kernels::aggregate::max(&self.id).map_or(0, |idx| {
			1 + usize::try_from(idx - frame::FIRST_INDEX).unwrap()
		});
		let mut seen = vec![false; unique_id_count];
		for (idx, id) in ids {
			let zero_based_id = usize::try_from(id - frame::FIRST_INDEX).unwrap();
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
