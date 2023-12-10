use std::{error::Error, io::Read};

use log::debug;

use arrow2::{
	array::StructArray,
	io::ipc::read::{read_stream_metadata, StreamReader, StreamState},
};

use crate::{
	io::{
		expect_bytes,
		peppi::ser::Peppi,
	},
	model::{
		frame::immutable::Frame,
		game::{self, immutable::Game},
		slippi,
	},
};

type JsMap = serde_json::Map<String, serde_json::Value>;

/// Options for parsing Peppi games.
#[derive(Clone, Debug, Default)]
pub struct Opts {
	/// Skip all frame data when parsing a replay for speed
	/// (when you only need start/end/metadata).
	pub skip_frames: bool,
}

fn read_arrow_frames<R: Read>(mut r: R, version: slippi::Version) -> Result<Frame, Box<dyn Error>> {
	// magic number `ARROW1\0\0`
	expect_bytes(&mut r, &[65, 82, 82, 79, 87, 49, 0, 0])?;
	let metadata = read_stream_metadata(&mut r)?;
	let reader = StreamReader::new(r, metadata, None);
	let mut frame: Option<Frame> = None;
	for result in reader {
		match result? {
			StreamState::Some(chunk) => match frame {
				None => {
					let f = chunk.arrays()[0]
						.as_any()
						.downcast_ref::<StructArray>()
						.expect("expected a `StructArray`");
					frame = Some(Frame::from_struct_array(f.clone(), version))
				}
				Some(_) => return Err("multiple batches".into()),
			},
			StreamState::Waiting => std::thread::sleep(std::time::Duration::from_millis(1000)),
		}
	}
	match frame {
		Some(f) => Ok(f),
		_ => Err("no batches".into()),
	}
}

fn read_peppi_start<R: Read>(mut r: R) -> Result<game::Start, Box<dyn Error>> {
	let mut buf = Vec::new();
	r.read_to_end(&mut buf)?;
	Ok(game::Start::from_bytes(buf.as_slice())?)
}

fn read_peppi_end<R: Read>(mut r: R) -> Result<game::End, Box<dyn Error>> {
	let mut buf = Vec::new();
	r.read_to_end(&mut buf)?;
	Ok(game::End::from_bytes(buf.as_slice())?)
}

fn read_peppi_metadata<R: Read>(r: R) -> Result<JsMap, Box<dyn Error>> {
	let json_object: serde_json::Value = serde_json::from_reader(r)?;
	match json_object {
		serde_json::Value::Object(map) => Ok(map),
		obj => Err(format!("expected map, got: {:?}", obj).into()),
	}
}

fn read_peppi_gecko_codes<R: Read>(mut r: R) -> Result<game::GeckoCodes, Box<dyn Error>> {
	let mut actual_size = [0; 4];
	r.read_exact(&mut actual_size)?;
	let mut bytes = Vec::new();
	r.read_to_end(&mut bytes)?;
	Ok(game::GeckoCodes {
		actual_size: u32::from_le_bytes(actual_size),
		bytes: bytes,
	})
}

pub fn read<R: Read>(r: R, opts: &Opts) -> Result<(Game, String), Box<dyn Error>> {
	let mut start: Option<game::Start> = None;
	let mut end: Option<game::End> = None;
	let mut metadata: Option<JsMap> = None;
	let mut gecko_codes: Option<game::GeckoCodes> = None;
	let mut frames: Option<Frame> = None;
	let mut peppi: Option<Peppi> = None;
	for entry in tar::Archive::new(r).entries()? {
		let file = entry?;
		let path = file.path()?;
		debug!("processing file: {}", path.display());
		match path.file_name().and_then(|n| n.to_str()) {
			Some("peppi.json") => peppi = serde_json::from_reader(file)?,
			Some("start.raw") => start = Some(read_peppi_start(file)?),
			Some("end.raw") => end = Some(read_peppi_end(file)?),
			Some("metadata.json") => metadata = Some(read_peppi_metadata(file)?),
			Some("gecko_codes.raw") => gecko_codes = Some(read_peppi_gecko_codes(file)?),
			Some("frames.arrow") => {
				let version = start.as_ref().map(|s| s.slippi.version).ok_or("no start")?;
				frames = Some(match opts.skip_frames {
					true => unimplemented!(),
					_ => read_arrow_frames(file, version)?,
				});
				if opts.skip_frames {
					break;
				}
			}
			_ => debug!("=> skipping"),
		};
	}

	let game = Game {
		metadata: metadata,
		start: start.ok_or("missing start")?,
		end: end,
		gecko_codes: gecko_codes,
		frames: frames.ok_or("missing frames")?,
	};
	Ok((game, peppi.ok_or("missing peppi")?.slp_hash))
}
