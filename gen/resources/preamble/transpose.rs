//! Single-frame representation using normal structs.

use std::{
	borrow::Cow,
	io::Result,
};

use byteorder::ReadBytesExt;

use crate::{
	frame::{EventCounts, Frames, PortOccupancy, Reader},
	game::Port,
	io::slippi::Version,
};

type BE = byteorder::BigEndian;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Data {
	pub pre: Pre,
	pub post: Post,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PortData {
	pub port: Port, // TODO: optimize away?
	pub leader: Data,
	pub follower: Option<Box<Data>>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Frame {
	pub id: i32,
	pub ports: Vec<Option<PortData>>,
	pub start: Option<Start>,
	pub end: Option<End>,
	pub items: Option<Vec<Item>>,
	pub fod_platforms: Option<Vec<FodPlatform>>,
	pub dreamland_whispys: Option<Vec<DreamlandWhispy>>,
	pub stadium_transformations: Option<Vec<StadiumTransformation>>,
}

impl Frames for Vec<Frame> {
	fn with_capacity(capacity: usize, version: Version, ports: &[PortOccupancy]) -> Self {
		Self::with_capacity(capacity)
	}

	fn len(&self) -> usize {
		self.len()
	}

	fn last_id(&self) -> Option<i32> {
		self.last().map(|f| f.id)
	}

	fn frame(&self, i: usize, version: Version) -> Cow<'_, Frame> {
		Cow::Borrowed(&self[i])
	}

	fn event_counts(&self) -> EventCounts {
		unimplemented!();
	}
}

impl Reader for Vec<Frame> {
	fn open(&mut self, version: Version, id: i32, port_count: usize) {
		self.push(Frame {
			id: id,
			ports: Vec::with_capacity(port_count),
			start: None,
			end: None,
			items: version.gte(3, 0).then(|| Vec::new()),
			fod_platforms: version.gte(3, 18).then(|| Vec::new()),
			dreamland_whispys: version.gte(3, 18).then(|| Vec::new()),
			stadium_transformations: version.gte(3, 18).then(|| Vec::new()),
		})
	}

	fn close(&mut self, version: Version, port_count: usize) {
		let last_frame = self.last_mut().unwrap();
		for _ in last_frame.ports.len() .. port_count {
			last_frame.ports.push(None);
		}
	}

	fn read_start(&mut self, r: &mut &[u8], version: Version) -> Result<()> {
		self.last_mut().unwrap().start = Some(Start::read(r, version)?);
		Ok(())
	}

	fn read_pre(&mut self, r: &mut &[u8], version: Version, id: i32, port_index: u8, port: Port, follower: bool) -> Result<()> {
		//FIXME: convert panics to Err
		let last_frame = self.last_mut().unwrap();
		assert_eq!(id, last_frame.id);
		let ports_len = last_frame.ports.len();
		if ports_len <= port_index as usize {
			assert!(!follower, "pre-frame update: follower before leader (frame: {}, port: {})", id, port);
			for i in ports_len .. port_index as usize {
				last_frame.ports.push(None);
			}
			last_frame.ports.push(Some(PortData {
				port: port,
				leader: Data {
					pre: Pre::read(r, version)?,
					post: Post::default(),
				},
				follower: None,
			}));
		} else if ports_len == port_index as usize + 1 {
			assert!(follower, "pre-frame update: duplicate leader (frame: {}, port: {})", id, port);
			let port_data = last_frame.ports.last_mut().unwrap().as_mut().expect("follower without leader");
			assert!(port_data.follower.is_none(), "pre-frame update: duplicate follower (frame: {}, port: {})", id, port);
			port_data.follower = Some(Box::new(Data {
				pre: Pre::read(r, version)?,
				post: Post::default(),
			}));
		} else {
			panic!("unexpected port (frame: {}, port: {})", id, port);
		}
		Ok(())
	}

	fn read_post(&mut self, r: &mut &[u8], version: Version, id: i32, port_index: u8, port: Port, follower: bool) -> Result<()> {
		let last_frame = self.last_mut().unwrap();
		assert!((port_index as usize) < last_frame.ports.len());
		let port_data = last_frame.ports[port_index as usize].as_mut().expect("port data exists");
		assert_eq!(port, port_data.port);
		let character = match follower {
			false => port_data.leader.post = Post::read(r, version)?,
			_ => port_data.follower.as_mut().expect("follower exists").post = Post::read(r, version)?,
		};
		Ok(())
	}

	fn read_item(&mut self, r: &mut &[u8], version: Version) -> Result<()> {
		self.last_mut().unwrap().items.as_mut().unwrap().push(Item::read(r, version)?);
		Ok(())
	}

	fn read_fod_platform(&mut self, r: &mut &[u8], version: Version) -> Result<()> {
		let last_frame = self.last_mut().unwrap();
		last_frame.fod_platforms.as_mut().unwrap().push(FodPlatform::read(r, version)?);
		Ok(())
	}

	fn read_dreamland_whispy(&mut self, r: &mut &[u8], version: Version) -> Result<()> {
		let last_frame = self.last_mut().unwrap();
		last_frame.dreamland_whispys.as_mut().unwrap().push(DreamlandWhispy::read(r, version)?);
		Ok(())
	}

	fn read_stadium_transformation(&mut self, r: &mut &[u8], version: Version) -> Result<()> {
		let last_frame = self.last_mut().unwrap();
		last_frame.stadium_transformations.as_mut().unwrap().push(StadiumTransformation::read(r, version)?);
		Ok(())
	}

	fn read_end(&mut self, r: &mut &[u8], version: Version) -> Result<()> {
		self.last_mut().unwrap().end = Some(End::read(r, version)?);
		Ok(())
	}
}
