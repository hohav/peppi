pub mod external {
    use crate::model::enums::character::External;

    #[non_exhaustive]
    #[derive(Clone, Debug)]
    pub struct Info {
        pub external: External,
        pub short_name: &'static str,
        pub long_name: &'static str,
    }

    info!(External => Info {
        CAPTAIN_FALCON {
            external: External::CAPTAIN_FALCON,
            short_name: "Falcon",
            long_name: "Captain Falcon",
        };

        DONKEY_KONG {
            external: External::DONKEY_KONG,
            short_name: "DK",
            long_name: "Donkey Kong",
        };

        FOX {
            external: External::FOX,
            short_name: "Fox",
            long_name: "Fox",
        };

        GAME_AND_WATCH {
            external: External::GAME_AND_WATCH,
            short_name: "G&W",
            long_name: "Game and Watch",
        };

        KIRBY {
            external: External::KIRBY,
            short_name: "Kirby",
            long_name: "Kirby",
        };

        BOWSER {
            external: External::BOWSER,
            short_name: "Bowser",
            long_name: "Bowser",
        };

        LINK {
            external: External::LINK,
            short_name: "Link",
            long_name: "Link",
        };

        LUIGI {
            external: External::LUIGI,
            short_name: "Luigi",
            long_name: "Luigi",
        };

        MARIO {
            external: External::MARIO,
            short_name: "Mario",
            long_name: "Mario",
        };

        MARTH {
            external: External::MARTH,
            short_name: "Marth",
            long_name: "Marth",
        };

        MEWTWO {
            external: External::MEWTWO,
            short_name: "Mewtwo",
            long_name: "Mewtwo",
        };

        NESS {
            external: External::NESS,
            short_name: "Ness",
            long_name: "Ness",
        };

        PEACH {
            external: External::PEACH,
            short_name: "Peach",
            long_name: "Peach",
        };

        PIKACHU {
            external: External::PIKACHU,
            short_name: "Pika",
            long_name: "Pikachu",
        };

        ICE_CLIMBERS {
            external: External::ICE_CLIMBERS,
            short_name: "ICs",
            long_name: "Ice Climbers",
        };

        JIGGLYPUFF {
            external: External::JIGGLYPUFF,
            short_name: "Puff",
            long_name: "Jigglypuff",
        };

        SAMUS {
            external: External::SAMUS,
            short_name: "Samus",
            long_name: "Samus",
        };

        YOSHI {
            external: External::YOSHI,
            short_name: "Yoshi",
            long_name: "Yoshi",
        };

        ZELDA {
            external: External::ZELDA,
            short_name: "Zelda",
            long_name: "Zelda",
        };

        SHEIK {
            external: External::SHEIK,
            short_name: "Sheik",
            long_name: "Sheik",
        };

        FALCO {
            external: External::FALCO,
            short_name: "Falco",
            long_name: "Falco",
        };

        YOUNG_LINK {
            external: External::YOUNG_LINK,
            short_name: "YL",
            long_name: "Young Link",
        };

        DR_MARIO {
            external: External::DR_MARIO,
            short_name: "Doc",
            long_name: "Dr. Mario",
        };

        ROY {
            external: External::ROY,
            short_name: "Roy",
            long_name: "Roy",
        };

        PICHU {
            external: External::PICHU,
            short_name: "Pichu",
            long_name: "Pichu",
        };

        GANONDORF {
            external: External::GANONDORF,
            short_name: "Ganon",
            long_name: "Ganondorf",
        };

        MASTER_HAND {
            external: External::MASTER_HAND,
            short_name: "Master Hand",
            long_name: "Master Hand",
        };

        WIRE_FRAME_MALE {
            external: External::WIRE_FRAME_MALE,
            short_name: "Male Wireframe",
            long_name: "Male Wireframe",
        };

        WIRE_FRAME_FEMALE {
            external: External::WIRE_FRAME_FEMALE,
            short_name: "Female Wireframe",
            long_name: "Female Wireframe",
        };

        GIGA_BOWSER {
            external: External::GIGA_BOWSER,
            short_name: "Giga Bowser",
            long_name: "Giga Bowser",
        };

        CRAZY_HAND {
            external: External::CRAZY_HAND,
            short_name: "Crazy Hand",
            long_name: "Crazy Hand",
        };

        SANDBAG {
            external: External::SANDBAG,
            short_name: "Sandbag",
            long_name: "Sandbag",
        };

        POPO {
            external: External::POPO,
            short_name: "Popo",
            long_name: "Popo",
        };
    });
}

