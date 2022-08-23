use std::fmt;
use super::character::External;

macro_rules! costume {
	($name: ident {
		$unknown: ident,
		$( $variant: ident ( $variant_type: ident ) => $external: ident ),* $(,)?
	}) => {
		#[derive(Copy, Clone, PartialEq, Eq, serde::Serialize)]
		#[serde(untagged)]
		pub enum $name {
			$unknown(u8),
			$( $variant($variant_type), )*
		}

		impl $name {
			pub fn from(value: u8, character: External) -> $name {
				match character {
					$( External::$external => $name::$variant($variant_type(value)), )*
					_ => $name::$unknown(value),
				}
			}

			#[cfg(feature = "regex_match")]
			pub fn try_match(character: External, s: &str) -> Option<$name> {
				use crate::regex::Regex;
				match character {
					$( External::$external => $variant_type::try_match(s).map($name::$variant), )*
					_ => None,
				}
			}

			pub fn default(character: External) -> $name {
				match character {
					$( External::$external => $name::$variant($variant_type(0)), )*
					_ => $name::$unknown(0),
				}
			}

			pub fn character(self) -> Option<External> {
				match self {
					$name::$unknown(_) => None,
					$( $name::$variant(_) => Some(External::$external), )*
				}
			}
		}

		impl From<$name> for u8 {
			fn from(state: $name) -> u8 {
				match state {
					$name::$unknown(s) => s,
					$( $name::$variant(s) => s.0, )*
				}
			}
		}

		impl fmt::Debug for $name {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				 match *self {
					$name::$unknown(s) => write!(f, "{:?}", s),
					$( $name::$variant(s) => write!(f, "{:?}", s), )*
				}
			}
		}
	}
}

costume!(Costume {
	Unknown,
	CaptainFalcon(CaptainFalcon) => CAPTAIN_FALCON,
	DonkeyKong(DonkeyKong) => DONKEY_KONG,
	Fox(Fox) => FOX,
	GameAndWatch(GameAndWatch) => GAME_AND_WATCH,
	Kirby(Kirby) => KIRBY,
	Bowser(Bowser) => BOWSER,
	Link(Link) => LINK,
	Luigi(Luigi) => LUIGI,
	Mario(Mario) => MARIO,
	Marth(Marth) => MARTH,
	Mewtwo(Mewtwo) => MEWTWO,
	Ness(Ness) => NESS,
	Peach(Peach) => PEACH,
	Pikachu(Pikachu) => PIKACHU,
	IceClimbers(IceClimbers) => ICE_CLIMBERS,
	Jigglypuff(Jigglypuff) => JIGGLYPUFF,
	Samus(Samus) => SAMUS,
	Yoshi(Yoshi) => YOSHI,
	Zelda(Zelda) => ZELDA,
	Sheik(Sheik) => SHEIK,
	Falco(Falco) => FALCO,
	YoungLink(YoungLink) => YOUNG_LINK,
	DrMario(DrMario) => DR_MARIO,
	Roy(Roy) => ROY,
	Pichu(Pichu) => PICHU,
	Ganondorf(Ganondorf) => GANONDORF,
});

pseudo_enum!(CaptainFalcon: u8 {
	0 => INDIGO,
	1 => BLACK,
	2 => RED,
	3 => WHITE,
	4 => GREEN,
	5 => BLUE,
});

pseudo_enum!(DonkeyKong: u8 {
	0 => BROWN,
	1 => BLACK,
	2 => RED,
	3 => BLUE,
	4 => GREEN,
});

pseudo_enum!(Fox: u8 {
	0 => WHITE,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
});

pseudo_enum!(GameAndWatch: u8 {
	0 => BLACK,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
});

pseudo_enum!(Kirby: u8 {
	0 => PINK,
	1 => YELLOW,
	2 => BLUE,
	3 => RED,
	4 => GREEN,
	5 => WHITE,
});

pseudo_enum!(Bowser: u8 {
	0 => GREEN,
	1 => RED,
	2 => BLUE,
	3 => BLACK,
});

pseudo_enum!(Link: u8 {
	0 => GREEN,
	1 => RED,
	2 => BLUE,
	3 => BLACK,
	4 => WHITE,
});

pseudo_enum!(Luigi: u8 {
	0 => GREEN,
	1 => WHITE,
	2 => BLUE,
	3 => RED,
});

pseudo_enum!(Mario: u8 {
	0 => RED,
	1 => YELLOW,
	2 => BLACK,
	3 => BLUE,
	4 => GREEN,
});

pseudo_enum!(Marth: u8 {
	0 => BLUE,
	1 => RED,
	2 => GREEN,
	3 => BLACK,
	4 => WHITE,
});

pseudo_enum!(Mewtwo: u8 {
	0 => PURPLE,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
});

pseudo_enum!(Ness: u8 {
	0 => RED,
	1 => YELLOW,
	2 => BLUE,
	3 => GREEN,
});

pseudo_enum!(Peach: u8 {
	0 => RED,
	1 => YELLOW,
	2 => WHITE,
	3 => BLUE,
	4 => GREEN,
});

pseudo_enum!(Pikachu: u8 {
	0 => YELLOW,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
});

pseudo_enum!(IceClimbers: u8 {
	0 => BLUE,
	1 => GREEN,
	2 => ORANGE,
	3 => RED,
});

pseudo_enum!(Jigglypuff: u8 {
	0 => PINK,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
	4 => YELLOW,
});

pseudo_enum!(Samus: u8 {
	0 => RED,
	1 => PINK,
	2 => BLACK,
	3 => GREEN,
	4 => BLUE,
});

pseudo_enum!(Yoshi: u8 {
	0 => GREEN,
	1 => RED,
	2 => BLUE,
	3 => YELLOW,
	4 => PINK,
	5 => CYAN,
});

pseudo_enum!(Zelda: u8 {
	0 => PINK,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
	4 => WHITE,
});

pseudo_enum!(Sheik: u8 {
	0 => NAVY,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
	4 => WHITE,
});

pseudo_enum!(Falco: u8 {
	0 => TAN,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
});

pseudo_enum!(YoungLink: u8 {
	0 => GREEN,
	1 => RED,
	2 => BLUE,
	3 => WHITE,
	4 => BLACK,
});

pseudo_enum!(DrMario: u8 {
	0 => WHITE,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
	4 => BLACK,
});

pseudo_enum!(Roy: u8 {
	0 => PURPLE,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
	4 => YELLOW,
});

pseudo_enum!(Pichu: u8 {
	0 => YELLOW,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
});

pseudo_enum!(Ganondorf: u8 {
	0 => BROWN,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
	4 => PURPLE,
});
