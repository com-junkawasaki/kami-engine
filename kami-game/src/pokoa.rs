//! Pokoa (ぽこあポケモン): Brainrot × Pokemon battle system.
//!
//! Brainrot meme characters reimagined as collectible battle creatures.
//! Turn-based combat with type effectiveness, capture mechanics, and evolution.

use crate::common::SimpleRng;
use std::collections::HashMap;

// =============================================================================
// Types
// =============================================================================

/// Pokoa elemental types (18 types, Pokemon-compatible).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PokoaType {
    Normal,
    Fire,
    Water,
    Electric,
    Grass,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
}

impl PokoaType {
    /// Type effectiveness multiplier: attacker type vs defender type.
    pub fn effectiveness(atk: PokoaType, def: PokoaType) -> f32 {
        use PokoaType::*;
        match (atk, def) {
            // Super effective (2.0)
            (Fire, Grass) | (Fire, Ice) | (Fire, Bug) | (Fire, Steel) => 2.0,
            (Water, Fire) | (Water, Ground) | (Water, Rock) => 2.0,
            (Electric, Water) | (Electric, Flying) => 2.0,
            (Grass, Water) | (Grass, Ground) | (Grass, Rock) => 2.0,
            (Ice, Grass) | (Ice, Ground) | (Ice, Flying) | (Ice, Dragon) => 2.0,
            (Fighting, Normal)
            | (Fighting, Ice)
            | (Fighting, Rock)
            | (Fighting, Dark)
            | (Fighting, Steel) => 2.0,
            (Poison, Grass) | (Poison, Fairy) => 2.0,
            (Ground, Fire)
            | (Ground, Electric)
            | (Ground, Poison)
            | (Ground, Rock)
            | (Ground, Steel) => 2.0,
            (Flying, Grass) | (Flying, Fighting) | (Flying, Bug) => 2.0,
            (Psychic, Fighting) | (Psychic, Poison) => 2.0,
            (Bug, Grass) | (Bug, Psychic) | (Bug, Dark) => 2.0,
            (Rock, Fire) | (Rock, Ice) | (Rock, Flying) | (Rock, Bug) => 2.0,
            (Ghost, Psychic) | (Ghost, Ghost) => 2.0,
            (Dragon, Dragon) => 2.0,
            (Dark, Psychic) | (Dark, Ghost) => 2.0,
            (Steel, Ice) | (Steel, Rock) | (Steel, Fairy) => 2.0,
            (Fairy, Fighting) | (Fairy, Dragon) | (Fairy, Dark) => 2.0,
            // Not very effective (0.5)
            (Fire, Fire) | (Fire, Water) | (Fire, Rock) | (Fire, Dragon) => 0.5,
            (Water, Water) | (Water, Grass) | (Water, Dragon) => 0.5,
            (Electric, Electric) | (Electric, Grass) | (Electric, Dragon) => 0.5,
            (Grass, Fire)
            | (Grass, Grass)
            | (Grass, Poison)
            | (Grass, Flying)
            | (Grass, Bug)
            | (Grass, Dragon)
            | (Grass, Steel) => 0.5,
            (Ice, Fire) | (Ice, Water) | (Ice, Ice) | (Ice, Steel) => 0.5,
            (Fighting, Poison)
            | (Fighting, Flying)
            | (Fighting, Psychic)
            | (Fighting, Bug)
            | (Fighting, Fairy) => 0.5,
            (Poison, Poison) | (Poison, Ground) | (Poison, Rock) | (Poison, Ghost) => 0.5,
            (Ground, Grass) | (Ground, Bug) => 0.5,
            (Flying, Electric) | (Flying, Rock) | (Flying, Steel) => 0.5,
            (Psychic, Psychic) | (Psychic, Steel) => 0.5,
            (Bug, Fire)
            | (Bug, Fighting)
            | (Bug, Poison)
            | (Bug, Flying)
            | (Bug, Ghost)
            | (Bug, Steel)
            | (Bug, Fairy) => 0.5,
            (Rock, Fighting) | (Rock, Ground) | (Rock, Steel) => 0.5,
            (Ghost, Dark) => 0.5,
            (Dark, Fighting) | (Dark, Dark) | (Dark, Fairy) => 0.5,
            (Steel, Fire) | (Steel, Water) | (Steel, Electric) | (Steel, Steel) => 0.5,
            (Fairy, Fire) | (Fairy, Poison) | (Fairy, Steel) => 0.5,
            // Immune (0.0)
            (Normal, Ghost) | (Ghost, Normal) => 0.0,
            (Electric, Ground) => 0.0,
            (Fighting, Ghost) => 0.0,
            (Poison, Steel) => 0.0,
            (Ground, Flying) => 0.0,
            (Psychic, Dark) => 0.0,
            (Dragon, Fairy) => 0.0,
            // Normal (1.0)
            _ => 1.0,
        }
    }

    /// Calculate combined effectiveness against dual-type defender.
    pub fn calc_effectiveness(atk: PokoaType, def: (PokoaType, Option<PokoaType>)) -> f32 {
        let m1 = Self::effectiveness(atk, def.0);
        match def.1 {
            Some(t2) => m1 * Self::effectiveness(atk, t2),
            None => m1,
        }
    }
}

// =============================================================================
// Stats
// =============================================================================

#[derive(Debug, Clone, Copy, Default)]
pub struct Stats {
    pub hp: u16,
    pub atk: u16,
    pub def: u16,
    pub spa: u16,
    pub spd: u16,
    pub spe: u16,
}

impl Stats {
    pub fn total(&self) -> u16 {
        self.hp + self.atk + self.def + self.spa + self.spd + self.spe
    }
}

/// Nature modifies two stats (+10%/-10%).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nature {
    Hardy,
    Lonely,
    Brave,
    Adamant,
    Naughty,
    Bold,
    Docile,
    Relaxed,
    Impish,
    Lax,
    Timid,
    Hasty,
    Serious,
    Jolly,
    Naive,
    Modest,
    Mild,
    Quiet,
    Bashful,
    Rash,
    Calm,
    Gentle,
    Sassy,
    Careful,
    Quirky,
}

