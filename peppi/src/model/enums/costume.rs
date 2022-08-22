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

regex_match!(captainfalcon : CaptainFalcon {
	INDIGO: r"(?i-u)^default|indigo$",
	BLACK: r"(?i-u)^black$",
	RED: r"(?i-u)^red$",
	WHITE: r"(?i-u)^white|pink$",
	GREEN: r"(?i-u)^green$",
	BLUE: r"(?i-u)^blue$",
});

pseudo_enum!(DonkeyKong: u8 {
	0 => BROWN,
	1 => BLACK,
	2 => RED,
	3 => BLUE,
	4 => GREEN,
});

regex_match!(donkeykong : DonkeyKong {
	BROWN: r"(?i-u)^default|brown$",
	BLACK: r"(?i-u)^black$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue|purple$",
	GREEN: r"(?i-u)^green$",
});

pseudo_enum!(Fox: u8 {
	0 => WHITE,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
});

regex_match!(fox : Fox {
	WHITE: r"(?i-u)^default|white|tan$",
	RED: r"(?i-u)^red|orange$",
	BLUE: r"(?i-u)^blue|lavender|purple$",
	GREEN: r"(?i-u)^green$",
});

pseudo_enum!(GameAndWatch: u8 {
	0 => BLACK,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
});

regex_match!(gameandwatch : GameAndWatch {
	BLACK: r"(?i-u)^default|black$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
});

pseudo_enum!(Kirby: u8 {
	0 => PINK,
	1 => YELLOW,
	2 => BLUE,
	3 => RED,
	4 => GREEN,
	5 => WHITE,
});

regex_match!(kirby : Kirby {
	PINK: r"(?i-u)^default|pink$",
	YELLOW: r"(?i-u)^yellow$",
	BLUE: r"(?i-u)^blue$",
	RED: r"(?i-u)^red|orange$",
	GREEN: r"(?i-u)^green$",
	WHITE: r"(?i-u)^white$",
});

pseudo_enum!(Bowser: u8 {
	0 => GREEN,
	1 => RED,
	2 => BLUE,
	3 => BLACK,
});

regex_match!(bowser : Bowser {
	GREEN: r"(?i-u)^default|green$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	BLACK: r"(?i-u)^black$",
});

pseudo_enum!(Link: u8 {
	0 => GREEN,
	1 => RED,
	2 => BLUE,
	3 => BLACK,
	4 => WHITE,
});

regex_match!(link : Link {
	GREEN: r"(?i-u)^default|green$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	BLACK: r"(?i-u)^black$",
	WHITE: r"(?i-u)^white$",
});

pseudo_enum!(Luigi: u8 {
	0 => GREEN,
	1 => WHITE,
	2 => BLUE,
	3 => RED,
});

regex_match!(luigi : Luigi {
	GREEN: r"(?i-u)^default|green$",
	WHITE: r"(?i-u)^white$",
	BLUE: r"(?i-u)^blue$",
	RED: r"(?i-u)^red|pink$",
});

pseudo_enum!(Mario: u8 {
	0 => RED,
	1 => YELLOW,
	2 => BLACK,
	3 => BLUE,
	4 => GREEN,
});

regex_match!(mario : Mario {
	RED: r"(?i-u)^default|red$",
	YELLOW: r"(?i-u)^yellow|wario$",
	BLACK: r"(?i-u)^black|brown$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
});

pseudo_enum!(Marth: u8 {
	0 => BLUE,
	1 => RED,
	2 => GREEN,
	3 => BLACK,
	4 => WHITE,
});

regex_match!(marth : Marth {
	BLUE: r"(?i-u)^default|blue$",
	RED: r"(?i-u)^red$",
	GREEN: r"(?i-u)^green$",
	BLACK: r"(?i-u)^black$",
	WHITE: r"(?i-u)^white$",
});

pseudo_enum!(Mewtwo: u8 {
	0 => PURPLE,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
});

regex_match!(mewtwo : Mewtwo {
	PURPLE: r"(?i-u)^default|purple|white$",
	RED: r"(?i-u)^red|orange$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
});

pseudo_enum!(Ness: u8 {
	0 => RED,
	1 => YELLOW,
	2 => BLUE,
	3 => GREEN,
});

regex_match!(ness : Ness {
	RED: r"(?i-u)^default|red$",
	YELLOW: r"(?i-u)^yellow$",
	BLUE: r"(?i-u)^blue|purple$",
	GREEN: r"(?i-u)^green$",
});

pseudo_enum!(Peach: u8 {
	0 => RED,
	1 => YELLOW,
	2 => WHITE,
	3 => BLUE,
	4 => GREEN,
});

