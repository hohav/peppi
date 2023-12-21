use std::{error::Error, io::Write, path::Path};

use arrow2::{
	array::Array,
	chunk::Chunk,
	datatypes::{Field, Schema},
	io::ipc::write::{Compression as ArrowCompression, FileWriter, WriteOptions},
};

use crate::{
	game::immutable::Game,
	io::{peppi, slippi},
};

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

pub fn write<W: Write>(w: W, game: Game, peppi: peppi::Peppi) -> Result<(), Box<dyn Error>> {
	slippi::assert_max_version(game.start.slippi.version)?;
	peppi::assert_current_version(peppi.version)?;

	let mut tar = tar::Builder::new(w);
	tar_append(&mut tar, &serde_json::to_vec(&peppi)?, "peppi.json")?;
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

	if game.frames.id.len() > 0 {
		let ports = super::port_occupancy(&game.start);
		let batch = game
			.frames
			.into_struct_array(game.start.slippi.version, &ports);
		let schema = Schema::from(vec![Field {
			name: "frame".to_string(),
			data_type: batch.data_type().clone(),
			is_nullable: false,
			metadata: Default::default(),
		}]);

		let chunk = Chunk::new(vec![Box::new(batch) as Box<dyn Array>]);
		let mut buf = Vec::new();
		let mut writer = FileWriter::try_new(
			&mut buf,
			schema,
			None,
			WriteOptions {
				compression: peppi.compression.map(|c| {
					use peppi::Compression::*;
					match c {
						LZ4 => ArrowCompression::LZ4,
						ZSTD => ArrowCompression::ZSTD,
					}
				}),
			},
		)?;
		writer.write(&chunk, None)?;
		writer.finish()?;
		tar_append(&mut tar, &buf, "frames.arrow")?;
	}

	tar.into_inner()?.flush()?;
	Ok(())
}
