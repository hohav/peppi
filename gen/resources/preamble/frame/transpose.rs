use crate::model::game::Port;

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

#[derive(PartialEq, Debug)]
pub struct Frame {
	pub id: i32,
	pub ports: Vec<PortData>,
	pub start: Option<Start>,
	pub end: Option<End>,
	pub items: Option<Vec<Item>>,
}
