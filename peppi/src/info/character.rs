pub mod external {
    use crate::model::enums::character::External;

    #[non_exhaustive]
    #[derive(Clone, Debug)]
    pub struct Info {
        pub external: External,
        pub short: &'static str,
        pub long: &'static str,
    }

    info!(External => Info {
        CAPTAIN_FALCON {
            external: External::CAPTAIN_FALCON,
            short: "Falcon",
            long: "Captain Falcon",
        };

        DONKEY_KONG {
            external: External::DONKEY_KONG,
            short: "DK",
            long: "Donkey Kong",
        };

        FOX {
            external: External::FOX,
            short: "Fox",
            long: "Fox",
        };

        GAME_AND_WATCH {
            external: External::GAME_AND_WATCH,
            short: "G&W",
            long: "Game and Watch",
        };

        KIRBY {
            external: External::KIRBY,
            short: "Kirby",
            long: "Kirby",
        };

        BOWSER {
            external: External::BOWSER,
            short: "Bowser",
            long: "Bowser",
        };

        LINK {
            external: External::LINK,
            short: "Link",
            long: "Link",
        };

        LUIGI {
            external: External::LUIGI,
            short: "Luigi",
            long: "Luigi",
        };

        MARIO {
            external: External::MARIO,
            short: "Mario",
            long: "Mario",
        };

        MARTH {
            external: External::MARTH,
            short: "Marth",
            long: "Marth",
        };

        MEWTWO {
            external: External::MEWTWO,
            short: "Mewtwo",
            long: "Mewtwo",
        };

        NESS {
            external: External::NESS,
            short: "Ness",
            long: "Ness",
        };

        PEACH {
            external: External::PEACH,
            short: "Peach",
            long: "Peach",
        };

        PIKACHU {
            external: External::PIKACHU,
            short: "Pika",
            long: "Pikachu",
        };

        ICE_CLIMBERS {
            external: External::ICE_CLIMBERS,
            short: "ICs",
            long: "Ice Climbers",
        };

        JIGGLYPUFF {
            external: External::JIGGLYPUFF,
            short: "Puff",
            long: "Jigglypuff",
        };

        SAMUS {
            external: External::SAMUS,
            short: "Samus",
            long: "Samus",
        };

        YOSHI {
            external: External::YOSHI,
            short: "Yoshi",
            long: "Yoshi",
        };

        ZELDA {
            external: External::ZELDA,
            short: "Zelda",
            long: "Zelda",
        };

        SHEIK {
            external: External::SHEIK,
            short: "Sheik",
            long: "Sheik",
        };

        FALCO {
            external: External::FALCO,
            short: "Falco",
            long: "Falco",
        };

        YOUNG_LINK {
            external: External::YOUNG_LINK,
            short: "YL",
            long: "Young Link",
        };

        DR_MARIO {
            external: External::DR_MARIO,
            short: "Doc",
            long: "Dr. Mario",
        };

        ROY {
            external: External::ROY,
            short: "Roy",
            long: "Roy",
        };

        PICHU {
            external: External::PICHU,
            short: "Pichu",
            long: "Pichu",
        };

        GANONDORF {
            external: External::GANONDORF,
            short: "Ganon",
            long: "Ganondorf",
        };

        MASTER_HAND {
            external: External::MASTER_HAND,
            short: "Master Hand",
            long: "Master Hand",
        };

        WIRE_FRAME_MALE {
            external: External::WIRE_FRAME_MALE,
            short: "Male Wireframe",
            long: "Male Wireframe",
        };

        WIRE_FRAME_FEMALE {
            external: External::WIRE_FRAME_FEMALE,
            short: "Female Wireframe",
            long: "Female Wireframe",
        };

        GIGA_BOWSER {
            external: External::GIGA_BOWSER,
            short: "Giga Bowser",
            long: "Giga Bowser",
        };

        CRAZY_HAND {
            external: External::CRAZY_HAND,
            short: "Crazy Hand",
            long: "Crazy Hand",
        };

        SANDBAG {
            external: External::SANDBAG,
            short: "Sandbag",
            long: "Sandbag",
        };

        POPO {
            external: External::POPO,
            short: "Popo",
            long: "Popo",
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