impl Nature {
    /// Returns (boosted_stat_index, lowered_stat_index). 0=atk,1=def,2=spa,3=spd,4=spe.
    /// Neutral natures return None.
    pub fn modifiers(&self) -> Option<(usize, usize)> {
        use Nature::*;
        match self {
            Lonely => Some((0, 1)),
            Brave => Some((0, 4)),
            Adamant => Some((0, 2)),
            Naughty => Some((0, 3)),
            Bold => Some((1, 0)),
            Relaxed => Some((1, 4)),
            Impish => Some((1, 2)),
            Lax => Some((1, 3)),
            Timid => Some((4, 0)),
            Hasty => Some((4, 1)),
            Jolly => Some((4, 2)),
            Naive => Some((4, 3)),
            Modest => Some((2, 0)),
            Mild => Some((2, 1)),
            Quiet => Some((2, 4)),
            Rash => Some((2, 3)),
            Calm => Some((3, 0)),
            Gentle => Some((3, 1)),
            Sassy => Some((3, 4)),
            Careful => Some((3, 2)),
            _ => None, // Hardy, Docile, Serious, Bashful, Quirky
        }
    }
}

// =============================================================================
// Moves
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

#[derive(Debug, Clone)]
pub struct MoveDef {
    pub id: &'static str,
    pub name: &'static str,
    pub pokoa_type: PokoaType,
    pub category: MoveCategory,
    pub power: u16,
    pub accuracy: u8,
    pub pp: u8,
}

#[derive(Debug, Clone)]
pub struct MoveSlot {
    pub def: MoveDef,
    pub pp_remaining: u8,
}

impl MoveSlot {
    pub fn new(def: MoveDef) -> Self {
        let pp = def.pp;
        Self {
            def,
            pp_remaining: pp,
        }
    }

    pub fn can_use(&self) -> bool {
        self.pp_remaining > 0
    }

    pub fn use_pp(&mut self) {
        if self.pp_remaining > 0 {
            self.pp_remaining -= 1;
        }
    }
}

// =============================================================================
// Species (Pokoa Dex)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvolutionTrigger {
    Level(u8),
    Item(&'static str),
}

