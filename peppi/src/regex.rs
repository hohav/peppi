use crate::model::enums::character::External;
use crate::model::enums::stage::Stage;
use crate::model::enums::costume::*;

pub trait Regex {
	fn try_match(s: &str) -> Option<Self> where Self: Sized;
}

macro_rules! regex_match {
	($module_name: ident : $type: ident {
		$($name: ident : $regex: expr),+ $(,)?
	}) => {
		#[cfg(feature = "regex_match")]
		mod $module_name {
			use lazy_static::lazy_static;
			use regex::Regex;
			lazy_static! {
				$( pub static ref $name: Regex = Regex::new($regex).unwrap(); )*
			}
		}

		#[cfg(feature = "regex_match")]
		impl Regex for $type {
			fn try_match(s: &str) -> Option<$type> {
				if !s.is_ascii() || s.len() > 50 {
					return None
				}

				match s {
					$( s if $module_name::$name.is_match(s) => Some($type::$name), )*
					_ => None,
				}
			}
		}
	}
}

regex_match!(external : External {
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
	YOUNG_LINK: r"(?i-u)^yl|yink|(y\.?|young)[ _]?link$",
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

regex_match!(stage : Stage {
	FOUNTAIN_OF_DREAMS: r"(?i-u)^fod|fountain([ _]?of[ _]?dreams)?$",
	POKEMON_STADIUM: r"(?i-u)^ps|pokemon([ _]?stadium([ _]?[1I])?)?$",
	YOSHIS_STORY: r"(?i-u)^ys|yoshi'?s[ _]?story$",
	DREAM_LAND_N64: r"(?i-u)^(dl|dream[ _]?land)[ _]?(N?64)?$",
	BATTLEFIELD: r"(?i-u)^bf|battle[ _]?field$",
	FINAL_DESTINATION: r"(?i-u)^fd|final[ _]?destination$",
	PRINCESS_PEACHS_CASTLE: r"(?i-u)^ppc|princess[ _]?peach'?s[ _]?castle$",
	KONGO_JUNGLE: r"(?i-u)^kj|kongo[ _]?jungle$",
	BRINSTAR: r"(?i-u)^brinstar$",
	CORNERIA: r"(?i-u)^corneria$",
	ONETT: r"(?i-u)^onett$",
	MUTE_CITY: r"(?i-u)^mc|mute[ _]?city$",
	RAINBOW_CRUISE: r"(?i-u)^rc|rainbow[ _]?cruise$",
	JUNGLE_JAPES: r"(?i-u)^jj|jungle[ _]?japes$",
	GREAT_BAY: r"(?i-u)^gb|great[ _]?bay$",
	HYRULE_TEMPLE: r"(?i-u)^ht|hyrule[ _]?temple$",
	BRINSTAR_DEPTHS: r"(?i-u)^bd|brinstar[ _]?depths$",
	YOSHIS_ISLAND: r"(?i-u)^yi|yoshi'?s[ _]?island$",
	GREEN_GREENS: r"(?i-u)^gg|green[ _]?greens$",
	FOURSIDE: r"(?i-u)^fourside$",
	MUSHROOM_KINGDOM_I: r"(?i-u)^(mk|mushroom[ _]?kingdom)[1i]?$",
	MUSHROOM_KINGDOM_II: r"(?i-u)^(mk|mushroom[ _]?kingdom)(2|ii)$",
	VENOM: r"(?i-u)^venom$",
	POKE_FLOATS: r"(?i-u)^pf|poke[ _]?floats$",
	BIG_BLUE: r"(?i-u)^bb|big[ _]?blue$",
	ICICLE_MOUNTAIN: r"(?i-u)^im|icicle[ _]?mountain$",
	FLAT_ZONE: r"(?i-u)^fz|flat[ _]?zone$",
	YOSHIS_ISLAND_N64: r"(?i-u)^(yi|yoshi'?s[ _]?island)[ _]?(N?64)$",
	KONGO_JUNGLE_N64: r"(?i-u)^(kj|kongo[ _]?jungle)[ _]?(N?64)$",
});

regex_match!(costume_captainfalcon : CaptainFalcon {
	INDIGO: r"(?i-u)^default|indigo$",
	BLACK: r"(?i-u)^black$",
	RED: r"(?i-u)^red$",
	WHITE: r"(?i-u)^white|pink$",
	GREEN: r"(?i-u)^green$",
	BLUE: r"(?i-u)^blue$",
});

regex_match!(costume_donkeykong : DonkeyKong {
	BROWN: r"(?i-u)^default|brown$",
	BLACK: r"(?i-u)^black$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue|purple$",
	GREEN: r"(?i-u)^green$",
});

regex_match!(costume_fox : Fox {
	WHITE: r"(?i-u)^default|white|tan$",
	RED: r"(?i-u)^red|orange$",
	BLUE: r"(?i-u)^blue|lavender|purple$",
	GREEN: r"(?i-u)^green$",
});

regex_match!(costume_gameandwatch : GameAndWatch {
	BLACK: r"(?i-u)^default|black$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
});

regex_match!(costume_kirby : Kirby {
	PINK: r"(?i-u)^default|pink$",
	YELLOW: r"(?i-u)^yellow$",
	BLUE: r"(?i-u)^blue$",
	RED: r"(?i-u)^red|orange$",
	GREEN: r"(?i-u)^green$",
	WHITE: r"(?i-u)^white$",
});

regex_match!(costume_bowser : Bowser {
	GREEN: r"(?i-u)^default|green$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	BLACK: r"(?i-u)^black$",
});

regex_match!(costume_link : Link {
	GREEN: r"(?i-u)^default|green$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	BLACK: r"(?i-u)^black$",
	WHITE: r"(?i-u)^white$",
});

regex_match!(costume_luigi : Luigi {
	GREEN: r"(?i-u)^default|green$",
	WHITE: r"(?i-u)^white$",
	BLUE: r"(?i-u)^blue$",
	RED: r"(?i-u)^red|pink$",
});

regex_match!(costume_mario : Mario {
	RED: r"(?i-u)^default|red$",
	YELLOW: r"(?i-u)^yellow|wario$",
	BLACK: r"(?i-u)^black|brown$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
});

regex_match!(costume_marth : Marth {
	BLUE: r"(?i-u)^default|blue$",
	RED: r"(?i-u)^red$",
	GREEN: r"(?i-u)^green$",
	BLACK: r"(?i-u)^black$",
	WHITE: r"(?i-u)^white$",
});

regex_match!(costume_mewtwo : Mewtwo {
	PURPLE: r"(?i-u)^default|purple|white$",
	RED: r"(?i-u)^red|orange$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
});

regex_match!(costume_ness : Ness {
	RED: r"(?i-u)^default|red$",
	YELLOW: r"(?i-u)^yellow$",
	BLUE: r"(?i-u)^blue|purple$",
	GREEN: r"(?i-u)^green$",
});

regex_match!(costume_peach : Peach {
	RED: r"(?i-u)^default|red|pink$",
	YELLOW: r"(?i-u)^yellow|daisy$",
	WHITE: r"(?i-u)^white$",
	BLUE: r"(?i-u)^blue|purple$",
	GREEN: r"(?i-u)^green$",
});

regex_match!(costume_pikachu : Pikachu {
	YELLOW: r"(?i-u)^default|yellow$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
});

regex_match!(costume_iceclimbers : IceClimbers {
	BLUE: r"(?i-u)^default|blue|purple$",
	GREEN: r"(?i-u)^green$",
	ORANGE: r"(?i-u)^orange$",
	RED: r"(?i-u)^red$",
});

regex_match!(costume_jigglypuff : Jigglypuff {
	PINK: r"(?i-u)^default|pink$",
	RED: r"(?i-u)^red|flower$",
	BLUE: r"(?i-u)^blue|bow$",
	GREEN: r"(?i-u)^green|head[ _]?band$",
	YELLOW: r"(?i-u)^yellow|gold|crown$",
});

regex_match!(costume_samus : Samus {
	RED: r"(?i-u)^default|red|orange$",
	PINK: r"(?i-u)^pink$",
	BLACK: r"(?i-u)^black|brown$",
	GREEN: r"(?i-u)^green$",
	BLUE: r"(?i-u)^blue|purple$",
});

regex_match!(costume_yoshi : Yoshi {
	GREEN: r"(?i-u)^default|green$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^(dark[ _]?)?blue$",
	YELLOW: r"(?i-u)^yellow$",
	PINK: r"(?i-u)^pink$",
	CYAN: r"(?i-u)^cyan|light[ _]?blue$",
});

regex_match!(costume_zelda : Zelda {
	PINK: r"(?i-u)^default|pink$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
	WHITE: r"(?i-u)^white$",
});

regex_match!(costume_sheik : Sheik {
	NAVY: r"(?i-u)^default|navy$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
	WHITE: r"(?i-u)^white$",
});

regex_match!(costume_falco : Falco {
	TAN: r"(?i-u)^default|tan$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
});

regex_match!(costume_younglink : YoungLink {
	GREEN: r"(?i-u)^default|green$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	WHITE: r"(?i-u)^white$",
	BLACK: r"(?i-u)^black$",
});

regex_match!(costume_drmario : DrMario {
	WHITE: r"(?i-u)^default|white$",
	RED: r"(?i-u)^red|salmon|pink$",
	BLUE: r"(?i-u)^blue|purple$",
	GREEN: r"(?i-u)^green$",
	BLACK: r"(?i-u)^black$",
});

regex_match!(costume_roy : Roy {
	PURPLE: r"(?i-u)^default|purple$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
	YELLOW: r"(?i-u)^yellow|gold$",
});

regex_match!(costume_pichu : Pichu {
	YELLOW: r"(?i-u)^default|yellow$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
});

regex_match!(costume_ganondorf : Ganondorf {
	BROWN: r"(?i-u)^default|brown$",
	RED: r"(?i-u)^red$",
	BLUE: r"(?i-u)^blue$",
	GREEN: r"(?i-u)^green$",
	PURPLE: r"(?i-u)^purple$",
});
