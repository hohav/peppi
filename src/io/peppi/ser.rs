use std::{error::Error, io::Write, path::Path};

use arrow2::{
	array::Array,
	chunk::Chunk,
	datatypes::{Field, Schema},
	io::ipc::write::{FileWriter, WriteOptions},
};

use serde::{Deserialize, Serialize};

use crate::{
	io::slippi::de::ICE_CLIMBERS,
	model::{
		frame::PortOccupancy,
		game::{self, immutable::Game},
	},
};

pub const PEPPI_FORMAT_VERSION: &str = "2.0";

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Peppi {
	pub version: String,
	pub slp_hash: String,
}

fn port_occupancy(game: &Game) -> Vec<PortOccupancy> {
	game.start
		.players
		.iter()
		.map(|p| PortOccupancy {
			port: p.port,
			follower: p.character == ICE_CLIMBERS,
		})
		.collect()
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

pub fn write<W: Write>(game: Game, w: W, slp_hash: String) -> Result<(), Box<dyn Error>> {
	if game.start.slippi.version > game::MAX_SUPPORTED_VERSION {
		return Err(format!(
			"Unsupported Slippi version ({} > {})",
			game.start.slippi.version,
			game::MAX_SUPPORTED_VERSION
		)
		.into());
	}

	let peppi = Peppi {
		version: PEPPI_FORMAT_VERSION.to_string(),
		slp_hash: slp_hash,
	};

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
		let ports = port_occupancy(&game);
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
				compression: None,
				//compression: Some(Compression::LZ4),
				//compression: Some(Compression::ZSTD),
			},
		)?;
		writer.write(&chunk, None)?;
		writer.finish()?;
		tar_append(&mut tar, &buf, "frames.arrow")?;
	}

	tar.into_inner()?.flush()?;
	Ok(())
}
