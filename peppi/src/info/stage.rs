use crate::model::enums::stage::Stage;

#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Info {
    pub short_name: &'static str,
    pub long_name: &'static str,
}

info!(Stage => Info {
	FOUNTAIN_OF_DREAMS {
        short_name: "FoD",
        long_name: "Fountain of Dreams",
    };

	POKEMON_STADIUM {
        short_name: "PS",
        long_name: "Pokemon Stadium",
    };

	PRINCESS_PEACHS_CASTLE {
        short_name: "PPC",
        long_name: "Princess Peach's Castle",
    };

	KONGO_JUNGLE {
        short_name: "KJ",
        long_name: "Kongo Jungle",
    };

	BRINSTAR {
        short_name: "Brinstar",
        long_name: "Brinstar",
    };

	CORNERIA {
        short_name: "Corneria",
        long_name: "Corneria",
    };

	YOSHIS_STORY {
        short_name: "YS",
        long_name: "Yoshi's Story",
    };

	ONETT {
        short_name: "Onett",
        long_name: "Onett",
    };

	MUTE_CITY {
        short_name: "MC",
        long_name: "Mute City",
    };

	RAINBOW_CRUISE {
        short_name: "RC",
        long_name: "Rainbow Cruise",
    };

	JUNGLE_JAPES {
        short_name: "JJ",
        long_name: "Jungle Japes",
    };

	GREAT_BAY {
        short_name: "GB",
        long_name: "Great Bay",
    };

	HYRULE_TEMPLE {
        short_name: "HT",
        long_name: "Hyrule Temple",
    };

	BRINSTAR_DEPTHS {
        short_name: "BD",
        long_name: "Brinstar Depths",
    };

	YOSHIS_ISLAND {
        short_name: "YI",
        long_name: "Yoshi's Island",
    };

	GREEN_GREENS {
        short_name: "GG",
        long_name: "Green Greens",
    };

	FOURSIDE {
        short_name: "Fourside",
        long_name: "Fourside",
    };

	MUSHROOM_KINGDOM_I {
        short_name: "MKI",
        long_name: "Mushroom Kingdom I",
    };

	MUSHROOM_KINGDOM_II {
        short_name: "MKII",
        long_name: "Mushroom Kingdom II",
    };

	VENOM {
        short_name: "Venom",
        long_name: "Venom",
    };

	POKE_FLOATS {
        short_name: "PF",
        long_name: "Poke Floats",
    };

	BIG_BLUE {
        short_name: "BB",
        long_name: "Big Blue",
    };

	ICICLE_MOUNTAIN {
        short_name: "IM",
        long_name: "Icicle Mountain",
    };

    // No ICETOP (unplayable in melee without hacks)

	FLAT_ZONE {
        short_name: "FZ",
        long_name: "Flat Zone",
    };

	DREAM_LAND_N64 {
        short_name: "DL64",
        long_name: "Dream Land N64",
    };

	YOSHIS_ISLAND_N64 {
        short_name: "YI64",
        long_name: "Yoshi's Island N64",
    };

	KONGO_JUNGLE_N64 {
        short_name: "KJ64",
        long_name: "Kongo Jungle N64",
    };

	BATTLEFIELD {
        short_name: "BF",
        long_name: "Battlefield",
    };

	FINAL_DESTINATION {
        short_name: "FD",
        long_name: "Final Destination",
    };
});
