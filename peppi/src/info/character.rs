use crate::model::enums::character::External;

#[non_exhaustive]
#[derive(Clone)]
pub struct Info {
	pub external: External,
	pub short: &'static str,
	pub long: &'static str,
	pub traction: f32,
}

macro_rules! info {
	($type: ident => $info: ident {
		$($name: ident {
			$( $field: ident : $value: expr ),+ $(,)?
		});+ $(;)?
	}) => {
		impl $info {
			pub fn try_from(external: $type) -> Option<$info> {
				match external {
					$( $type::$name => Some($name), )*
					_ => None
				}
			}
		}

		$(pub const $name: $info = $info {
			$( $field: $value ),*
		};)*
	}
}

macro_rules! info_regex {
	($info: ident {
        $($name: ident : $regex: expr),+ $(,)?
    }) => {
		mod regexes {
			use lazy_static::lazy_static;
			use regex::Regex;
			lazy_static! {
				$( pub static ref $name: Regex = Regex::new($regex).unwrap(); )*
			}
		}

		impl $info {
			pub fn try_match(s: &str) -> Option<$info> {
				match s {
					$( s if regexes::$name.is_match(s) => Some($name), )*
					_ => None,
				}
			}
		}
	}
}

info!(External => Info {
	CAPTAIN_FALCON {
        external: External::CAPTAIN_FALCON,
		short: "Falcon",
		long: "Captain Falcon",
		traction: 0.08,
	};

	DONKEY_KONG {
        external: External::DONKEY_KONG,
		short: "DK",
		long: "Donkey Kong",
		traction: 0.08,
	};

	FOX {
        external: External::FOX,
		short: "Fox",
		long: "Fox",
		traction: 0.08,
	};

	GAME_AND_WATCH {
        external: External::GAME_AND_WATCH,
		short: "G&W",
		long: "Game and Watch",
		traction: 0.06,
	};

	KIRBY {
        external: External::KIRBY,
		short: "Kirby",
		long: "Kirby",
		traction: 0.08,
	};

	BOWSER {
        external: External::BOWSER,
		short: "Bowser",
		long: "Bowser",
		traction: 0.06,
	};

	LINK {
        external: External::LINK,
		short: "Link",
		long: "Link",
		traction: 0.1,
	};

	LUIGI {
        external: External::LUIGI,
		short: "Luigi",
		long: "Luigi",
		traction: 0.025,
	};

	MARIO {
        external: External::MARIO,
		short: "Mario",
		long: "Mario",
		traction: 0.06,
	};

	MARTH {
        external: External::MARTH,
		short: "Marth",
		long: "Marth",
		traction: 0.06,
	};

	MEWTWO {
        external: External::MEWTWO,
		short: "Mewtwo",
		long: "Mewtwo",
		traction: 0.04,
	};

	NESS {
        external: External::NESS,
		short: "Ness",
		long: "Ness",
		traction: 0.06,
	};

	PEACH {
        external: External::PEACH,
		short: "Peach",
		long: "Peach",
		traction: 0.1,
	};

	PIKACHU {
        external: External::PIKACHU,
		short: "Pika",
		long: "Pikachu",
		traction: 0.09,
	};

	ICE_CLIMBERS {
        external: External::ICE_CLIMBERS,
		short: "ICs",
		long: "Ice Climbers",
		traction: 0.035,
	};

	JIGGLYPUFF {
        external: External::JIGGLYPUFF,
		short: "Puff",
		long: "Jigglypuff",
		traction: 0.09,
	};

	SAMUS {
        external: External::SAMUS,
		short: "Samus",
		long: "Samus",
		traction: 0.06,
	};

	YOSHI {
        external: External::YOSHI,
		short: "Yoshi",
		long: "Yoshi",
		traction: 0.06,
	};

	ZELDA {
        external: External::ZELDA,
		short: "Zelda",
		long: "Zelda",
		traction: 0.1,
	};

	SHEIK {
        external: External::SHEIK,
		short: "Sheik",
		long: "Sheik",
		traction: 0.08,
	};

	FALCO {
        external: External::FALCO,
		short: "Falco",
		long: "Falco",
		traction: 0.08,
	};

	YOUNG_LINK {
        external: External::YOUNG_LINK,
		short: "YL",
		long: "Young Link",
		traction: 0.08,
	};

	DR_MARIO {
        external: External::DR_MARIO,
		short: "Doc",
		long: "Dr. Mario",
		traction: 0.06,
	};

	ROY {
        external: External::ROY,
		short: "Roy",
		long: "Roy",
		traction: 0.06,
	};

	PICHU {
        external: External::PICHU,
		short: "Pichu",
		long: "Pichu",
		traction: 0.1,
	};

	GANONDORF {
        external: External::GANONDORF,
		short: "Ganon",
		long: "Ganondorf",
		traction: 0.07,
	};

	MASTER_HAND {
        external: External::MASTER_HAND,
		short: "Master Hand",
		long: "Master Hand",
		traction: 0.0,
	};

	WIRE_FRAME_MALE {
        external: External::WIRE_FRAME_MALE,
		short: "Male Wireframe",
		long: "Male Wireframe",
		traction: 0.0,
	};

	WIRE_FRAME_FEMALE {
        external: External::WIRE_FRAME_FEMALE,
		short: "Female Wireframe",
		long: "Female Wireframe",
		traction: 0.0,
	};

	GIGA_BOWSER {
        external: External::GIGA_BOWSER,
		short: "Giga Bowser",
		long: "Giga Bowser",
		traction: 0.0,
	};

	CRAZY_HAND {
        external: External::CRAZY_HAND,
		short: "Crazy Hand",
		long: "Crazy Hand",
		traction: 0.0,
	};

	SANDBAG {
        external: External::SANDBAG,
		short: "Sandbag",
		long: "Sandbag",
		traction: 0.0,
	};

	POPO {
        external: External::POPO,
		short: "Popo",
		long: "Popo",
		traction: 0.035,
	};
});