#[derive(Debug, Clone)]
pub struct SpeciesDef {
    pub id: u16,
    pub name: &'static str,
    pub types: (PokoaType, Option<PokoaType>),
    pub base_stats: Stats,
    pub catch_rate: u8,
    pub exp_yield: u16,
    pub evolves_to: Option<(u16, EvolutionTrigger)>,
    pub learnable_moves: Vec<(u8, &'static str)>, // (level, move_id)
    pub description: &'static str,
}

/// 12 Brainrot Pokoa species (4 evolution lines + 2 legendaries).
pub fn pokoa_dex() -> Vec<SpeciesDef> {
    vec![
        // --- Line 1: Toilettle → Skibidrain → MegaSkibidi ---
        SpeciesDef {
            id: 1,
            name: "Toilettle",
            types: (PokoaType::Water, Some(PokoaType::Dark)),
            base_stats: Stats {
                hp: 44,
                atk: 48,
                def: 65,
                spa: 50,
                spd: 64,
                spe: 43,
            },
            catch_rate: 45,
            exp_yield: 63,
            evolves_to: Some((2, EvolutionTrigger::Level(16))),
            learnable_moves: vec![
                (1, "splash"),
                (1, "leer"),
                (5, "flush-cannon"),
                (9, "dark-pulse"),
                (13, "aqua-jet"),
            ],
            description: "A tiny toilet creature. Makes 'dop dop' sounds when happy.",
        },
        SpeciesDef {
            id: 2,
            name: "Skibidrain",
            types: (PokoaType::Water, Some(PokoaType::Dark)),
            base_stats: Stats {
                hp: 59,
                atk: 63,
                def: 80,
                spa: 65,
                spd: 80,
                spe: 58,
            },
            catch_rate: 45,
            exp_yield: 142,
            evolves_to: Some((3, EvolutionTrigger::Level(36))),
            learnable_moves: vec![
                (1, "flush-cannon"),
                (1, "dark-pulse"),
                (16, "plumber-slam"),
                (22, "sewage-wave"),
                (28, "yes-yes-beam"),
            ],
            description: "Rises from toilets to ambush prey. Head rotates 360 degrees.",
        },
        SpeciesDef {
            id: 3,
            name: "MegaSkibidi",
            types: (PokoaType::Water, Some(PokoaType::Dark)),
            base_stats: Stats {
                hp: 79,
                atk: 83,
                def: 100,
                spa: 85,
                spd: 105,
                spe: 78,
            },
            catch_rate: 45,
            exp_yield: 236,
            evolves_to: None,
            learnable_moves: vec![
                (1, "yes-yes-beam"),
                (36, "hydro-pump"),
                (42, "dop-dop-cannon"),
                (50, "mega-flush"),
            ],
            description: "The ultimate Skibidi boss. Its 'dop dop yes yes' cry terrifies opponents.",
        },
        // --- Line 2: Sigpup → Sigmachu → Gigachad ---
        SpeciesDef {
            id: 4,
            name: "Sigpup",
            types: (PokoaType::Electric, Some(PokoaType::Fighting)),
            base_stats: Stats {
                hp: 35,
                atk: 55,
                def: 40,
                spa: 50,
                spd: 50,
                spe: 90,
            },
            catch_rate: 190,
            exp_yield: 112,
            evolves_to: Some((5, EvolutionTrigger::Level(20))),
            learnable_moves: vec![
                (1, "thunder-shock"),
                (1, "leer"),
                (5, "sigma-stare"),
                (10, "quick-attack"),
                (15, "grindset-punch"),
            ],
            description: "A lone wolf pup. Refuses to follow the pack.",
        },
        SpeciesDef {
            id: 5,
            name: "Sigmachu",
            types: (PokoaType::Electric, Some(PokoaType::Fighting)),
            base_stats: Stats {
                hp: 60,
                atk: 90,
                def: 55,
                spa: 90,
                spd: 80,
                spe: 110,
            },
            catch_rate: 75,
            exp_yield: 218,
            evolves_to: Some((6, EvolutionTrigger::Item("protein-shake"))),
            learnable_moves: vec![
                (1, "grindset-punch"),
                (20, "thunderbolt"),
                (28, "bulk-up"),
                (36, "sigma-barrage"),
            ],
            description: "Trains alone at the gym. Its electric punches never miss leg day.",
        },
        SpeciesDef {
            id: 6,
            name: "Gigachad",
            types: (PokoaType::Electric, Some(PokoaType::Fighting)),
            base_stats: Stats {
                hp: 75,
                atk: 130,
                def: 70,
                spa: 95,
                spd: 85,
                spe: 120,
            },
            catch_rate: 45,
            exp_yield: 270,
            evolves_to: None,
            learnable_moves: vec![
                (1, "sigma-barrage"),
                (1, "thunderbolt"),
                (42, "gigachad-flex"),
                (50, "thunder"),
            ],
            description: "The ultimate sigma male. Its jawline alone can deflect attacks.",
        },
        // --- Line 3: Ohiolet → Ohiodon ---
        SpeciesDef {
            id: 7,
            name: "Ohiolet",
            types: (PokoaType::Dark, Some(PokoaType::Ghost)),
            base_stats: Stats {
                hp: 45,
                atk: 65,
                def: 40,
                spa: 80,
                spd: 40,
                spe: 68,
            },
            catch_rate: 120,
            exp_yield: 87,
            evolves_to: Some((8, EvolutionTrigger::Level(28))),
            learnable_moves: vec![
                (1, "shadow-sneak"),
                (1, "confusion"),
                (7, "ohio-glitch"),
                (14, "teleport-strike"),
            ],
            description: "'Only in Ohio' — this creature IS the anomaly.",
        },
        SpeciesDef {
            id: 8,
            name: "Ohiodon",
            types: (PokoaType::Dark, Some(PokoaType::Ghost)),
            base_stats: Stats {
                hp: 65,
                atk: 95,
                def: 60,
                spa: 120,
                spd: 60,
                spe: 98,
            },
            catch_rate: 45,
            exp_yield: 227,
            evolves_to: None,
            learnable_moves: vec![
                (1, "ohio-glitch"),
                (28, "shadow-ball"),
                (35, "reality-warp"),
                (42, "ohio-final-form"),
            ],
            description: "The Ohio Final Boss. Teleports through dimensions at will.",
        },
        // --- Line 4: Grimini → Grimaceon ---
        SpeciesDef {
            id: 9,
            name: "Grimini",
            types: (PokoaType::Poison, Some(PokoaType::Fairy)),
            base_stats: Stats {
                hp: 70,
                atk: 45,
                def: 50,
                spa: 65,
                spd: 65,
                spe: 40,
            },
            catch_rate: 120,
            exp_yield: 66,
            evolves_to: Some((10, EvolutionTrigger::Level(25))),
            learnable_moves: vec![
                (1, "absorb"),
                (1, "growl"),
                (6, "purple-shake"),
                (12, "sludge"),
                (18, "moonblast"),
            ],
            description: "A cute purple blob. Don't drink its shake.",
        },
        SpeciesDef {
            id: 10,
            name: "Grimaceon",
            types: (PokoaType::Poison, Some(PokoaType::Fairy)),
            base_stats: Stats {
                hp: 130,
                atk: 65,
                def: 60,
                spa: 110,
                spd: 95,
                spe: 30,
            },
            catch_rate: 45,
            exp_yield: 230,
            evolves_to: None,
            learnable_moves: vec![
                (1, "purple-shake"),
                (25, "sludge-bomb"),
                (32, "dazzling-gleam"),
                (40, "grimace-shake-doom"),
            ],
            description: "It IS the Grimace Shake. Area-denial specialist with toxic puddles.",
        },
        // --- Legendary: Rizzlord ---
        SpeciesDef {
            id: 11,
            name: "Rizzlord",
            types: (PokoaType::Fire, Some(PokoaType::Psychic)),
            base_stats: Stats {
                hp: 100,
                atk: 100,
                def: 100,
                spa: 130,
                spd: 100,
                spe: 100,
            },
            catch_rate: 3,
            exp_yield: 306,
            evolves_to: None,
            learnable_moves: vec![
                (1, "flamethrower"),
                (1, "psychic"),
                (50, "rizz-beam"),
                (65, "infinite-rizz"),
            ],
            description: "Legendary W rizz incarnate. Its charm transcends type matchups.",
        },
        // --- Legendary: Fanumoth ---
        SpeciesDef {
            id: 12,
            name: "Fanumoth",
            types: (PokoaType::Normal, Some(PokoaType::Steel)),
            base_stats: Stats {
                hp: 100,
                atk: 130,
                def: 100,
                spa: 80,
                spd: 100,
                spe: 100,
            },
            catch_rate: 3,
            exp_yield: 306,
            evolves_to: None,
            learnable_moves: vec![
                (1, "iron-head"),
                (1, "body-slam"),
                (50, "fanum-tax"),
                (65, "yoink"),
            ],
            description: "Legendary tax collector. It takes 30% of everything you own.",
        },
    ]
}

/// Full move catalog for Pokoa.
pub fn move_catalog() -> HashMap<&'static str, MoveDef> {
    let moves = vec![
        // Basic
        MoveDef {
            id: "splash",
            name: "Splash",
            pokoa_type: PokoaType::Water,
            category: MoveCategory::Status,
            power: 0,
            accuracy: 100,
            pp: 40,
        },
        MoveDef {
            id: "leer",
            name: "Leer",
            pokoa_type: PokoaType::Normal,
            category: MoveCategory::Status,
            power: 0,
            accuracy: 100,
            pp: 30,
        },
        MoveDef {
            id: "growl",
            name: "Growl",
            pokoa_type: PokoaType::Normal,
            category: MoveCategory::Status,
            power: 0,
            accuracy: 100,
            pp: 40,
        },
        MoveDef {
            id: "quick-attack",
            name: "Quick Attack",
            pokoa_type: PokoaType::Normal,
            category: MoveCategory::Physical,
            power: 40,
            accuracy: 100,
            pp: 30,
        },
        MoveDef {
            id: "absorb",
            name: "Absorb",
            pokoa_type: PokoaType::Grass,
            category: MoveCategory::Special,
            power: 20,
            accuracy: 100,
            pp: 25,
        },
        MoveDef {
            id: "confusion",
            name: "Confusion",
            pokoa_type: PokoaType::Psychic,
            category: MoveCategory::Special,
            power: 50,
            accuracy: 100,
            pp: 25,
        },
        MoveDef {
            id: "shadow-sneak",
            name: "Shadow Sneak",
            pokoa_type: PokoaType::Ghost,
            category: MoveCategory::Physical,
            power: 40,
            accuracy: 100,
            pp: 30,
        },
        // Standard
        MoveDef {
            id: "thunder-shock",
            name: "Thunder Shock",
            pokoa_type: PokoaType::Electric,
            category: MoveCategory::Special,
            power: 40,
            accuracy: 100,
            pp: 30,
        },
        MoveDef {
            id: "thunderbolt",
            name: "Thunderbolt",
            pokoa_type: PokoaType::Electric,
            category: MoveCategory::Special,
            power: 90,
            accuracy: 100,
            pp: 15,
        },
        MoveDef {
            id: "thunder",
            name: "Thunder",
            pokoa_type: PokoaType::Electric,
            category: MoveCategory::Special,
            power: 110,
            accuracy: 70,
            pp: 10,
        },
        MoveDef {
            id: "flamethrower",
            name: "Flamethrower",
            pokoa_type: PokoaType::Fire,
            category: MoveCategory::Special,
            power: 90,
            accuracy: 100,
            pp: 15,
        },
        MoveDef {
            id: "psychic",
            name: "Psychic",
            pokoa_type: PokoaType::Psychic,
            category: MoveCategory::Special,
            power: 90,
            accuracy: 100,
            pp: 10,
        },
        MoveDef {
            id: "sludge",
            name: "Sludge",
            pokoa_type: PokoaType::Poison,
            category: MoveCategory::Special,
            power: 65,
            accuracy: 100,
            pp: 20,
        },
        MoveDef {
            id: "sludge-bomb",
            name: "Sludge Bomb",
            pokoa_type: PokoaType::Poison,
            category: MoveCategory::Special,
            power: 90,
            accuracy: 100,
            pp: 10,
        },
        MoveDef {
            id: "shadow-ball",
            name: "Shadow Ball",
            pokoa_type: PokoaType::Ghost,
            category: MoveCategory::Special,
            power: 80,
            accuracy: 100,
            pp: 15,
        },
        MoveDef {
            id: "moonblast",
            name: "Moonblast",
            pokoa_type: PokoaType::Fairy,
            category: MoveCategory::Special,
            power: 95,
            accuracy: 100,
            pp: 15,
        },
        MoveDef {
            id: "dazzling-gleam",
            name: "Dazzling Gleam",
            pokoa_type: PokoaType::Fairy,
            category: MoveCategory::Special,
            power: 80,
            accuracy: 100,
            pp: 10,
        },
        MoveDef {
            id: "body-slam",
            name: "Body Slam",
            pokoa_type: PokoaType::Normal,
            category: MoveCategory::Physical,
            power: 85,
            accuracy: 100,
            pp: 15,
        },
        MoveDef {
            id: "iron-head",
            name: "Iron Head",
            pokoa_type: PokoaType::Steel,
            category: MoveCategory::Physical,
            power: 80,
            accuracy: 100,
            pp: 15,
        },
        MoveDef {
            id: "hydro-pump",
            name: "Hydro Pump",
            pokoa_type: PokoaType::Water,
            category: MoveCategory::Special,
            power: 110,
            accuracy: 80,
            pp: 5,
        },
        MoveDef {
            id: "aqua-jet",
            name: "Aqua Jet",
            pokoa_type: PokoaType::Water,
            category: MoveCategory::Physical,
            power: 40,
            accuracy: 100,
            pp: 20,
        },
        MoveDef {
            id: "dark-pulse",
            name: "Dark Pulse",
            pokoa_type: PokoaType::Dark,
            category: MoveCategory::Special,
            power: 80,
            accuracy: 100,
            pp: 15,
        },
        MoveDef {
            id: "bulk-up",
            name: "Bulk Up",
            pokoa_type: PokoaType::Fighting,
            category: MoveCategory::Status,
            power: 0,
            accuracy: 100,
            pp: 20,
        },
        // Brainrot Signature Moves
        MoveDef {
            id: "flush-cannon",
            name: "Flush Cannon",
            pokoa_type: PokoaType::Water,
            category: MoveCategory::Special,
            power: 65,
            accuracy: 95,
            pp: 15,
        },
        MoveDef {
            id: "plumber-slam",
            name: "Plumber Slam",
            pokoa_type: PokoaType::Fighting,
            category: MoveCategory::Physical,
            power: 75,
            accuracy: 100,
            pp: 15,
        },
        MoveDef {
            id: "sewage-wave",
            name: "Sewage Wave",
            pokoa_type: PokoaType::Poison,
            category: MoveCategory::Special,
            power: 85,
            accuracy: 90,
            pp: 10,
        },
        MoveDef {
            id: "yes-yes-beam",
            name: "Yes Yes Beam",
            pokoa_type: PokoaType::Dark,
            category: MoveCategory::Special,
            power: 90,
            accuracy: 90,
            pp: 10,
        },
        MoveDef {
            id: "dop-dop-cannon",
            name: "Dop Dop Cannon",
            pokoa_type: PokoaType::Water,
            category: MoveCategory::Special,
            power: 100,
            accuracy: 85,
            pp: 5,
        },
        MoveDef {
            id: "mega-flush",
            name: "Mega Flush",
            pokoa_type: PokoaType::Water,
            category: MoveCategory::Special,
            power: 130,
            accuracy: 75,
            pp: 5,
        },
        MoveDef {
            id: "sigma-stare",
            name: "Sigma Stare",
            pokoa_type: PokoaType::Psychic,
            category: MoveCategory::Status,
            power: 0,
            accuracy: 100,
            pp: 20,
        },
        MoveDef {
            id: "grindset-punch",
            name: "Grindset Punch",
            pokoa_type: PokoaType::Fighting,
            category: MoveCategory::Physical,
            power: 60,
            accuracy: 100,
            pp: 20,
        },
        MoveDef {
            id: "sigma-barrage",
            name: "Sigma Barrage",
            pokoa_type: PokoaType::Fighting,
            category: MoveCategory::Physical,
            power: 90,
            accuracy: 90,
            pp: 10,
        },
        MoveDef {
            id: "gigachad-flex",
            name: "Gigachad Flex",
            pokoa_type: PokoaType::Fighting,
            category: MoveCategory::Physical,
            power: 120,
            accuracy: 80,
            pp: 5,
        },
        MoveDef {
            id: "ohio-glitch",
            name: "Ohio Glitch",
            pokoa_type: PokoaType::Ghost,
            category: MoveCategory::Special,
            power: 70,
            accuracy: 100,
            pp: 15,
        },
        MoveDef {
            id: "teleport-strike",
            name: "Teleport Strike",
            pokoa_type: PokoaType::Dark,
            category: MoveCategory::Physical,
            power: 80,
            accuracy: 90,
            pp: 10,
        },
        MoveDef {
            id: "reality-warp",
            name: "Reality Warp",
            pokoa_type: PokoaType::Psychic,
            category: MoveCategory::Special,
            power: 100,
            accuracy: 85,
            pp: 5,
        },
        MoveDef {
            id: "ohio-final-form",
            name: "Ohio Final Form",
            pokoa_type: PokoaType::Dark,
            category: MoveCategory::Special,
            power: 130,
            accuracy: 75,
            pp: 5,
        },
        MoveDef {
            id: "purple-shake",
            name: "Purple Shake",
            pokoa_type: PokoaType::Poison,
            category: MoveCategory::Special,
            power: 55,
            accuracy: 100,
            pp: 20,
        },
        MoveDef {
            id: "grimace-shake-doom",
            name: "Grimace Shake Doom",
            pokoa_type: PokoaType::Poison,
            category: MoveCategory::Special,
            power: 120,
            accuracy: 80,
            pp: 5,
        },
        MoveDef {
            id: "rizz-beam",
            name: "Rizz Beam",
            pokoa_type: PokoaType::Fire,
            category: MoveCategory::Special,
            power: 110,
            accuracy: 90,
            pp: 5,
        },
        MoveDef {
            id: "infinite-rizz",
            name: "Infinite Rizz",
            pokoa_type: PokoaType::Psychic,
            category: MoveCategory::Special,
            power: 140,
            accuracy: 70,
            pp: 5,
        },
        MoveDef {
            id: "fanum-tax",
            name: "Fanum Tax",
            pokoa_type: PokoaType::Dark,
            category: MoveCategory::Physical,
            power: 100,
            accuracy: 100,
            pp: 5,
        },
        MoveDef {
            id: "yoink",
            name: "Yoink",
            pokoa_type: PokoaType::Steel,
            category: MoveCategory::Physical,
            power: 130,
            accuracy: 85,
            pp: 5,
        },
    ];
    moves.into_iter().map(|m| (m.id, m)).collect()
}

