use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use peppi::{
	self,
	io::slippi::de::{read, Opts},
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
					|buf| read(&mut Cursor::new(buf), None),
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
	name = bench_skip_frames;
	config = Criterion::default()
		.warm_up_time(Duration::from_secs(1));
	targets = skip_frames
}

criterion_main!(bench_into_game, bench_skip_frames);
