use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use peppi::{
	self,
	game::port_occupancy,
	io::slippi::de::{Opts, read},
};

use std::{fs, io::Cursor, path::PathBuf, time::Duration};

pub fn into_game(c: &mut Criterion) {
	let dir = PathBuf::from("benches/data");
	for replay in fs::read_dir(dir).unwrap() {
		let path = replay.unwrap().path();
		let name = path.file_name().unwrap().to_str().unwrap().to_string();
		let contents = fs::read(path).unwrap();
		c.bench_with_input(
			BenchmarkId::new("into_game", &name),
			&contents,
			|b, contents| {
				b.iter_batched(
					|| contents.as_slice(),
					|buf| read(&mut Cursor::new(buf), None).unwrap(),
					BatchSize::LargeInput,
				)
			},
		);
	}
}

pub fn into_struct_array(c: &mut Criterion) {
	let dir = PathBuf::from("benches/data");
	for replay in fs::read_dir(dir).unwrap() {
		let path = replay.unwrap().path();
		let name = path.file_name().unwrap().to_str().unwrap().to_string();
		let contents = fs::read(path).unwrap();
		c.bench_with_input(
			BenchmarkId::new("into_struct_array", &name),
			&contents,
			|b, contents| {
				b.iter_batched(
					|| contents.as_slice(),
					|buf| {
						let game = read(&mut Cursor::new(buf), None).unwrap();
						game.frames.into_struct_array(game.start.slippi.version, &port_occupancy(&game.start))
					},
					BatchSize::LargeInput,
				)
			},
		);
	}
}

pub fn skip_frames(c: &mut Criterion) {
	let dir = PathBuf::from("benches/data");
	for replay in fs::read_dir(dir).unwrap() {
		let path = replay.unwrap().path();
		let name = path.file_name().unwrap().to_str().unwrap().to_string();
		let contents = fs::read(path).unwrap();
		c.bench_with_input(
			BenchmarkId::new("skip_frames", &name),
			&contents,
			|b, contents| {
				b.iter_batched(
					|| contents.as_slice(),
					|buf| {
						read(
							&mut Cursor::new(buf),
							Some(&Opts {
								skip_frames: true,
								..Default::default()
							}),
						)
					},
					BatchSize::SmallInput,
				)
			},
		);
	}
}

criterion_group! {
	name = bench_into_game;
	config = Criterion::default()
		.warm_up_time(Duration::from_secs(1));
	targets = into_game
}

criterion_group! {
	name = bench_into_struct_array;
	config = Criterion::default()
		.warm_up_time(Duration::from_secs(1));
	targets = into_struct_array
}

criterion_group! {
	name = bench_skip_frames;
	config = Criterion::default()
		.warm_up_time(Duration::from_secs(1));
	targets = skip_frames
}

criterion_main!(bench_into_game, bench_into_struct_array, bench_skip_frames);
