use peppi::{self, serde::de};
use std::{fs::File, io::BufReader, path::PathBuf};

fn game(replay: &str, skip_frames: bool) {
	let path = PathBuf::from(format!("benches/data/{}.slp", replay));
	let mut buf = BufReader::new(File::open(path).unwrap());
	let opts = de::Opts {
		skip_frames,
		..Default::default()
	};
	peppi::game(&mut buf, Some(&opts)).unwrap();
}

fn casual_doubles() {
	game("casual_doubles", false)
}

fn casual_doubles_skip_frames() {
	game("casual_doubles", true)
}

fn hbox_llod_timeout_g8() {
	game("hbox_llod_timeout_g8", false)
}

fn hbox_llod_timeout_g8_skip_frames() {
	game("hbox_llod_timeout_g8", true)
}

fn ics_ditto() {
	game("ics_ditto", false)
}

fn ics_ditto_skip_frames() {
	game("ics_ditto", true)
}

fn mango_zain_netplay() {
	game("mango_zain_netplay", false)
}

fn mango_zain_netplay_skip_frames() {
	game("mango_zain_netplay", true)
}

fn old_ver_thegang() {
	game("old_ver_thegang", false)
}

fn old_ver_thegang_skip_frames() {
	game("old_ver_thegang", true)
}

fn short_game_tbh10() {
	game("short_game_tbh10", false)
}

fn short_game_tbh10_skip_frames() {
	game("short_game_tbh10", true)
}

iai::main!(
	casual_doubles,
	casual_doubles_skip_frames,
	hbox_llod_timeout_g8,
	hbox_llod_timeout_g8_skip_frames,
	ics_ditto,
	ics_ditto_skip_frames,
	mango_zain_netplay,
	mango_zain_netplay_skip_frames,
	old_ver_thegang,
	old_ver_thegang_skip_frames,
	short_game_tbh10,
	short_game_tbh10_skip_frames,
);
