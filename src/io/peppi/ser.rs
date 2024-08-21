use std::{error::Error, io::Write, path::Path, sync::Arc};

use arrow::{
	array::{Array, StructArray},
	datatypes::{Field, Fields},
	record_batch::RecordBatch,
};
use arrow_ipc::{
	gen::Message::CompressionType,
	writer::{FileWriter, IpcWriteOptions},
};

use crate::{
	game::{immutable::Game, port_occupancy},
	io::{peppi, slippi},
};

/// Options for writing Peppi files.
#[derive(Clone, Debug, Default)]
pub struct Opts {
	/// Internal compression to use, if any.
	///
	/// Use this to maximize read speed while saving some disk space (e.g. for machine learning).
	/// If you just want maximum compression, compress the entire `.slpp` file instead.
	pub compression: Option<CompressionType>,
}

fn tar_append<W: Write, P: AsRef<Path>>(
	builder: &mut tar::Builder<W>,
	buf: &[u8],
	path: P,
) -> Result<(), Box<dyn Error>> {
	let mut header = tar::Header::new_gnu();
	header.set_size(buf.len().try_into()?);
	header.set_path(path)?;
	header.set_mode(0o644);
	header.set_cksum();
	builder.append(&header, buf)?;
	Ok(())
}

fn write_frames(
	frames: StructArray,
	buf: &mut Vec<u8>,
	opts: Option<&Opts>,
) -> Result<(), Box<dyn Error>> {
	let frames = StructArray::new(
		Fields::from(vec![Field::new("frame", frames.data_type().clone(), false)]),
		vec![Arc::new(frames) as Arc<dyn Array>],
		None,
	);
	let batch = RecordBatch::from(frames);
	let mut writer = FileWriter::try_new_with_options(
		buf,
		batch.schema_ref(),
		IpcWriteOptions::default().try_with_compression(opts.and_then(|o| o.compression))?,
	)?;
	writer.write(&batch)?;
	writer.finish()?;
	Ok(())
}

/// Writes a replay to `w` in Peppi (`.slpp`) format.
///
/// Returns an error if the game's version is higher than `MAX_SUPPORTED_VERSION`.
pub fn write<W: Write>(w: W, game: Game, opts: Option<&Opts>) -> Result<(), Box<dyn Error>> {
	slippi::assert_max_version(game.start.slippi.version)?;

	let mut tar = tar::Builder::new(w);
	tar_append(
		&mut tar,
		&serde_json::to_vec(&peppi::Peppi {
			version: peppi::CURRENT_VERSION,
			slp_hash: game.hash,
			quirks: game.quirks,
		})?,
		"peppi.json",
	)?;
	tar_append(
		&mut tar,
		&serde_json::to_vec(&game.metadata)?,
		"metadata.json",
	)?;
	tar_append(&mut tar, &serde_json::to_vec(&game.start)?, "start.json")?;
	tar_append(&mut tar, &game.start.bytes.0, "start.raw")?;
	if let Some(end) = &game.end {
		tar_append(&mut tar, &serde_json::to_vec(end)?, "end.json")?;
		tar_append(&mut tar, &end.bytes.0, "end.raw")?;
	}

	if let Some(gecko_codes) = &game.gecko_codes {
		let mut buf = gecko_codes.actual_size.to_le_bytes().to_vec();
		buf.write_all(&gecko_codes.bytes)?;
		tar_append(&mut tar, &buf, "gecko_codes.raw")?;
	}

	if !game.frames.id.is_empty() {
		let ports = port_occupancy(&game.start);
		let frames = game
			.frames
			.into_struct_array(game.start.slippi.version, &ports);

		let mut buf = Vec::new();
		write_frames(frames, &mut buf, opts)?;
		tar_append(&mut tar, &buf, "frames.arrow")?;
	}

	tar.into_inner()?.flush()?;
	Ok(())
}
