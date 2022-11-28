mod common;
use common::game;

#[test]
fn crazy_name_tags() {
	let game = game("crazy_name_tags");
	let tag1 = game.start.players[0].name_tag.as_ref().unwrap();
	let tag2 = game.start.players[1].name_tag.as_ref().unwrap();
	let tag3 = game.start.players[2].name_tag.as_ref().unwrap();
	let tag4 = game.start.players[3].name_tag.as_ref().unwrap();

	assert_eq!(tag1.as_str(), "！　CLOWN");
	assert_eq!(tag2.as_str(), "C＠ぞ～");
	assert_eq!(tag3.as_str(), "A ＄ホ ぬヅ。");
	assert_eq!(tag4.as_str(), "！！！！！！！！");

	assert_eq!(tag1.to_normalized(), "! CLOWN");
	assert_eq!(tag2.to_normalized(), "C@ぞ~");
	assert_eq!(tag3.to_normalized(), "A $ホ ぬヅ。");
	assert_eq!(tag4.to_normalized(), "!!!!!!!!");
}
