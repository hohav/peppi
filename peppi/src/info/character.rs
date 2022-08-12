use crate::model::enums::character::External;

#[derive(Clone)]
pub struct Info {
	pub external: External,
	pub short: &'static str,
	pub long: &'static str,
	pub traction: f32,
}

macro_rules! char_info {
	($info: ident {
		$($name: ident {
			regex: $regex: expr,
			$( $field: ident : $value: expr ),* $(,)?
		});* $(;)?
	}) => {
		mod regexes {
			use lazy_static::lazy_static;
			use regex::Regex;
			lazy_static! {
				$( pub static ref $name: Regex = Regex::new($regex).unwrap(); )*
			}
		}

		impl $info {
			pub fn try_from(external: External) -> Option<$info> {
				match external {
					$( External::$name => Some($name), )*
					_ => None
				}
			}

			pub fn try_match(s: &str) -> Option<$info> {
				match s {
					$( s if regexes::$name.is_match(s) => Some($name), )*
					_ => None,
				}
			}
		}

		$(pub const $name: $info = $info {
			external: External::$name,
			$( $field: $value ),*
		};)*
	}
}


char_info!(Info {
	CAPTAIN_FALCON {
		regex: r"(?i-u)(capt(ain|\.)?[ _]?)?falcon",
		short: "Falcon",
		long: "Captain Falcon",
		traction: 0.08,
	};

	DONKEY_KONG {
		regex: r"(?i-u)dk|(donkey[ _]?kong",
		short: "DK",
		long: "Donkey Kong",
		traction: 0.08,
	};

	FOX {
		regex: r"(?i-u)fox",
		short: "Fox",
		long: "Fox",
		traction: 0.08,
	};

	GAME_AND_WATCH {
		regex: r"(?i-u)(g|game)[ _]?(and|&|n)[ _]?(w|watch)",
		short: "G&W",
		long: "Game and Watch",
		traction: 0.06,
	};

	KIRBY {
		regex: r"(?i-u)kirby",
		short: "Kirby",
		long: "Kirby",
		traction: 0.08,
	};

	BOWSER {
		regex: r"(?i-u)bowser",
		short: "Bowser",
		long: "Bowser",
		traction: 0.06,
	};

	LINK {
		regex: r"(?i-u)link",
		short: "Link",
		long: "Link",
		traction: 0.1,
	};

	LUIGI {
		regex: r"(?i-u)luigi",
		short: "Luigi",
		long: "Luigi",
		traction: 0.025,
	};

	MARIO {
		regex: r"(?i-u)mario",
		short: "Mario",
		long: "Mario",
		traction: 0.06,
	};

	MARTH {
		regex: r"(?i-u)marth",
		short: "Marth",
		long: "Marth",
		traction: 0.06,
	};

	MEWTWO {
		regex: r"(?i-u)mewtwo|mew2|m2",
		short: "Mewtwo",
		long: "Mewtwo",
		traction: 0.04,
	};

	NESS {
		regex: r"(?i-u)ness",
		short: "Ness",
		long: "Ness",
		traction: 0.06,
	};

	PEACH {
		regex: r"(?i-u)peach",
		short: "Peach",
		long: "Peach",
		traction: 0.1,
	};

	PIKACHU {
		regex: r"(?i-u)pika(chu)?",
		short: "Pika",
		long: "Pikachu",
		traction: 0.09,
	};

	ICE_CLIMBERS {
		regex: r"(?i-u)ic(|s|ies|e[ _]?climbers)",
		short: "ICs",
		long: "Ice Climbers",
		traction: 0.035,
	};

	JIGGLYPUFF {
		regex: r"(?i-u)(jiggly)?puff|jiggs",
		short: "Puff",
		long: "Jigglypuff",
		traction: 0.09,
	};

	SAMUS {
		regex: r"(?i-u)samus",
		short: "Samus",
		long: "Samus",
		traction: 0.06,
	};

	YOSHI {
		regex: r"(?i-u)yoshi",
		short: "Yoshi",
		long: "Yoshi",
		traction: 0.06,
	};

	ZELDA {
		regex: r"(?i-u)zelda",
		short: "Zelda",
		long: "Zelda",
		traction: 0.1,
	};

	SHEIK {
		regex: r"(?i-u)sheik",
		short: "Sheik",
		long: "Sheik",
		traction: 0.08,
	};

	FALCO {
		regex: r"(?i-u)falco",
		short: "Falco",
		long: "Falco",
		traction: 0.08,
	};

	YOUNG_LINK {
		regex: r"(?i-u)(y\.?|young)[ _]?(l|link",
		short: "YL",
		long: "Young Link",
		traction: 0.08,
	};

	DR_MARIO {
		regex: r"(?i-u)doc|dr\.?[ _]?mario",
		short: "Doc",
		long: "Dr. Mario",
		traction: 0.06,
	};

	ROY {
		regex: r"(?i-u)roy",
		short: "Roy",
		long: "Roy",
		traction: 0.06,
	};

	PICHU {
		regex: r"(?i-u)pichu",
		short: "Pichu",
		long: "Pichu",
		traction: 0.1,
	};

	GANONDORF {
		regex: r"(?i-u)ganon(dorf)?",
		short: "Ganon",
		long: "Ganondorf",
		traction: 0.07,
	};

	MASTER_HAND {
		regex: r"(?i-u)master[ _]?hand",
		short: "Master Hand",
		long: "Master Hand",
		traction: 0.0,
	};

	WIRE_FRAME_MALE {
		regex: r"(?i-u)male[ _]?wire[ _]?frame|wire[ _]?frame[ _]?male",
		short: "Male Wireframe",
		long: "Male Wireframe",
		traction: 0.0,
	};

	WIRE_FRAME_FEMALE {
		regex: r"(?i-u)female[ _]?wire[ _]?frame|wire[ _]?frame[ _]?female",
		short: "Female Wireframe",
		long: "Female Wireframe",
		traction: 0.0,
	};

	GIGA_BOWSER {
		regex: r"(?i-u)giga[ _]?bowser",
		short: "Giga Bowser",
		long: "Giga Bowser",
		traction: 0.0,
	};

	CRAZY_HAND {
		regex: r"(?i-u)crazy[ _]?hand",
		short: "Crazy Hand",
		long: "Crazy Hand",
		traction: 0.0,
	};

	SANDBAG {
		regex: r"(?i-u)sandbag",
		short: "Sandbag",
		long: "Sandbag",
		traction: 0.0,
	};

	POPO {
		regex: r"(?i-u)popo",
		short: "Popo",
		long: "Popo",
		traction: 0.035,
	};
});