// =============================================================================
// Pokemon Instance
// =============================================================================

#[derive(Debug, Clone)]
pub struct Pokoa {
    pub species_id: u16,
    pub nickname: Option<String>,
    pub level: u8,
    pub exp: u32,
    pub nature: Nature,
    pub ivs: Stats,
    pub evs: Stats,
    pub current_hp: u16,
    pub max_hp: u16,
    pub stats: Stats,
    pub moves: Vec<MoveSlot>,
}

impl Pokoa {
    /// Create a new Pokoa at a given level with random IVs (deterministic from seed).
    pub fn new(species: &SpeciesDef, level: u8, nature: Nature, seed: u32) -> Self {
        let ivs = Stats {
            hp: ((seed >> 0) & 31) as u16,
            atk: ((seed >> 5) & 31) as u16,
            def: ((seed >> 10) & 31) as u16,
            spa: ((seed >> 15) & 31) as u16,
            spd: ((seed >> 20) & 31) as u16,
            spe: ((seed >> 25) & 31) as u16,
        };
        let evs = Stats::default();
        let stats = calc_stats(&species.base_stats, &ivs, &evs, level, nature);
        let catalog = move_catalog();
        let moves: Vec<MoveSlot> = species
            .learnable_moves
            .iter()
            .filter(|(lv, _)| *lv <= level)
            .rev()
            .take(4)
            .filter_map(|(_, mid)| catalog.get(mid).map(|m| MoveSlot::new(m.clone())))
            .collect();

        Self {
            species_id: species.id,
            nickname: None,
            level,
            exp: exp_for_level(level),
            nature,
            ivs,
            evs,
            current_hp: stats.hp,
            max_hp: stats.hp,
            stats,
            moves,
        }
    }