info_regex!(Info {
    CAPTAIN_FALCON: r"(?i-u)^(capt(ain|\.)?[ _]?)?falcon$",
    DONKEY_KONG: r"(?i-u)^dk|(donkey[ _]?kong$",
    FOX: r"(?i-u)^fox$",
    GAME_AND_WATCH: r"(?i-u)^(g|game)[ _]?(and|&|n)[ _]?(w|watch)$",
    KIRBY: r"(?i-u)^kirby$",
    BOWSER: r"(?i-u)^bowser$",
    LINK: r"(?i-u)^link$",
    LUIGI: r"(?i-u)^luigi$",
    MARIO: r"(?i-u)^mario$",
    MARTH: r"(?i-u)^marth$",
    MEWTWO: r"(?i-u)^mewtwo|mew2|m2$",
    NESS: r"(?i-u)^ness$",
    PEACH: r"(?i-u)^peach$",
    PIKACHU: r"(?i-u)^pika(chu)?$",
    ICE_CLIMBERS: r"(?i-u)^ic(|s|ies|e[ _]?climbers)$",
    JIGGLYPUFF: r"(?i-u)^(jiggly)?puff|jiggs$",
    SAMUS: r"(?i-u)^samus$",
    YOSHI: r"(?i-u)^yoshi$",
    ZELDA: r"(?i-u)^zelda$",
    SHEIK: r"(?i-u)^sheik$",
    FALCO: r"(?i-u)^falco$",
    YOUNG_LINK: r"(?i-u)^(y\.?|young)[ _]?(l|link$",
    DR_MARIO: r"(?i-u)^doc|dr\.?[ _]?mario$",
    ROY: r"(?i-u)^roy$",
    PICHU: r"(?i-u)^pichu$",
    GANONDORF: r"(?i-u)^ganon(dorf)?$",
    MASTER_HAND: r"(?i-u)^master[ _]?hand$",
    WIRE_FRAME_MALE: r"(?i-u)^male[ _]?wire[ _]?frame|wire[ _]?frame[ _]?male$",
    WIRE_FRAME_FEMALE: r"(?i-u)^female[ _]?wire[ _]?frame|wire[ _]?frame[ _]?female$",
    GIGA_BOWSER: r"(?i-u)^giga[ _]?bowser$",
    CRAZY_HAND: r"(?i-u)^crazy[ _]?hand$",
    SANDBAG: r"(?i-u)^sandbag$",
    POPO: r"(?i-u)^popo$",
});
