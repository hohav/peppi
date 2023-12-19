use std::io::{Error, Read};

use log::debug;

use arrow2::{
	array::StructArray,
	io::ipc::read::{read_stream_metadata, StreamReader, StreamState},
};

use crate::{
	io::{expect_bytes, peppi, slippi},
	model::{
		frame::{immutable::Frame, mutable::Frame as MutableFrame},
		game::{self, immutable::Game},
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

fn read_arrow_frames<R: Read>(mut r: R, version: slippi::Version) -> Result<Frame, Error> {
	// magic number `ARROW1\0\0`
	expect_bytes(&mut r, &[65, 82, 82, 79, 87, 49, 0, 0])?;
	let metadata = read_stream_metadata(&mut r).map_err(Error::other)?;
	let reader = StreamReader::new(r, metadata, None);
	let mut frame: Option<Frame> = None;
	for result in reader {
		match result.map_err(Error::other)? {
			StreamState::Some(chunk) => match frame {
				None => {
					let f = chunk.arrays()[0]
						.as_any()
						.downcast_ref::<StructArray>()
						.expect("expected a `StructArray`");
					frame = Some(Frame::from_struct_array(f.clone(), version))
				}
				Some(_) => return Err(err!("multiple batches")),
			},
			StreamState::Waiting => std::thread::sleep(std::time::Duration::from_millis(1000)),
		}
	}
	match frame {
		Some(f) => Ok(f),
		_ => Err(err!("no batches")),
	}
}

fn read_peppi_start<R: Read>(mut r: R) -> Result<game::Start, Error> {
	let mut buf = Vec::new();
	r.read_to_end(&mut buf)?;
	Ok(game::Start::from_bytes(buf.as_slice())?)
}

fn read_peppi_end<R: Read>(mut r: R) -> Result<game::End, Error> {
	let mut buf = Vec::new();
	r.read_to_end(&mut buf)?;
	Ok(game::End::from_bytes(buf.as_slice())?)
}

fn read_peppi_metadata<R: Read>(r: R) -> Result<JsMap, Error> {
	let json_object: serde_json::Value = serde_json::from_reader(r)?;
	match json_object {
		serde_json::Value::Object(map) => Ok(map),
		obj => Err(err!("expected map, got: {:?}", obj)),
	}
}

fn read_peppi_gecko_codes<R: Read>(mut r: R) -> Result<game::GeckoCodes, Error> {
	let mut actual_size = [0; 4];
	r.read_exact(&mut actual_size)?;
	let mut bytes = Vec::new();
	r.read_to_end(&mut bytes)?;
	Ok(game::GeckoCodes {
		actual_size: u32::from_le_bytes(actual_size),
		bytes: bytes,
	})
}

pub(crate) fn read<R: Read>(r: R, opts: Option<&Opts>) -> Result<(Game, peppi::Peppi), Error> {
	let mut start: Option<game::Start> = None;
	let mut end: Option<game::End> = None;
	let mut metadata: Option<JsMap> = None;
	let mut gecko_codes: Option<game::GeckoCodes> = None;
	let mut frames: Option<Frame> = None;
	let mut peppi: Option<peppi::Peppi> = None;
	for entry in tar::Archive::new(r).entries()? {
		let file = entry?;
		let path = file.path()?;
		debug!("processing file: {}", path.display());
		match path.file_name().and_then(|n| n.to_str()) {
			Some("peppi.json") => {
				let p: peppi::Peppi = serde_json::from_reader::<_, peppi::Peppi>(file)?;
				// TODO: support reading v1
				super::assert_current_version(p.version)?;
				peppi = Some(p);
			}
			Some("start.raw") => start = Some(read_peppi_start(file)?),
			Some("end.raw") => end = Some(read_peppi_end(file)?),
			Some("metadata.json") => metadata = Some(read_peppi_metadata(file)?),
			Some("gecko_codes.raw") => gecko_codes = Some(read_peppi_gecko_codes(file)?),
			Some("frames.arrow") => {
				let version = start
					.as_ref()
					.map(|s| s.slippi.version)
					.ok_or(err!("no start"))?;
				frames = Some(match opts.map_or(false, |o| o.skip_frames) {
					true => {
						let start = start.as_ref().ok_or(err!("missing start"))?;
						MutableFrame::with_capacity(
							0,
							start.slippi.version,
							&super::port_occupancy(start),
						)
						.into()
					}
					_ => read_arrow_frames(file, version)?,
				});
			}
			_ => debug!("=> skipping"),
		};
	}

	Ok((
		Game {
			metadata: metadata,
			start: start.ok_or(err!("missing start"))?,
			end: end,
			gecko_codes: gecko_codes,
			frames: frames.ok_or(err!("missing frames"))?,
		},
		peppi.ok_or(err!("missing peppi"))?,
	))
}
