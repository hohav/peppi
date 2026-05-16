use std::{error::Error, io::Write, path::Path, sync::Arc};

use arrow::{
	array::RecordBatch,
	datatypes::{DataType, Field, Schema},
};
use arrow_ipc::{
	r#gen::{Message::CompressionType, Schema::MetadataVersion},
	writer::{FileWriter, IpcWriteOptions},
};

use crate::{
	game::{Game, port_occupancy},
	io::{
		peppi, slippi,
		ubjson::{self, JMap},
	},
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
	if let Some(metadata) = game.metadata {
		let mut buf = Vec::<u8>::new();
		ubjson::write_map(&mut buf, &metadata)?;
		tar_append(&mut tar, &buf, "metadata.raw")?;
		tar_append(
			&mut tar,
			&serde_json::to_vec(&JMap::from(metadata))?,
			"metadata.json",
		)?;
	}
	tar_append(&mut tar, &game.start.bytes.0, "start.raw")?;
	tar_append(&mut tar, &serde_json::to_vec(&game.start)?, "start.json")?;
	if let Some(end) = &game.end {
		tar_append(&mut tar, &end.bytes.0, "end.raw")?;
		tar_append(&mut tar, &serde_json::to_vec(end)?, "end.json")?;
	}

	if let Some(gecko_codes) = &game.gecko_codes {
		let mut buf = gecko_codes.actual_size.to_le_bytes().to_vec();
		buf.write_all(&gecko_codes.bytes)?;
		tar_append(&mut tar, &buf, "gecko_codes.raw")?;
	}

	if game.frames.id.len() > 0 {
		let ports = port_occupancy(&game.start);
		let frames = game
			.frames
			.into_struct_array(game.start.slippi.version, &ports);
		let schema = Schema::new(vec![Field::new(
			"frame",
			DataType::Struct(frames.fields().clone()),
			false,
		)]);

		let mut buf = Vec::new();
		let mut writer = FileWriter::try_new_with_options(
			&mut buf,
			&schema,
			IpcWriteOptions::try_new(8, false, MetadataVersion::V5)?
				.try_with_compression(opts.and_then(|o| o.compression))?,
		)?;
		writer.write(&RecordBatch::try_new(
			Arc::new(schema),
			vec![Arc::new(frames)],
		)?)?;
		writer.finish()?;
		tar_append(&mut tar, &buf, "frames.arrow")?;
	}

	tar.into_inner()?.flush()?;
	Ok(())
}
