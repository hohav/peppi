use peppi::{
	self,
	io::slippi::de::{read, Opts},
};
use std::{fs::File, io::BufReader, path::PathBuf};

use iai_callgrind::{library_benchmark, library_benchmark_group, main};

fn game(replay: &str, skip_frames: bool) {
	let path = PathBuf::from(format!("benches/data/{}.slp", replay));
	let mut buf = BufReader::new(File::open(path).unwrap());
	read(
		&mut buf,
		Some(&Opts {
			skip_frames,
			..Default::default()
		}),
	)
}

#[library_benchmark]
fn casual_doubles() {
	game("casual_doubles", false)
}

#[library_benchmark]
fn casual_doubles_skip_frames() {
	game("casual_doubles", true)
}

#[library_benchmark]
fn hbox_llod_timeout_g8() {
	game("hbox_llod_timeout_g8", false)
}

#[library_benchmark]
fn hbox_llod_timeout_g8_skip_frames() {
	game("hbox_llod_timeout_g8", true)
}

#[library_benchmark]
fn ics_ditto() {
	game("ics_ditto", false)
}

#[library_benchmark]
fn ics_ditto_skip_frames() {
	game("ics_ditto", true)
}

#[library_benchmark]
fn mango_zain_netplay() {
	game("mango_zain_netplay", false)
}

#[library_benchmark]
fn mango_zain_netplay_skip_frames() {
	game("mango_zain_netplay", true)
}

#[library_benchmark]
fn old_ver_thegang() {
	game("old_ver_thegang", false)
}

#[library_benchmark]
fn old_ver_thegang_skip_frames() {
	game("old_ver_thegang", true)
}

#[library_benchmark]
fn short_game_tbh10() {
	game("short_game_tbh10", false)
}

#[library_benchmark]
fn short_game_tbh10_skip_frames() {
	game("short_game_tbh10", true)
}

library_benchmark_group!(
	name = full;
	benchmarks =
		casual_doubles,
		hbox_llod_timeout_g8,
		ics_ditto,
		mango_zain_netplay,
		old_ver_thegang,
		short_game_tbh10
);

library_benchmark_group!(
	name = skip_frames;
	benchmarks =
		casual_doubles_skip_frames,
		hbox_llod_timeout_g8_skip_frames,
		ics_ditto_skip_frames,
		mango_zain_netplay_skip_frames,
		old_ver_thegang_skip_frames,
		short_game_tbh10_skip_frames
);

main!(library_benchmark_groups = full, skip_frames);