    pub fn is_fainted(&self) -> bool {
        self.current_hp == 0
    }

    pub fn take_damage(&mut self, damage: u16) {
        self.current_hp = self.current_hp.saturating_sub(damage);
    }

    pub fn heal(&mut self, amount: u16) {
        self.current_hp = (self.current_hp + amount).min(self.max_hp);
    }

    pub fn heal_full(&mut self) {
        self.current_hp = self.max_hp;
        for m in &mut self.moves {
            m.pp_remaining = m.def.pp;
        }
    }

    /// Add experience and return new level if leveled up.
    pub fn gain_exp(&mut self, amount: u32, species: &SpeciesDef) -> Option<u8> {
        self.exp += amount;
        let new_level = level_from_exp(self.exp);
        if new_level > self.level {
            let old_level = self.level;
            self.level = new_level.min(100);
            self.stats = calc_stats(
                &species.base_stats,
                &self.ivs,
                &self.evs,
                self.level,
                self.nature,
            );
            let old_max = self.max_hp;
            self.max_hp = self.stats.hp;
            self.current_hp = self.current_hp + (self.max_hp - old_max);
            let _ = old_level;
            Some(self.level)
        } else {
            None
        }
    }

    /// Check if this Pokoa can evolve.
    pub fn can_evolve(&self, species: &SpeciesDef) -> bool {
        match species.evolves_to {
            Some((_, EvolutionTrigger::Level(lv))) => self.level >= lv,
            _ => false,
        }
    }

    /// HP percentage (0.0-1.0).
    pub fn hp_pct(&self) -> f32 {
        if self.max_hp == 0 {
            return 0.0;
        }
        self.current_hp as f32 / self.max_hp as f32
    }
}

/// Standard Pokemon stat formula.
fn calc_stats(base: &Stats, iv: &Stats, ev: &Stats, level: u8, nature: Nature) -> Stats {
    let lv = level as u32;
    let hp = ((2 * base.hp as u32 + iv.hp as u32 + ev.hp as u32 / 4) * lv / 100 + lv + 10) as u16;

    let calc_stat = |base_val: u16, iv_val: u16, ev_val: u16, stat_idx: usize| -> u16 {
        let raw = ((2 * base_val as u32 + iv_val as u32 + ev_val as u32 / 4) * lv / 100 + 5) as f32;
        let modifier = match nature.modifiers() {
            Some((b, _)) if b == stat_idx => 1.1,
            Some((_, l)) if l == stat_idx => 0.9,
            _ => 1.0,
        };
        (raw * modifier) as u16
    };

    Stats {
        hp,
        atk: calc_stat(base.atk, iv.atk, ev.atk, 0),
        def: calc_stat(base.def, iv.def, ev.def, 1),
        spa: calc_stat(base.spa, iv.spa, ev.spa, 2),
        spd: calc_stat(base.spd, iv.spd, ev.spd, 3),
        spe: calc_stat(base.spe, iv.spe, ev.spe, 4),
    }
}

fn exp_for_level(level: u8) -> u32 {
    (level as u32).pow(3)
}

fn level_from_exp(exp: u32) -> u8 {
    let mut lv = 1u8;
    while lv < 100 && exp_for_level(lv + 1) <= exp {
        lv += 1;
    }
    lv
}

// =============================================================================
// Battle System
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleType {
    Wild,
    Trainer,
    Gym,
    Legendary,
}

