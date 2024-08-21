use std::io::Read;

use log::debug;

use arrow::array::StructArray;
use arrow_ipc::reader::StreamReader;

use crate::{
	frame::{immutable::Frame, mutable::Frame as MutableFrame},
	game::{self, immutable::Game, port_occupancy},
	io::{expect_bytes, peppi, slippi, Result},
};

type JsMap = serde_json::Map<String, serde_json::Value>;

/// Options for parsing Peppi games.
#[derive(Clone, Debug, Default)]
pub struct Opts {
	/// Skip all frame data when parsing a replay for speed
	/// (when you only need start/end/metadata).
	pub skip_frames: bool,
}

fn read_arrow_frames<R: Read>(mut r: R, version: slippi::Version) -> Result<Frame> {
	// magic number `ARROW1\0\0`
	expect_bytes(&mut r, &[65, 82, 82, 79, 87, 49, 0, 0])?;
	let mut reader = StreamReader::try_new(r, None)?;
	let batch = reader.next().ok_or(err!("no batches"))??;
	let frames = StructArray::from(batch);
	assert_eq!(frames.column_names(), vec!["frame"]);
	let frames = frames
		.column(0)
		.as_any()
		.downcast_ref::<StructArray>()
		.ok_or(err!("expected a StructArray"))?;
	Ok(Frame::from_struct_array(frames.clone(), version))
}

fn read_peppi_start<R: Read>(mut r: R) -> Result<game::Start> {
	let mut buf = Vec::new();
	r.read_to_end(&mut buf)?;
	slippi::de::game_start(&mut &buf[..])
}

fn read_peppi_end<R: Read>(mut r: R) -> Result<game::End> {
	let mut buf = Vec::new();
	r.read_to_end(&mut buf)?;
	slippi::de::game_end(&mut &buf[..])
}

fn read_peppi_metadata<R: Read>(r: R) -> Result<JsMap> {
	let json_object: serde_json::Value = serde_json::from_reader(r)?;
	match json_object {
		serde_json::Value::Object(map) => Ok(map),
		obj => Err(err!("expected map, got: {:?}", obj)),
	}
}

fn read_peppi_gecko_codes<R: Read>(mut r: R) -> Result<game::GeckoCodes> {
	let mut actual_size = [0; 4];
	r.read_exact(&mut actual_size)?;
	let mut bytes = Vec::new();
	r.read_to_end(&mut bytes)?;
	Ok(game::GeckoCodes {
		actual_size: u32::from_le_bytes(actual_size),
		bytes: bytes,
	})
}

/// Reads a Peppi (`.slpp`) replay from `r`.
pub fn read<R: Read>(r: R, opts: Option<&Opts>) -> Result<Game> {
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
						MutableFrame::with_capacity(0, start.slippi.version, &port_occupancy(start))
							.finish()
					}
					_ => read_arrow_frames(file, version)?,
				});
				break;
			}
			_ => debug!("=> skipping"),
		};
	}

	let peppi = peppi.ok_or(err!("missing peppi"))?;
	Ok(Game {
		metadata: metadata,
		start: start.ok_or(err!("missing start"))?,
		end: end,
		gecko_codes: gecko_codes,
		frames: frames.ok_or(err!("missing frames"))?,
		hash: peppi.slp_hash,
		quirks: peppi.quirks,
	})
}
