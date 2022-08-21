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
