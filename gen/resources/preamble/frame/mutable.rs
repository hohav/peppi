#![allow(unused_parens)]
#![allow(unused_variables)]
#![allow(dead_code)]

use arrow2::{
	array::MutablePrimitiveArray,
	offset::Offsets,
};

use byteorder::ReadBytesExt;
use std::io::Result;

use crate::{
	model::{
		frame::{transpose, PortOccupancy},
		game::Port,
		slippi::Version,
	},
};

type BE = byteorder::BigEndian;

pub struct Data {
	pub pre: Pre,
	pub post: Post,
}

impl Data {
	pub fn with_capacity(capacity: usize, version: Version) -> Self {
		Self {
			pre: Pre::with_capacity(capacity, version),
			post: Post::with_capacity(capacity, version),
		}
	}

	pub fn push_none(&mut self, version: Version) {
		self.pre.push_none(version);
		self.post.push_none(version);
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::Data {
		transpose::Data {
			pre: self.pre.transpose_one(i, version),
			post: self.post.transpose_one(i, version),
		}
	}
}

pub struct PortData {
	pub port: Port,
	pub leader: Data,
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

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::PortData {
		transpose::PortData {
			port: self.port,
			leader: self.leader.transpose_one(i, version),
			follower: self.follower.as_ref().map(|f| f.transpose_one(i, version)),
		}
	}
}

pub struct Frame {
	pub id: MutablePrimitiveArray<i32>,
	pub start: Start,
	pub end: End,
	pub port: Vec<PortData>,
	pub item_offset: Offsets<i32>,
	pub item: Item,
}

impl Frame {
	pub fn with_capacity(capacity: usize, version: Version, ports: &[PortOccupancy]) -> Self {
		Self {
			id: MutablePrimitiveArray::<i32>::with_capacity(capacity),
			start: Start::with_capacity(capacity, version),
			end: End::with_capacity(capacity, version),
			port: ports
				.iter()
				.map(|p| PortData::with_capacity(capacity, version, *p))
				.collect(),
			item_offset: Offsets::<i32>::with_capacity(capacity),
			item: Item::with_capacity(0, version),
		}
	}

	pub fn transpose_one(&self, i: usize, version: Version) -> transpose::Frame {
		transpose::Frame {
			id: self.id.values()[i],
			start: self.start.transpose_one(i, version),
			end: self.end.transpose_one(i, version),
			port: self.port.iter().map(|p| p.transpose_one(i, version)).collect(),
		}
	}
}