regex_match!(peach : Peach {
	RED: r"(?i-u)^default|red|pink$",
	YELLOW: r"(?i-u)^yellow|daisy$",
	WHITE: r"(?i-u)^white$",
	BLUE: r"(?i-u)^blue|purple$",
	GREEN: r"(?i-u)^green$",
});

pseudo_enum!(Pikachu: u8 {
	0 => YELLOW,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
});

regex_match!(pikachu : Pikachu {
	YELLOW: r"(?i-u)^default|yellow$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
});

pseudo_enum!(IceClimbers: u8 {
	0 => BLUE,
	1 => GREEN,
	2 => ORANGE,
	3 => RED,
});

regex_match!(iceclimbers : IceClimbers {
	BLUE: r"(?i-u)^default|blue|purple$",
	GREEN: r"(?i-u)^green$",
	ORANGE: r"(?i-u)^orange$",
	RED: r"(?i-u)^red$",
});

pseudo_enum!(Jigglypuff: u8 {
	0 => PINK,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
	4 => YELLOW,
});

regex_match!(jigglypuff : Jigglypuff {
	PINK: r"(?i-u)^default|pink$",
	RED: r"(?i-u)^red|flower$",
	BLUE: r"(?i-u)^blue|bow$",
	GREEN: r"(?i-u)^green|head[ _]?band$",
	YELLOW: r"(?i-u)^yellow|gold|crown$",
});

pseudo_enum!(Samus: u8 {
	0 => RED,
	1 => PINK,
	2 => BLACK,
	3 => GREEN,
	4 => BLUE,
});

regex_match!(samus : Samus {
	RED: r"(?i-u)^default|red|orange$",
	PINK: r"(?i-u)^pink$",
	BLACK: r"(?i-u)^black|brown$",
	GREEN: r"(?i-u)^green$",
	BLUE: r"(?i-u)^blue|purple$",
});

pseudo_enum!(Yoshi: u8 {
	0 => GREEN,
	1 => RED,
	2 => BLUE,
	3 => YELLOW,
	4 => PINK,
	5 => CYAN,
});

regex_match!(yoshi : Yoshi {
	GREEN: r"(?i-u)^default|green$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^(dark[ _]?)?blue$",
	YELLOW: r"(?i-u)^yellow$",
	PINK: r"(?i-u)^pink$",
	CYAN: r"(?i-u)^cyan|light[ _]?blue$",
});

pseudo_enum!(Zelda: u8 {
	0 => PINK,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
	4 => WHITE,
});

regex_match!(zelda : Zelda {
	PINK: r"(?i-u)^default|pink$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
	WHITE: r"(?i-u)^white$",
});

pseudo_enum!(Sheik: u8 {
	0 => NAVY,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
	4 => WHITE,
});

regex_match!(sheik : Sheik {
	NAVY: r"(?i-u)^default|navy$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
	WHITE: r"(?i-u)^white$",
});

pseudo_enum!(Falco: u8 {
	0 => TAN,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
});

regex_match!(falco : Falco {
	TAN: r"(?i-u)^default|tan$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
});

pseudo_enum!(YoungLink: u8 {
	0 => GREEN,
	1 => RED,
	2 => BLUE,
	3 => WHITE,
	4 => BLACK,
});

regex_match!(younglink : YoungLink {
	GREEN: r"(?i-u)^default|green$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	WHITE: r"(?i-u)^white$",
	BLACK: r"(?i-u)^black$",
});

pseudo_enum!(DrMario: u8 {
	0 => WHITE,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
	4 => BLACK,
});

regex_match!(drmario : DrMario {
	WHITE: r"(?i-u)^default|white$",
	RED: r"(?i-u)^red|salmon|pink$",
	BLUE: r"(?i-u)^blue|purple$",
	GREEN: r"(?i-u)^green$",
	BLACK: r"(?i-u)^black$",
});

pseudo_enum!(Roy: u8 {
	0 => PURPLE,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
	4 => YELLOW,
});

regex_match!(roy : Roy {
	PURPLE: r"(?i-u)^default|purple$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
	YELLOW: r"(?i-u)^yellow|gold$",
});

pseudo_enum!(Pichu: u8 {
	0 => YELLOW,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
});

regex_match!(pichu : Pichu {
	YELLOW: r"(?i-u)^default|yellow$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
});

pseudo_enum!(Ganondorf: u8 {
	0 => BROWN,
	1 => RED,
	2 => BLUE,
	3 => GREEN,
	4 => PURPLE,
});

regex_match!(ganondorf : Ganondorf {
	BROWN: r"(?i-u)^default|brown$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
	PURPLE: r"(?i-u)^purple$",
});
