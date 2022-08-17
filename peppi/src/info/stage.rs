use crate::model::enums::stage::Stage;

#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Info {
    pub stage: Stage,
    pub short: &'static str,
    pub long: &'static str,
}

info!(Stage => Info {
	FOUNTAIN_OF_DREAMS {
        stage: Stage::FOUNTAIN_OF_DREAMS,
        short: "FoD",
        long: "Fountain of Dreams",
    };

	POKEMON_STADIUM {
        stage: Stage::POKEMON_STADIUM,
        short: "PS",
        long: "Pokemon Stadium",
    };

	PRINCESS_PEACHS_CASTLE {
        stage: Stage::PRINCESS_PEACHS_CASTLE,
        short: "PPC",
        long: "Princess Peach's Castle",
    };

	KONGO_JUNGLE {
        stage: Stage::KONGO_JUNGLE,
        short: "KJ",
        long: "Kongo Jungle",
    };

	BRINSTAR {
        stage: Stage::BRINSTAR,
        short: "Brinstar",
        long: "Brinstar",
    };

	CORNERIA {
        stage: Stage::CORNERIA,
        short: "Corneria",
        long: "Corneria",
    };

	YOSHIS_STORY {
        stage: Stage::YOSHIS_STORY,
        short: "YS",
        long: "Yoshi's Story",
    };

	ONETT {
        stage: Stage::ONETT,
        short: "Onett",
        long: "Onett",
    };

	MUTE_CITY {
        stage: Stage::MUTE_CITY,
        short: "MC",
        long: "Mute City",
    };

	RAINBOW_CRUISE {
        stage: Stage::RAINBOW_CRUISE,
        short: "RC",
        long: "Rainbow Cruise",
    };

	JUNGLE_JAPES {
        stage: Stage::JUNGLE_JAPES,
        short: "JJ",
        long: "Jungle Japes",
    };

	GREAT_BAY {
        stage: Stage::GREAT_BAY,
        short: "GB",
        long: "Great Bay",
    };

	HYRULE_TEMPLE {
        stage: Stage::HYRULE_TEMPLE,
        short: "HT",
        long: "Hyrule Temple",
    };

	BRINSTAR_DEPTHS {
        stage: Stage::BRINSTAR_DEPTHS,
        short: "BD",
        long: "Brinstar Depths",
    };

	YOSHIS_ISLAND {
        stage: Stage::YOSHIS_ISLAND,
        short: "YI",
        long: "Yoshi's Island",
    };

	GREEN_GREENS {
        stage: Stage::GREEN_GREENS,
        short: "GG",
        long: "Green Greens",
    };

	FOURSIDE {
        stage: Stage::FOURSIDE,
        short: "Fourside",
        long: "Fourside",
    };

	MUSHROOM_KINGDOM_I {
        stage: Stage::MUSHROOM_KINGDOM_I,
        short: "MKI",
        long: "Mushroom Kingdom I",
    };

	MUSHROOM_KINGDOM_II {
        stage: Stage::MUSHROOM_KINGDOM_II,
        short: "MKII",
        long: "Mushroom Kingdom II",
    };

	VENOM {
        stage: Stage::VENOM,
        short: "Venom",
        long: "Venom",
    };

	POKE_FLOATS {
        stage: Stage::POKE_FLOATS,
        short: "PF",
        long: "Poke Floats",
    };

	BIG_BLUE {
        stage: Stage::BIG_BLUE,
        short: "BB",
        long: "Big Blue",
    };

	ICICLE_MOUNTAIN {
        stage: Stage::ICICLE_MOUNTAIN,
        short: "IM",
        long: "Icicle Mountain",
    };

    // No ICETOP (unplayable in melee without hacks)

	FLAT_ZONE {
        stage: Stage::FLAT_ZONE,
        short: "FZ",
        long: "Flat Zone",
    };

	DREAM_LAND_N64 {
        stage: Stage::DREAM_LAND_N64,
        short: "DL64",
        long: "Dream Land N64",
    };

	YOSHIS_ISLAND_N64 {
        stage: Stage::YOSHIS_ISLAND_N64,
        short: "YI64",
        long: "Yoshi's Island N64",
    };

	KONGO_JUNGLE_N64 {
        stage: Stage::KONGO_JUNGLE_N64,
        short: "KJ64",
        long: "Kongo Jungle N64",
    };

	BATTLEFIELD {
        stage: Stage::BATTLEFIELD,
        short: "BF",
        long: "Battlefield",
    };

	FINAL_DESTINATION {
        stage: Stage::FINAL_DESTINATION,
        short: "FD",
        long: "Final Destination",
    };
});

info_regex!(Info {
    // Common stages at top to save regex time in common case
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
