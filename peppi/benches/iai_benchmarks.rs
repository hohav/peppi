use peppi::{self, serde::de};
use std::{fs::File, io::BufReader, path::PathBuf};

struct DumbHandler;
impl de::Handlers for DumbHandler {}

fn path(replay: &str) -> PathBuf {
	PathBuf::from(format!("benches/data/{}.slp", replay))
}

fn into_game(path: PathBuf) {
	let mut buf = BufReader::new(File::open(path).unwrap());
	peppi::game(&mut buf, None, None).unwrap();
}

fn event_handlers(path: PathBuf) {
	let mut buf = BufReader::new(File::open(path).unwrap());
	peppi::parse(&mut buf, &mut DumbHandler, None).unwrap();
}

fn skip_frames(path: PathBuf) {
	let mut buf = BufReader::new(File::open(path).unwrap());
	let opts = de::Opts {
		skip_frames: true,
		debug_dir: None,
	};
	peppi::game(&mut buf, Some(&opts), None).unwrap();
}

fn casual_doubles_into_game() {
	into_game(path("casual_doubles"));
}
fn casual_doubles_event_handlers() {
	event_handlers(path("casual_doubles"));
}
fn casual_doubles_skip_frames() {
	skip_frames(path("casual_doubles"));
}
fn hbox_llod_timeout_g8_into_game() {
	into_game(path("hbox_llod_timeout_g8"));
}
fn hbox_llod_timeout_g8_event_handlers() {
	event_handlers(path("hbox_llod_timeout_g8"));
}
fn hbox_llod_timeout_g8_skip_frames() {
	skip_frames(path("hbox_llod_timeout_g8"));
}
fn ics_ditto_into_game() {
	into_game(path("ics_ditto"));
}
fn ics_ditto_event_handlers() {
	event_handlers(path("ics_ditto"));
}
fn ics_ditto_skip_frames() {
	skip_frames(path("ics_ditto"));
}
fn long_pause_into_game() {
	into_game(path("long_pause"));
}
fn long_pause_event_handlers() {
	event_handlers(path("long_pause"));
}
fn long_pause_skip_frames() {
	skip_frames(path("long_pause"));
}
fn mango_zain_netplay_into_game() {
	into_game(path("mango_zain_netplay"));
}
fn mango_zain_netplay_event_handlers() {
	event_handlers(path("mango_zain_netplay"));
}
fn mango_zain_netplay_skip_frames() {
	skip_frames(path("mango_zain_netplay"));
}
fn old_ver_thegang_into_game() {
	into_game(path("old_ver_thegang"));
}
fn old_ver_thegang_event_handlers() {
	event_handlers(path("old_ver_thegang"));
}
fn old_ver_thegang_skip_frames() {
	skip_frames(path("old_ver_thegang"));
}
fn short_game_tbh10_into_game() {
	into_game(path("short_game_tbh10"));
}
fn short_game_tbh10_event_handlers() {
	event_handlers(path("short_game_tbh10"));
}
fn short_game_tbh10_skip_frames() {
	skip_frames(path("short_game_tbh10"));
}

iai::main!(
	// casual_doubles_into_game,
	// casual_doubles_event_handlers,
	casual_doubles_skip_frames,
	hbox_llod_timeout_g8_into_game,
	hbox_llod_timeout_g8_event_handlers,
	hbox_llod_timeout_g8_skip_frames,
	ics_ditto_into_game,
	ics_ditto_event_handlers,
	ics_ditto_skip_frames,
	long_pause_into_game,
	long_pause_event_handlers,
	long_pause_skip_frames,
	mango_zain_netplay_into_game,
	mango_zain_netplay_event_handlers,
	mango_zain_netplay_skip_frames,
	old_ver_thegang_into_game,
	old_ver_thegang_event_handlers,
	old_ver_thegang_skip_frames,
	short_game_tbh10_into_game,
	short_game_tbh10_event_handlers,
	short_game_tbh10_skip_frames,
);
