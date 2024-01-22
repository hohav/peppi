use peppi::{
	self,
	io::slippi::de::{self, Opts},
};
use std::{hint::black_box, io::Cursor, path::PathBuf};

use iai_callgrind::{library_benchmark, library_benchmark_group, main};

fn read(replay: &str) -> Vec<u8> {
	let path = PathBuf::from(format!("benches/data/{}.slp", replay));
	std::fs::read(path).unwrap()
}

#[library_benchmark]
#[benches::parse_full(
	read("casual_doubles"),
	read("hbox_llod_timeout_g8"),
	read("ics_ditto"),
	read("mango_zain_netplay"),
	read("old_ver_thegang"),
	read("short_game_tbh10")
)]
fn parse_full(buf: Vec<u8>) {
	black_box(de::read(&mut Cursor::new(&buf[..]), None).unwrap());
}

#[library_benchmark]
#[benches::parse_skip_frames(
	read("casual_doubles"),
	read("hbox_llod_timeout_g8"),
	read("ics_ditto"),
	read("mango_zain_netplay"),
	read("old_ver_thegang"),
	read("short_game_tbh10")
)]
fn parse_skip_frames(buf: Vec<u8>) {
	let opts = Opts {
		skip_frames: true,
		..Default::default()
	};
	black_box(de::read(&mut Cursor::new(&buf[..]), Some(&opts)).unwrap());
}

library_benchmark_group!(
	name = full;
	benchmarks = parse_full
);

library_benchmark_group!(
	name = skip_frames;
	benchmarks = parse_skip_frames
);

main!(library_benchmark_groups = full, skip_frames);