#[derive(Debug, Clone)]
pub struct BattleEvent {
    pub message: String,
    pub effectiveness: f32,
    pub damage: u16,
    pub critical: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleOutcome {
    PlayerWin,
    OpponentWin,
    Caught,
    Fled,
}

#[derive(Debug, Clone)]
pub struct Battle {
    pub battle_type: BattleType,
    pub turn: u32,
    pub player: Pokoa,
    pub opponent: Pokoa,
    pub player_team: Vec<Pokoa>,
    pub opponent_team: Vec<Pokoa>,
    pub log: Vec<BattleEvent>,
    pub outcome: Option<BattleOutcome>,
    rng: SimpleRng,
}

impl Battle {
    pub fn new(battle_type: BattleType, player: Pokoa, opponent: Pokoa) -> Self {
        Self {
            battle_type,
            turn: 0,
            player,
            opponent,
            player_team: vec![],
            opponent_team: vec![],
            log: vec![],
            outcome: None,
            rng: SimpleRng::new(42),
        }
    }

    /// Calculate damage for one attack.
    pub fn calc_damage(
        &mut self,
        attacker: &Pokoa,
        move_def: &MoveDef,
        defender: &Pokoa,
        defender_types: (PokoaType, Option<PokoaType>),
    ) -> (u16, f32, bool) {
        if move_def.category == MoveCategory::Status || move_def.power == 0 {
            return (0, 1.0, false);
        }

        let level = attacker.level as f32;
        let (atk_stat, def_stat) = match move_def.category {
            MoveCategory::Physical => (attacker.stats.atk as f32, defender.stats.def as f32),
            MoveCategory::Special => (attacker.stats.spa as f32, defender.stats.spd as f32),
            MoveCategory::Status => unreachable!(),
        };

        // Standard damage formula
        let base =
            ((2.0 * level / 5.0 + 2.0) * move_def.power as f32 * atk_stat / def_stat) / 50.0 + 2.0;

        // STAB (Same Type Attack Bonus)
        let stab = if move_def.pokoa_type == defender_types.0 {
            1.0
        } else {
            1.5
        }; // TODO: check attacker types
        let effectiveness = PokoaType::calc_effectiveness(move_def.pokoa_type, defender_types);

        // Critical hit (6.25% chance)
        let crit_roll = self.rng.next_f32();
        let critical = crit_roll < 0.0625;
        let crit_mult = if critical { 1.5 } else { 1.0 };

        // Random factor (0.85-1.0)
        let random = self.rng.range(0.85, 1.0);

        let damage = (base * stab * effectiveness * crit_mult * random).max(1.0) as u16;
        (damage, effectiveness, critical)
    }

    /// Execute one turn: player uses move at index, opponent AI selects.
    pub fn execute_turn(&mut self, player_move_idx: usize) -> Vec<BattleEvent> {
        if self.outcome.is_some() {
            return vec![];
        }
        self.turn += 1;
        let mut events = vec![];

        // Determine turn order by speed
        let player_first = self.player.stats.spe >= self.opponent.stats.spe;

        if player_first {
            events.extend(self.attack_phase(true, player_move_idx));
            if self.outcome.is_none() {
                let opp_move = self.select_opponent_move();
                events.extend(self.attack_phase(false, opp_move));
            }
        } else {
            let opp_move = self.select_opponent_move();
            events.extend(self.attack_phase(false, opp_move));
            if self.outcome.is_none() {
                events.extend(self.attack_phase(true, player_move_idx));
            }
        }

        self.log.extend(events.clone());
        events
    }

    fn attack_phase(&mut self, is_player: bool, move_idx: usize) -> Vec<BattleEvent> {
        let mut events = vec![];

        let (attacker_name, move_slot) = if is_player {
            let name = format!("Your Pokoa");
            let slot = self.player.moves.get(move_idx).cloned();
            (name, slot)
        } else {
            let name = format!("Wild Pokoa");
            let slot = self.opponent.moves.get(move_idx).cloned();
            (name, slot)
        };

        let Some(mut slot) = move_slot else {
            events.push(BattleEvent {
                message: format!("{attacker_name} has no move to use!"),
                effectiveness: 1.0,
                damage: 0,
                critical: false,
            });
            return events;
        };

        if !slot.can_use() {
            events.push(BattleEvent {
                message: format!("{attacker_name} has no PP left for {}!", slot.def.name),
                effectiveness: 1.0,
                damage: 0,
                critical: false,
            });
            return events;
        }

        slot.use_pp();

        // Accuracy check
        let acc_roll = self.rng.next_f32() * 100.0;
        if acc_roll > slot.def.accuracy as f32 {
            events.push(BattleEvent {
                message: format!("{attacker_name} used {} but it missed!", slot.def.name),
                effectiveness: 1.0,
                damage: 0,
                critical: false,
            });
            // Update PP in actual pokoa
            if is_player {
                if let Some(m) = self.player.moves.get_mut(move_idx) {
                    m.use_pp();
                }
            } else {
                if let Some(m) = self.opponent.moves.get_mut(move_idx) {
                    m.use_pp();
                }
            }
            return events;
        }

        // Clone data needed for calc to avoid borrow issues
        let attacker = if is_player {
            self.player.clone()
        } else {
            self.opponent.clone()
        };
        let defender = if is_player {
            self.opponent.clone()
        } else {
            self.player.clone()
        };
        let defender_types = (PokoaType::Normal, None); // TODO: look up from dex

        let (damage, effectiveness, critical) =
            self.calc_damage(&attacker, &slot.def, &defender, defender_types);

        // Apply damage
        if is_player {
            self.opponent.take_damage(damage);
            if let Some(m) = self.player.moves.get_mut(move_idx) {
                m.use_pp();
            }
        } else {
            self.player.take_damage(damage);
            if let Some(m) = self.opponent.moves.get_mut(move_idx) {
                m.use_pp();
            }
        }

        let eff_msg = if effectiveness > 1.5 {
            " It's super effective!"
        } else if effectiveness < 0.5 && effectiveness > 0.0 {
            " It's not very effective..."
        } else if effectiveness == 0.0 {
            " It had no effect!"
        } else {
            ""
        };

        let crit_msg = if critical { " Critical hit!" } else { "" };

        events.push(BattleEvent {
            message: format!(
                "{attacker_name} used {}! {damage} damage.{eff_msg}{crit_msg}",
                slot.def.name
            ),
            effectiveness,
            damage,
            critical,
        });

        // Check faint
        if self.opponent.is_fainted() {
            events.push(BattleEvent {
                message: "Wild Pokoa fainted! You win!".into(),
                effectiveness: 1.0,
                damage: 0,
                critical: false,
            });
            self.outcome = Some(BattleOutcome::PlayerWin);
        } else if self.player.is_fainted() {
            events.push(BattleEvent {
                message: "Your Pokoa fainted! You blacked out...".into(),
                effectiveness: 1.0,
                damage: 0,
                critical: false,
            });
            self.outcome = Some(BattleOutcome::OpponentWin);
        }

        events
    }

