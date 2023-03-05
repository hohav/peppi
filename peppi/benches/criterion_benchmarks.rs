use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use peppi::{self, serde::de};

use std::{fs, path::PathBuf, time::Duration};

struct DumbHandler;
impl de::Handlers for DumbHandler {}

pub fn bench(c: &mut Criterion) {
	let mut group = c.benchmark_group("replays");
	let dir = PathBuf::from("benches/data");
	for replay in fs::read_dir(dir).unwrap() {
		let path = replay.unwrap().path();
		let name = path.file_name().unwrap().to_str().unwrap().to_string();
		let contents = fs::read(path).unwrap();

		group.throughput(Throughput::Bytes(contents.len().try_into().unwrap()));
		group.bench_with_input(
			BenchmarkId::new("into_game", &name),
			&contents,
			|b, contents| {
				b.iter_batched(
					|| contents.as_slice(),
					|mut buf| peppi::game(&mut buf, None, None),
					BatchSize::LargeInput,
				)
			},
		);
		group.bench_with_input(
			BenchmarkId::new("event_handlers", &name),
			&contents,
			|b, contents| {
				b.iter_batched(
					|| contents.as_slice(),
					|mut buf| peppi::parse(&mut buf, &mut DumbHandler, None),
					BatchSize::SmallInput,
				)
			},
		);
		group.bench_with_input(
			BenchmarkId::new("skip_frames", &name),
			&contents,
			|b, contents| {
				b.iter_batched(
					|| contents.as_slice(),
					|mut buf| {
						let opts = de::Opts {
							skip_frames: true,
                            ..Default::default()
						};
						peppi::game(&mut buf, Some(&opts), None)
					},
					BatchSize::SmallInput,
				)
			},
		);
	}

	group.finish();
}
criterion_group!{
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_secs(1));
    targets = bench
}
criterion_main!(benches);
