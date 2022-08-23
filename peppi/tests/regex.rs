#[cfg(feature = "regex_match")]
mod regex {
	use peppi::model::enums::character::External;
	use peppi::model::enums::costume::{self, Costume};
	use peppi::regex::Regex;

	#[test]
	fn regex() -> Result<(), String> {
		let character = External::try_match("falcon");
		assert_eq!(character.unwrap(), External::CAPTAIN_FALCON);

		let costume = Costume::try_match(External::LUIGI, "pink");
		assert_eq!(costume.unwrap(), Costume::Luigi(costume::Luigi::RED));

		Ok(())
	}
}
