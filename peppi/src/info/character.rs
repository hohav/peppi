mod external {
    use crate::model::enums::character::External;

    #[non_exhaustive]
    #[derive(Clone, Debug)]
    pub struct Info {
        pub short_name: &'static str,
        pub long_name: &'static str,
    }

    info!(External => Info {
        CAPTAIN_FALCON {
            short_name: "Falcon",
            long_name: "Captain Falcon",
        };

        DONKEY_KONG {
            short_name: "DK",
            long_name: "Donkey Kong",
        };

        FOX {
            short_name: "Fox",
            long_name: "Fox",
        };

        GAME_AND_WATCH {
            short_name: "G&W",
            long_name: "Game and Watch",
        };

        KIRBY {
            short_name: "Kirby",
            long_name: "Kirby",
        };

        BOWSER {
            short_name: "Bowser",
            long_name: "Bowser",
        };

        LINK {
            short_name: "Link",
            long_name: "Link",
        };

        LUIGI {
            short_name: "Luigi",
            long_name: "Luigi",
        };

        MARIO {
            short_name: "Mario",
            long_name: "Mario",
        };

        MARTH {
            short_name: "Marth",
            long_name: "Marth",
        };

        MEWTWO {
            short_name: "Mewtwo",
            long_name: "Mewtwo",
        };

        NESS {
            short_name: "Ness",
            long_name: "Ness",
        };

        PEACH {
            short_name: "Peach",
            long_name: "Peach",
        };

        PIKACHU {
            short_name: "Pika",
            long_name: "Pikachu",
        };

        ICE_CLIMBERS {
            short_name: "ICs",
            long_name: "Ice Climbers",
        };

        JIGGLYPUFF {
            short_name: "Puff",
            long_name: "Jigglypuff",
        };

        SAMUS {
            short_name: "Samus",
            long_name: "Samus",
        };

        YOSHI {
            short_name: "Yoshi",
            long_name: "Yoshi",
        };

        ZELDA {
            short_name: "Zelda",
            long_name: "Zelda",
        };

        SHEIK {
            short_name: "Sheik",
            long_name: "Sheik",
        };

        FALCO {
            short_name: "Falco",
            long_name: "Falco",
        };

        YOUNG_LINK {
            short_name: "YL",
            long_name: "Young Link",
        };

        DR_MARIO {
            short_name: "Doc",
            long_name: "Dr. Mario",
        };

        ROY {
            short_name: "Roy",
            long_name: "Roy",
        };

        PICHU {
            short_name: "Pichu",
            long_name: "Pichu",
        };

        GANONDORF {
            short_name: "Ganon",
            long_name: "Ganondorf",
        };

        MASTER_HAND {
            short_name: "Master Hand",
            long_name: "Master Hand",
        };

        WIRE_FRAME_MALE {
            short_name: "Male Wireframe",
            long_name: "Male Wireframe",
        };

        WIRE_FRAME_FEMALE {
            short_name: "Female Wireframe",
            long_name: "Female Wireframe",
        };

        GIGA_BOWSER {
            short_name: "Giga Bowser",
            long_name: "Giga Bowser",
        };

        CRAZY_HAND {
            short_name: "Crazy Hand",
            long_name: "Crazy Hand",
        };

        SANDBAG {
            short_name: "Sandbag",
            long_name: "Sandbag",
        };

        POPO {
            short_name: "Popo",
            long_name: "Popo",
        };
    });
}

mod internal {
    use crate::model::enums::character::Internal;
    #[non_exhaustive]
    #[derive(Clone, Debug)]
    pub struct Info {
        pub jumpsquat: u8,
        pub empty_landing_lag: u8,
        pub can_walljump: bool,
    }

    info!(Internal => Info {
        MARIO {
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: true,
        };

        FOX {
            jumpsquat: 3,
            empty_landing_lag: 4,
            can_walljump: true,
        };

        CAPTAIN_FALCON {
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: true,
        };

        DONKEY_KONG {
            jumpsquat: 5,
            empty_landing_lag: 5,
            can_walljump: false,
        };

        KIRBY {
            jumpsquat: 3,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        BOWSER {
            jumpsquat: 8,
            empty_landing_lag: 6,
            can_walljump: false,
        };

        LINK {
            jumpsquat: 6,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        SHEIK {
            jumpsquat: 3,
            empty_landing_lag: 4,
            can_walljump: true,
        };

        NESS {
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        PEACH {
            jumpsquat: 5,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        POPO {
            jumpsquat: 3,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        NANA {
            jumpsquat: 3,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        PIKACHU {
            jumpsquat: 3,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        SAMUS {
            jumpsquat: 3,
            empty_landing_lag: 4,
            can_walljump: true,
        };

        YOSHI {
            jumpsquat: 5,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        JIGGLYPUFF {
            jumpsquat: 5,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        MEWTWO {
            jumpsquat: 5,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        LUIGI {
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        MARTH {
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        ZELDA {
            jumpsquat: 6,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        YOUNG_LINK {
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: true,
        };

        DR_MARIO {
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        FALCO {
            jumpsquat: 5,
            empty_landing_lag: 4,
            can_walljump: true,
        };

        PICHU {
            jumpsquat: 3,
            empty_landing_lag: 2,
            can_walljump: true,
        };

        GAME_AND_WATCH {
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        GANONDORF {
            jumpsquat: 6,
            empty_landing_lag: 5,
            can_walljump: false,
        };

        ROY {
            jumpsquat: 5,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        WIRE_FRAME_MALE {
            jumpsquat: 7,
            empty_landing_lag: 15,
            can_walljump: false,
        };

        WIRE_FRAME_FEMALE {
            jumpsquat: 7,
            empty_landing_lag: 15,
            can_walljump: false,
        };

        GIGA_BOWSER {
            jumpsquat: 6,
            empty_landing_lag: 30,
            can_walljump: false,
        };
    });
}