    fn select_opponent_move(&mut self) -> usize {
        // Simple AI: pick first usable move
        for (i, m) in self.opponent.moves.iter().enumerate() {
            if m.can_use() {
                return i;
            }
        }
        0
    }

    /// Attempt to catch wild Pokoa. Returns true if caught.
    pub fn attempt_catch(&mut self, ball_modifier: f32, catch_rate: u8) -> bool {
        if self.battle_type != BattleType::Wild && self.battle_type != BattleType::Legendary {
            self.log.push(BattleEvent {
                message: "Can't catch a trainer's Pokoa!".into(),
                effectiveness: 1.0,
                damage: 0,
                critical: false,
            });
            return false;
        }

        let hp_factor = (3.0 * self.opponent.max_hp as f32 - 2.0 * self.opponent.current_hp as f32)
            / (3.0 * self.opponent.max_hp as f32);
        let catch_chance = (catch_rate as f32 * ball_modifier * hp_factor) / 255.0;

        let roll = self.rng.next_f32();
        let caught = roll < catch_chance;

        if caught {
            self.log.push(BattleEvent {
                message: "Gotcha! Pokoa was caught!".into(),
                effectiveness: 1.0,
                damage: 0,
                critical: false,
            });
            self.outcome = Some(BattleOutcome::Caught);
        } else {
            let shakes = if roll < catch_chance * 1.5 {
                3
            } else if roll < catch_chance * 2.0 {
                2
            } else {
                1
            };
            self.log.push(BattleEvent {
                message: format!("The ball shook {shakes} time(s)... but the Pokoa broke free!"),
                effectiveness: 1.0,
                damage: 0,
                critical: false,
            });
        }
        caught
    }

    /// Attempt to flee from wild battle.
    pub fn attempt_flee(&mut self) -> bool {
        if self.battle_type == BattleType::Trainer || self.battle_type == BattleType::Gym {
            self.log.push(BattleEvent {
                message: "Can't run from a trainer battle!".into(),
                effectiveness: 1.0,
                damage: 0,
                critical: false,
            });
            return false;
        }

        let flee_chance = (self.player.stats.spe as f32 * 128.0 / self.opponent.stats.spe as f32
            + 30.0 * self.turn as f32)
            / 256.0;
        let roll = self.rng.next_f32();
        let fled = roll < flee_chance.min(1.0);

        if fled {
            self.log.push(BattleEvent {
                message: "Got away safely!".into(),
                effectiveness: 1.0,
                damage: 0,
                critical: false,
            });
            self.outcome = Some(BattleOutcome::Fled);
        } else {
            self.log.push(BattleEvent {
                message: "Can't escape!".into(),
                effectiveness: 1.0,
                damage: 0,
                critical: false,
            });
        }
        fled
    }
}

// =============================================================================
// Trainer
// =============================================================================

#[derive(Debug, Clone)]
pub struct Trainer {
    pub name: String,
    pub team: Vec<Pokoa>,
    pub money: i64,
    pub badges: Vec<String>,
    pub pokedex_seen: Vec<u16>,
    pub pokedex_caught: Vec<u16>,
}

impl Trainer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            team: vec![],
            money: 3000,
            badges: vec![],
            pokedex_seen: vec![],
            pokedex_caught: vec![],
        }
    }

    pub fn add_pokoa(&mut self, pokoa: Pokoa) -> bool {
        if self.team.len() >= 6 {
            return false;
        }
        let species_id = pokoa.species_id;
        self.team.push(pokoa);
        if !self.pokedex_caught.contains(&species_id) {
            self.pokedex_caught.push(species_id);
        }
        if !self.pokedex_seen.contains(&species_id) {
            self.pokedex_seen.push(species_id);
        }
        true
    }

    pub fn see_pokoa(&mut self, species_id: u16) {
        if !self.pokedex_seen.contains(&species_id) {
            self.pokedex_seen.push(species_id);
        }
    }

    pub fn heal_all(&mut self) {
        for p in &mut self.team {
            p.heal_full();
        }
    }

    pub fn first_alive(&self) -> Option<usize> {
        self.team.iter().position(|p| !p.is_fainted())
    }
}

