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
	pub start: Start,
	pub end: End,
	pub port: Vec<PortData>,
	//FIXME
	//pub item_offset: OffsetsBuffer<i32>,
	//pub item: Item,
}
