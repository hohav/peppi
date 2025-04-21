//! Single-frame representation using normal structs.
//!
//! Transposing frame data is fairly slow. Work with Arrow arrays when possible.

use crate::game::Port;

#[derive(PartialEq, Debug)]
pub struct Data {
	pub pre: Pre,
	pub post: Post,
}

#[derive(PartialEq, Debug)]
pub struct PortData {
	pub port: Port,
	pub leader: Data,
	pub follower: Option<Data>,
}

#[derive(PartialEq, Debug, Default)]
pub struct Frame {
	pub id: i32,
	pub ports: Vec<PortData>,
	pub start: Option<Start>,
	pub end: Option<End>,
	pub items: Option<Vec<Item>>,
	pub fod_platforms: Option<Vec<FodPlatform>>,
	pub dreamland_whispys: Option<Vec<DreamlandWhispy>>,
	pub stadium_transformations: Option<Vec<StadiumTransformation>>,
}