// =============================================================================
// Items
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemType {
    Pokeball { catch_modifier: u8 },
    Potion { heal_amount: u16 },
    Revive { hp_pct: u8 },
    EvolutionItem { item_id: &'static str },
    KeyItem { name: &'static str },
}

#[derive(Debug, Clone)]
pub struct ItemDef {
    pub id: &'static str,
    pub name: &'static str,
    pub item_type: ItemType,
    pub price: u32,
}

pub fn pokoa_items() -> Vec<ItemDef> {
    vec![
        ItemDef {
            id: "pokoa-ball",
            name: "Pokoa Ball",
            item_type: ItemType::Pokeball { catch_modifier: 1 },
            price: 200,
        },
        ItemDef {
            id: "great-ball",
            name: "Great Ball",
            item_type: ItemType::Pokeball { catch_modifier: 2 },
            price: 600,
        },
        ItemDef {
            id: "ultra-ball",
            name: "Ultra Ball",
            item_type: ItemType::Pokeball { catch_modifier: 3 },
            price: 1200,
        },
        ItemDef {
            id: "master-ball",
            name: "Master Ball",
            item_type: ItemType::Pokeball {
                catch_modifier: 255,
            },
            price: 0,
        },
        ItemDef {
            id: "potion",
            name: "Potion",
            item_type: ItemType::Potion { heal_amount: 20 },
            price: 300,
        },
        ItemDef {
            id: "super-potion",
            name: "Super Potion",
            item_type: ItemType::Potion { heal_amount: 50 },
            price: 700,
        },
        ItemDef {
            id: "hyper-potion",
            name: "Hyper Potion",
            item_type: ItemType::Potion { heal_amount: 200 },
            price: 1200,
        },
        ItemDef {
            id: "revive",
            name: "Revive",
            item_type: ItemType::Revive { hp_pct: 50 },
            price: 1500,
        },
        ItemDef {
            id: "protein-shake",
            name: "Protein Shake",
            item_type: ItemType::EvolutionItem {
                item_id: "protein-shake",
            },
            price: 5000,
        },
        ItemDef {
            id: "grimace-shake-item",
            name: "Grimace Shake",
            item_type: ItemType::Potion { heal_amount: 999 },
            price: 0,
        },
    ]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_effectiveness_basic() {
        assert_eq!(
            PokoaType::effectiveness(PokoaType::Fire, PokoaType::Grass),
            2.0
        );
        assert_eq!(
            PokoaType::effectiveness(PokoaType::Water, PokoaType::Fire),
            2.0
        );
        assert_eq!(
            PokoaType::effectiveness(PokoaType::Fire, PokoaType::Water),
            0.5
        );
        assert_eq!(
            PokoaType::effectiveness(PokoaType::Normal, PokoaType::Ghost),
            0.0
        );
        assert_eq!(
            PokoaType::effectiveness(PokoaType::Normal, PokoaType::Normal),
            1.0
        );
    }

    #[test]
    fn dual_type_effectiveness() {
        // Fire vs Water/Ground = 0.5 (Water) * 1.0 (Ground) = 0.5
        let eff = PokoaType::calc_effectiveness(
            PokoaType::Fire,
            (PokoaType::Water, Some(PokoaType::Ground)),
        );
        assert!((eff - 0.5).abs() < 0.01);
        // Electric vs Water/Flying = 2.0 * 2.0 = 4.0
        let eff = PokoaType::calc_effectiveness(
            PokoaType::Electric,
            (PokoaType::Water, Some(PokoaType::Flying)),
        );
        assert!((eff - 4.0).abs() < 0.01);
        // Grass vs Water/Ground = 2.0 * 2.0 = 4.0
        let eff = PokoaType::calc_effectiveness(
            PokoaType::Grass,
            (PokoaType::Water, Some(PokoaType::Ground)),
        );
        assert!((eff - 4.0).abs() < 0.01);
    }

    #[test]
    fn pokoa_dex_has_12_species() {
        let dex = pokoa_dex();
        assert_eq!(dex.len(), 12);
        for s in &dex {
            assert!(s.base_stats.total() > 200);
            assert!(!s.learnable_moves.is_empty());
        }
    }

    #[test]
    fn move_catalog_complete() {
        let catalog = move_catalog();
        let dex = pokoa_dex();
        for species in &dex {
            for (_, move_id) in &species.learnable_moves {
                assert!(
                    catalog.contains_key(move_id),
                    "missing move: {move_id} for {}",
                    species.name
                );
            }
        }
    }

    #[test]
    fn pokoa_creation() {
        let dex = pokoa_dex();
        let toilettle = &dex[0];
        let pokoa = Pokoa::new(toilettle, 10, Nature::Adamant, 12345);
        assert_eq!(pokoa.level, 10);
        assert!(pokoa.max_hp > 0);
        assert!(!pokoa.moves.is_empty());
        assert!(!pokoa.is_fainted());
    }

    #[test]
    fn battle_basic_flow() {
        let dex = pokoa_dex();
        let player = Pokoa::new(&dex[4], 25, Nature::Jolly, 111); // Sigmachu
        let opponent = Pokoa::new(&dex[0], 20, Nature::Hardy, 222); // Toilettle

        let mut battle = Battle::new(BattleType::Wild, player, opponent);
        let events = battle.execute_turn(0);
        assert!(!events.is_empty());
        assert!(battle.turn == 1);
    }

    #[test]
    fn battle_ends_on_faint() {
        let dex = pokoa_dex();
        let player = Pokoa::new(&dex[5], 50, Nature::Adamant, 111); // Gigachad Lv50
        let opponent = Pokoa::new(&dex[0], 5, Nature::Hardy, 222); // Toilettle Lv5

        let mut battle = Battle::new(BattleType::Wild, player, opponent);
        // Gigachad should demolish the Toilettle
        for _ in 0..10 {
            if battle.outcome.is_some() {
                break;
            }
            battle.execute_turn(0);
        }
        assert_eq!(battle.outcome, Some(BattleOutcome::PlayerWin));
    }

    #[test]
    fn catch_mechanics() {
        let dex = pokoa_dex();
        let player = Pokoa::new(&dex[4], 30, Nature::Jolly, 111);
        let mut opponent = Pokoa::new(&dex[0], 5, Nature::Hardy, 222);
        opponent.current_hp = 1; // Very low HP

        let mut battle = Battle::new(BattleType::Wild, player, opponent);
        // With Master Ball (modifier 255), should always catch
        let caught = battle.attempt_catch(255.0, dex[0].catch_rate);
        assert!(caught);
        assert_eq!(battle.outcome, Some(BattleOutcome::Caught));
    }

    #[test]
    fn trainer_team_management() {
        let dex = pokoa_dex();
        let mut trainer = Trainer::new("Brainrot Master");
        for i in 0..6 {
            let pokoa = Pokoa::new(&dex[i % dex.len()], 10, Nature::Hardy, i as u32 * 111);
            assert!(trainer.add_pokoa(pokoa));
        }
        // Team full
        let extra = Pokoa::new(&dex[0], 5, Nature::Hardy, 999);
        assert!(!trainer.add_pokoa(extra));
        assert_eq!(trainer.team.len(), 6);
    }

    #[test]
    fn level_experience_system() {
        assert_eq!(exp_for_level(1), 1);
        assert_eq!(exp_for_level(10), 1000);
        assert_eq!(exp_for_level(100), 1000000);
        assert_eq!(level_from_exp(0), 1);
        assert_eq!(level_from_exp(1000), 10);
        assert_eq!(level_from_exp(8000), 20);
    }

    #[test]
    fn evolution_check() {
        let dex = pokoa_dex();
        let toilettle = &dex[0]; // evolves at level 16
        let pokoa_low = Pokoa::new(toilettle, 10, Nature::Hardy, 111);
        assert!(!pokoa_low.can_evolve(toilettle));
        let pokoa_high = Pokoa::new(toilettle, 16, Nature::Hardy, 111);
        assert!(pokoa_high.can_evolve(toilettle));
    }

    #[test]
    fn legendary_low_catch_rate() {
        let dex = pokoa_dex();
        let rizzlord = &dex[10]; // catch_rate = 3
        assert_eq!(rizzlord.catch_rate, 3);
        assert!(rizzlord.base_stats.total() >= 600);
    }

    #[test]
    fn items_catalog() {
        let items = pokoa_items();
        assert!(items.len() >= 10);
        assert!(items.iter().any(|i| i.id == "master-ball"));
        assert!(items.iter().any(|i| i.id == "protein-shake"));
    }
}