pub mod internal {
    use crate::model::enums::character::Internal;

    #[non_exhaustive]
    #[derive(Clone, Debug)]
    pub struct Info {
        pub internal: Internal,
        pub jumpsquat: u8,
        pub empty_landing_lag: u8,
        pub can_walljump: bool,
    }

    info!(Internal => Info {
        MARIO {
            internal: Internal::MARIO,
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: true,
        };

        FOX {
            internal: Internal::FOX,
            jumpsquat: 3,
            empty_landing_lag: 4,
            can_walljump: true,
        };

        CAPTAIN_FALCON {
            internal: Internal::CAPTAIN_FALCON,
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: true,
        };

        DONKEY_KONG {
            internal: Internal::DONKEY_KONG,
            jumpsquat: 5,
            empty_landing_lag: 5,
            can_walljump: false,
        };

        KIRBY {
            internal: Internal::KIRBY,
            jumpsquat: 3,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        BOWSER {
            internal: Internal::BOWSER,
            jumpsquat: 8,
            empty_landing_lag: 6,
            can_walljump: false,
        };

        LINK {
            internal: Internal::LINK,
            jumpsquat: 6,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        SHEIK {
            internal: Internal::SHEIK,
            jumpsquat: 3,
            empty_landing_lag: 4,
            can_walljump: true,
        };

        NESS {
            internal: Internal::NESS,
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        PEACH {
            internal: Internal::PEACH,
            jumpsquat: 5,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        POPO {
            internal: Internal::POPO,
            jumpsquat: 3,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        NANA {
            internal: Internal::NANA,
            jumpsquat: 3,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        PIKACHU {
            internal: Internal::PIKACHU,
            jumpsquat: 3,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        SAMUS {
            internal: Internal::SAMUS,
            jumpsquat: 3,
            empty_landing_lag: 4,
            can_walljump: true,
        };

        YOSHI {
            internal: Internal::YOSHI,
            jumpsquat: 5,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        JIGGLYPUFF {
            internal: Internal::JIGGLYPUFF,
            jumpsquat: 5,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        MEWTWO {
            internal: Internal::MEWTWO,
            jumpsquat: 5,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        LUIGI {
            internal: Internal::LUIGI,
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        MARTH {
            internal: Internal::MARTH,
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        ZELDA {
            internal: Internal::ZELDA,
            jumpsquat: 6,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        YOUNG_LINK {
            internal: Internal::YOUNG_LINK,
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: true,
        };

        DR_MARIO {
            internal: Internal::DR_MARIO,
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        FALCO {
            internal: Internal::FALCO,
            jumpsquat: 5,
            empty_landing_lag: 4,
            can_walljump: true,
        };

        PICHU {
            internal: Internal::PICHU,
            jumpsquat: 3,
            empty_landing_lag: 2,
            can_walljump: true,
        };

        GAME_AND_WATCH {
            internal: Internal::GAME_AND_WATCH,
            jumpsquat: 4,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        GANONDORF {
            internal: Internal::GANONDORF,
            jumpsquat: 6,
            empty_landing_lag: 5,
            can_walljump: false,
        };

        ROY {
            internal: Internal::ROY,
            jumpsquat: 5,
            empty_landing_lag: 4,
            can_walljump: false,
        };

        WIRE_FRAME_MALE {
            internal: Internal::WIRE_FRAME_MALE,
            jumpsquat: 7,
            empty_landing_lag: 15,
            can_walljump: false,
        };

        WIRE_FRAME_FEMALE {
            internal: Internal::WIRE_FRAME_FEMALE,
            jumpsquat: 7,
            empty_landing_lag: 15,
            can_walljump: false,
        };

        GIGA_BOWSER {
            internal: Internal::GIGA_BOWSER,
            jumpsquat: 6,
            empty_landing_lag: 30,
            can_walljump: false,
        };
    });
}
