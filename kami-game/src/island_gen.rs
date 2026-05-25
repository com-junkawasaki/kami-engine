//! Batch Island generator: convert Godot game catalog to KAMI Islands.
//!
//! Each Godot game becomes a KAMI Island with:
//!   - Game-specific scene (genre-based template)
//!   - W Protocol CQRS integration (WRecord for write, Q/G for read)
//!   - KAMI addons (C5-C15) replacing Godot addon GDScript

use crate::brainrot_mesh::BrainrotCharacter;
use crate::scene::{
    CharacterAppearance, CharacterDef, ComponentDef, EntityDef, IslandScene, MeshRef,
};

/// Game metadata from the Godot catalog.
#[derive(Debug, Clone)]
pub struct GameDef {
    pub slug: String,
    pub title: String,
    pub genre: Genre,
    pub max_players: u32,
    pub description: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Genre {
    IoMultiplayer, // agar, slither, diep, mope, splix, hole, paper, wings, zombs
    Puzzle,        // colorbynumber, match3desires
    Rpg,           // dungeonslave, kyberfrontier
    Simulation,    // idolproduction, nightclubtycoon
    VisualNovel,   // loveandglitch
    Card,          // strippoker
    Arcade,        // infinitedive, snake
    Adult,         // alchemistlust, haremconquest, succubusagency
    Brainrot,      // skibidi, sigma, ohio, grimace, rizz, fanum
    Chase,         // ketsu-gorilla
}

/// The 22 Godot games from games.etzhayyim.com.
pub fn godot_game_catalog() -> Vec<GameDef> {
    vec![
        game(
            "agar",
            "Agar Arena",
            Genre::IoMultiplayer,
            100,
            "Grow by absorbing smaller cells",
        ),
        game(
            "slither",
            "Slither World",
            Genre::IoMultiplayer,
            100,
            "Snake multiplayer arena",
        ),
        game(
            "diep",
            "Diep Tanks",
            Genre::IoMultiplayer,
            50,
            "Tank shooter arena",
        ),
        game(
            "mope",
            "Mope Wilderness",
            Genre::IoMultiplayer,
            80,
            "Animal evolution multiplayer",
        ),
        game(
            "splix",
            "Splix Territory",
            Genre::IoMultiplayer,
            50,
            "Territory capture game",
        ),
        game(
            "hole",
            "Hole Devourer",
            Genre::IoMultiplayer,
            30,
            "Devour everything in the city",
        ),
        game(
            "paper",
            "Paper Conquest",
            Genre::IoMultiplayer,
            50,
            "Claim territory on paper",
        ),
        game(
            "wings",
            "Wings Dogfight",
            Genre::IoMultiplayer,
            30,
            "Aerial combat multiplayer",
        ),
        game(
            "zombs",
            "Zombs Defense",
            Genre::IoMultiplayer,
            4,
            "Base defense against zombies",
        ),
        game(
            "snake",
            "Snake Classic",
            Genre::Arcade,
            1,
            "Classic snake game",
        ),
        game(
            "colorbynumber",
            "Color Zen",
            Genre::Puzzle,
            1,
            "Relaxing color puzzle",
        ),
        game(
            "match3desires",
            "Match 3",
            Genre::Puzzle,
            1,
            "Match-3 puzzle game",
        ),
        game(
            "infinitedive",
            "Infinite Dive",
            Genre::Arcade,
            1,
            "Endless falling arcade",
        ),
        game(
            "dungeonslave",
            "Dungeon Quest",
            Genre::Rpg,
            4,
            "Dungeon crawler RPG",
        ),
        game(
            "kyberfrontier",
            "Kyber Frontier",
            Genre::Rpg,
            8,
            "Cyberpunk RPG adventure",
        ),
        game(
            "idolproduction",
            "Idol Manager",
            Genre::Simulation,
            1,
            "Idol production simulation",
        ),
        game(
            "nightclubtycoon",
            "Club Tycoon",
            Genre::Simulation,
            1,
            "Nightclub management sim",
        ),
        game(
            "loveandglitch",
            "Love & Glitch",
            Genre::VisualNovel,
            1,
            "Visual novel romance",
        ),
        game(
            "strippoker",
            "Card Showdown",
            Genre::Card,
            4,
            "Multiplayer card game",
        ),
        game(
            "alchemistlust",
            "Alchemist Lab",
            Genre::Adult,
            1,
            "Alchemy simulation",
        ),
        game(
            "haremconquest",
            "Conquest",
            Genre::Adult,
            1,
            "Strategy conquest game",
        ),
        game(
            "succubusagency",
            "Agency",
            Genre::Adult,
            1,
            "Management simulation",
        ),
        // Brainrot collection
        game(
            "skibidi",
            "Skibidi Arena",
            Genre::Brainrot,
            50,
            "Giant toilet boss battle — dop dop yes yes",
        ),
        game(
            "sigma",
            "Sigma Grindset",
            Genre::Brainrot,
            30,
            "Lone wolf gym simulator — no distractions",
        ),
        game(
            "ohio",
            "Ohio Final Boss",
            Genre::Brainrot,
            20,
            "Only in Ohio — survive the anomalies",
        ),
        game(
            "grimace",
            "Grimace Shake",
            Genre::Brainrot,
            40,
            "Purple blob chaos — don't drink the shake",
        ),
        game(
            "rizz",
            "Rizz Academy",
            Genre::Brainrot,
            25,
            "Master the art of W rizz",
        ),
        game(
            "fanum",
            "Fanum Tax",
            Genre::Brainrot,
            30,
            "Protect your food — tax collectors everywhere",
        ),
        // Chase games
        game(
            "ketsu-gorilla",
            "Goriketsu Dash!!",
            Genre::Chase,
            10,
            "Slap a sleeping gorilla's butt and RUN — goriririri gorigori ketsu dasshu!",
        ),
    ]
}

fn game(slug: &str, title: &str, genre: Genre, max_players: u32, desc: &str) -> GameDef {
    GameDef {
        slug: slug.into(),
        title: title.into(),
        genre,
        max_players,
        description: desc.into(),
    }
}

/// Generate a KAMI IslandScene from a game definition.
pub fn game_to_island(game: &GameDef) -> IslandScene {
    let mut entities = Vec::new();

    // Ground (all genres)
    let ground_color = match game.genre {
        Genre::IoMultiplayer => [0.15, 0.25, 0.35, 1.0],
        Genre::Puzzle => [0.2, 0.3, 0.25, 1.0],
        Genre::Rpg => [0.25, 0.2, 0.15, 1.0],
        Genre::Simulation => [0.3, 0.3, 0.3, 1.0],
        Genre::Arcade => [0.1, 0.1, 0.2, 1.0],
        Genre::Brainrot => [0.15, 0.05, 0.2, 1.0], // purple-dark chaos ground
        Genre::Chase => [0.08, 0.35, 0.05, 1.0],   // jungle green
        _ => [0.2, 0.2, 0.25, 1.0],
    };
    let arena_size = if game.max_players > 30 {
        80.0
    } else if game.max_players > 4 {
        50.0
    } else {
        30.0
    };

    entities.push(entity(
        "ground",
        [0.0, -0.5, 0.0],
        [arena_size, 1.0, arena_size],
        ground_color,
        vec![],
    ));

    // Walls for multiplayer arenas
    if game.max_players > 1 {
        let h = arena_size / 2.0;
        entities.push(entity(
            "wall-n",
            [0.0, 2.0, -h],
            [arena_size, 4.0, 1.0],
            [0.4, 0.4, 0.45, 1.0],
            vec![],
        ));
        entities.push(entity(
            "wall-s",
            [0.0, 2.0, h],
            [arena_size, 4.0, 1.0],
            [0.4, 0.4, 0.45, 1.0],
            vec![],
        ));
        entities.push(entity(
            "wall-e",
            [h, 2.0, 0.0],
            [1.0, 4.0, arena_size],
            [0.4, 0.4, 0.45, 1.0],
            vec![],
        ));
        entities.push(entity(
            "wall-w",
            [-h, 2.0, 0.0],
            [1.0, 4.0, arena_size],
            [0.4, 0.4, 0.45, 1.0],
            vec![],
        ));
    }

    // Player spawns (up to 4 visible, more handled by matchmaking)
    let spawn_count = game.max_players.min(4);
    let spawn_colors = [
        [0.2, 0.6, 1.0, 1.0],
        [1.0, 0.4, 0.2, 1.0],
        [0.2, 0.9, 0.3, 1.0],
        [0.9, 0.9, 0.1, 1.0],
    ];
    for i in 0..spawn_count {
        let angle = (i as f32 / spawn_count as f32) * std::f32::consts::TAU;
        let r = arena_size * 0.3;
        entities.push(entity(
            &format!("spawn-{i}"),
            [r * angle.cos(), 1.0, r * angle.sin()],
            [0.8, 1.6, 0.8],
            spawn_colors[i as usize % 4],
            vec![
                ComponentDef::PlayerSpawn,
                ComponentDef::Physics { dynamic: true },
            ],
        ));
    }

    // Genre-specific decorations
    match game.genre {
        Genre::IoMultiplayer => {
            // Scattered food/orbs
            for i in 0..20 {
                let angle = (i as f32 / 20.0) * std::f32::consts::TAU;
                let r = arena_size * 0.2 + (i as f32 * 1.7) % (arena_size * 0.3);
                entities.push(entity(
                    &format!("orb-{i}"),
                    [r * angle.cos(), 0.3, r * angle.sin()],
                    [0.3, 0.3, 0.3],
                    [0.0, 0.8, 0.6, 1.0],
                    vec![ComponentDef::Item {
                        item_id: "orb".into(),
                        item_name: "Energy Orb".into(),
                    }],
                ));
            }
        }
        Genre::Rpg => {
            // Dungeon structures
            entities.push(entity(
                "castle",
                [0.0, 3.0, -arena_size * 0.3],
                [8.0, 6.0, 8.0],
                [0.4, 0.35, 0.3, 1.0],
                vec![],
            ));
            entities.push(entity(
                "tower-l",
                [-arena_size * 0.2, 4.0, -arena_size * 0.3],
                [3.0, 8.0, 3.0],
                [0.45, 0.4, 0.35, 1.0],
                vec![],
            ));
            entities.push(entity(
                "tower-r",
                [arena_size * 0.2, 4.0, -arena_size * 0.3],
                [3.0, 8.0, 3.0],
                [0.45, 0.4, 0.35, 1.0],
                vec![],
            ));
            // Treasure chests
            entities.push(entity(
                "chest-1",
                [5.0, 0.3, 5.0],
                [0.6, 0.4, 0.4],
                [0.8, 0.7, 0.2, 1.0],
                vec![ComponentDef::Item {
                    item_id: "gem-gold".into(),
                    item_name: "Gold Gem".into(),
                }],
            ));
        }
        Genre::Puzzle => {
            // Color-by-numbers art studio
            let grid_size = 8;
            let cell_size = arena_size / (grid_size as f32 + 2.0);

            // Easel backdrop (wooden frame)
            entities.push(entity(
                "easel-frame",
                [
                    0.0,
                    (grid_size as f32 * cell_size) / 2.0 + 1.0,
                    -arena_size * 0.35,
                ],
                [
                    (grid_size as f32 + 1.0) * cell_size,
                    (grid_size as f32 + 1.0) * cell_size,
                    0.5,
                ],
                [0.55, 0.35, 0.2, 1.0],
                vec![ComponentDef::Physics { dynamic: false }],
            ));

            // Canvas grid cells (colored tiles on easel)
            let palette = [
                [0.99, 0.42, 0.42, 1.0],
                [0.31, 0.80, 0.77, 1.0],
                [0.27, 0.72, 0.82, 1.0],
                [0.59, 0.81, 0.71, 1.0],
                [1.0, 0.92, 0.65, 1.0],
                [0.87, 0.63, 0.87, 1.0],
            ];
            for i in 0..grid_size {
                for j in 0..grid_size {
                    let x = (j as f32 - grid_size as f32 / 2.0 + 0.5) * cell_size;
                    let y = ((grid_size - 1 - i) as f32 + 0.5) * cell_size + 1.0;
                    let z = -arena_size * 0.35 + 0.3;
                    let idx = i * grid_size + j;
                    let color_num = (idx % palette.len()) + 1;
                    // Alternate filled/unfilled for visual interest
                    let color = if (i + j) % 3 == 0 {
                        palette[color_num % palette.len()]
                    } else {
                        [0.92, 0.90, 0.85, 1.0] // unfilled white
                    };
                    entities.push(entity(
                        &format!("cell-{idx}"),
                        [x, y, z],
                        [cell_size * 0.92, cell_size * 0.92, 0.15],
                        color,
                        vec![ComponentDef::Trigger {
                            kind: "paint".into(),
                            data: format!(
                                r#"{{"cell_index":{},"color_number":{}}}"#,
                                idx, color_num
                            ),
                        }],
                    ));
                }
            }

            // Paint bucket items around the easel
            for (i, pal_color) in palette.iter().enumerate() {
                let angle = (i as f32 / palette.len() as f32) * std::f32::consts::TAU;
                let r = arena_size * 0.15;
                entities.push(entity(
                    &format!("paint-{i}"),
                    [r * angle.cos(), 0.35, r * angle.sin()],
                    [0.6, 0.8, 0.6],
                    *pal_color,
                    vec![ComponentDef::Item {
                        item_id: format!("paint-{i}"),
                        item_name: format!("Paint #{}", i + 1),
                    }],
                ));
            }

            // Palette display stand
            entities.push(entity(
                "palette-stand",
                [arena_size * 0.25, 0.5, 0.0],
                [3.0, 1.0, 2.0],
                [0.55, 0.35, 0.2, 1.0],
                vec![],
            ));
        }
        Genre::Brainrot => {
            // Giant Toilet (Skibidi HQ) — center
            entities.push(entity(
                "skibidi-bowl",
                [0.0, 1.5, 0.0],
                [4.0, 3.0, 4.0],
                [0.95, 0.95, 0.95, 1.0],
                vec![ComponentDef::Physics { dynamic: false }],
            ));
            entities.push(entity(
                "skibidi-tank",
                [0.0, 4.0, -2.5],
                [3.5, 3.0, 1.5],
                [0.9, 0.9, 0.92, 1.0],
                vec![],
            ));
            entities.push(entity(
                "skibidi-lid",
                [0.0, 3.5, 0.5],
                [3.8, 0.3, 3.0],
                [0.92, 0.92, 0.95, 1.0],
                vec![],
            ));
            entities.push(entity(
                "skibidi-head",
                [0.0, 5.5, 0.0],
                [2.0, 2.0, 2.0],
                [0.85, 0.7, 0.6, 1.0],
                vec![ComponentDef::Npc {
                    name: "Skibidi Commander".into(),
                    waypoints: vec![[0.0, 5.5, 0.0], [0.0, 6.5, 0.0]],
                }],
            ));

            // Sigma Gym — northeast
            entities.push(entity(
                "sigma-gym-base",
                [arena_size * 0.3, 0.5, -arena_size * 0.3],
                [10.0, 1.0, 10.0],
                [0.25, 0.25, 0.3, 1.0],
                vec![],
            ));
            entities.push(entity(
                "sigma-throne",
                [arena_size * 0.3, 2.0, -arena_size * 0.3],
                [2.0, 3.0, 2.0],
                [0.1, 0.1, 0.15, 1.0],
                vec![],
            ));
            for i in 0..4 {
                let x = arena_size * 0.3 + (i as f32 - 1.5) * 2.0;
                entities.push(EntityDef {
                    id: format!("sigma-dumbbell-{i}"),
                    position: [x, 0.5, -arena_size * 0.3 + 3.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [0.6, 0.6, 0.6],
                    mesh: MeshRef::Sphere {
                        color: [0.3, 0.3, 0.35, 1.0],
                        radius: 0.3,
                    },
                    components: vec![ComponentDef::Item {
                        item_id: format!("dumbbell-{i}"),
                        item_name: "Sigma Weight".into(),
                    }],
                    layer: None,
                });
            }

            // Ohio Obelisk — northwest
            entities.push(entity(
                "ohio-obelisk",
                [-arena_size * 0.35, 6.0, -arena_size * 0.25],
                [1.5, 12.0, 1.5],
                [0.8, 0.0, 0.0, 1.0],
                vec![],
            ));
            for i in 0..6 {
                let angle = (i as f32 / 6.0) * std::f32::consts::TAU;
                let r = 5.0;
                entities.push(entity(
                    &format!("ohio-cube-{i}"),
                    [
                        -arena_size * 0.35 + r * angle.cos(),
                        3.0 + (i as f32) * 0.8,
                        -arena_size * 0.25 + r * angle.sin(),
                    ],
                    [1.2, 1.2, 1.2],
                    [0.9, 0.1, 0.1, 0.9],
                    vec![ComponentDef::Trigger {
                        kind: "damage".into(),
                        data: r#"{"damage":10}"#.into(),
                    }],
                ));
            }

            // Grimace Swamp — southwest
            entities.push(EntityDef {
                id: "grimace-body".into(),
                position: [-arena_size * 0.3, 3.0, arena_size * 0.3],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [5.0, 5.0, 5.0],
                mesh: MeshRef::Sphere {
                    color: [0.5, 0.0, 0.8, 0.85],
                    radius: 2.5,
                },
                components: vec![ComponentDef::Npc {
                    name: "Grimace".into(),
                    waypoints: vec![[-arena_size * 0.3, 3.0, arena_size * 0.3]],
                }],
                layer: None,
            });
            entities.push(EntityDef {
                id: "grimace-head".into(),
                position: [-arena_size * 0.3, 7.0, arena_size * 0.3],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [3.0, 3.0, 3.0],
                mesh: MeshRef::Sphere {
                    color: [0.55, 0.05, 0.85, 0.9],
                    radius: 1.5,
                },
                components: vec![],
                layer: None,
            });
            for i in 0..8 {
                let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
                let r = 8.0;
                entities.push(EntityDef {
                    id: format!("grimace-puddle-{i}"),
                    position: [
                        -arena_size * 0.3 + r * angle.cos(),
                        0.05,
                        arena_size * 0.3 + r * angle.sin(),
                    ],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [2.0, 0.1, 2.0],
                    mesh: MeshRef::Plane {
                        color: [0.4, 0.0, 0.6, 0.7],
                        width: 2.0,
                        depth: 2.0,
                        subdivisions: 1,
                    },
                    components: vec![ComponentDef::Trigger {
                        kind: "damage".into(),
                        data: r#"{"damage":5,"effect":"slow"}"#.into(),
                    }],
                    layer: None,
                });
            }

            // Rizz Academy — southeast
            entities.push(entity(
                "rizz-podium",
                [arena_size * 0.3, 1.0, arena_size * 0.3],
                [3.0, 2.0, 3.0],
                [0.9, 0.15, 0.5, 1.0],
                vec![],
            ));
            entities.push(entity(
                "rizz-stage",
                [arena_size * 0.3, 0.3, arena_size * 0.3],
                [8.0, 0.5, 8.0],
                [0.8, 0.1, 0.4, 1.0],
                vec![],
            ));

            // Fanum Market — east
            entities.push(entity(
                "fanum-stall",
                [arena_size * 0.4, 1.5, 0.0],
                [6.0, 3.0, 4.0],
                [0.9, 0.6, 0.2, 1.0],
                vec![],
            ));
            for i in 0..5 {
                entities.push(entity(
                    &format!("fanum-food-{i}"),
                    [arena_size * 0.4 + (i as f32 - 2.0) * 1.5, 0.5, 2.0],
                    [0.8, 0.8, 0.8],
                    [0.95, 0.75, 0.3, 1.0],
                    vec![ComponentDef::Item {
                        item_id: format!("food-{i}"),
                        item_name: "Fanum Snack".into(),
                    }],
                ));
            }

            // Gyatt orbs
            for i in 0..12 {
                let angle = (i as f32 / 12.0) * std::f32::consts::TAU;
                let r = arena_size * 0.15 + (i as f32 * 2.3) % (arena_size * 0.2);
                entities.push(EntityDef {
                    id: format!("gyatt-orb-{i}"),
                    position: [r * angle.cos(), 0.5, r * angle.sin()],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [0.5, 0.5, 0.5],
                    mesh: MeshRef::Sphere {
                        color: [1.0, 0.2, 0.6, 1.0],
                        radius: 0.25,
                    },
                    components: vec![ComponentDef::Item {
                        item_id: "gyatt-orb".into(),
                        item_name: "Gyatt Energy".into(),
                    }],
                    layer: None,
                });
            }
        }
        Genre::Chase => {
            // Sleeping gorilla at center
            entities.push(entity(
                "gorilla-spot",
                [0.0, 0.1, 0.0],
                [5.0, 0.2, 5.0],
                [0.15, 0.08, 0.03, 1.0],
                vec![ComponentDef::Physics { dynamic: false }],
            ));
            entities.push(EntityDef {
                id: "gorilla-boss".into(),
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
                mesh: MeshRef::Sphere {
                    color: [0.25, 0.15, 0.08, 1.0],
                    radius: 1.8,
                },
                components: vec![ComponentDef::Npc {
                    name: "Goriketsu".into(),
                    waypoints: vec![[0.0, 0.0, 0.0]],
                }],
                layer: None,
            });
            // Gorilla's RED BUTT
            entities.push(EntityDef {
                id: "gorilla-butt".into(),
                position: [0.0, 1.0, -1.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [2.2, 2.2, 2.2],
                mesh: MeshRef::Sphere {
                    color: [0.95, 0.15, 0.1, 1.0],
                    radius: 1.1,
                },
                components: vec![],
                layer: None,
            });

            // Palm trees scattered around
            for i in 0..8 {
                let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
                let r = arena_size * 0.2 + (i as f32 * 3.7) % (arena_size * 0.15);
                entities.push(EntityDef {
                    id: format!("palm-{i}"),
                    position: [r * angle.cos(), 0.0, r * angle.sin()],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                    mesh: MeshRef::Cylinder {
                        color: [0.45, 0.3, 0.15, 1.0],
                        h: 6.0 + i as f32 * 0.5,
                        r1: 0.5,
                        r2: 0.3,
                    },
                    components: vec![ComponentDef::Physics { dynamic: false }],
                    layer: None,
                });
            }

            // Bananas scattered
            for i in 0..20 {
                let angle = (i as f32 / 20.0) * std::f32::consts::TAU + 0.3;
                let r = 10.0 + (i as f32 * 2.1) % 30.0;
                entities.push(EntityDef {
                    id: format!("banana-{i}"),
                    position: [r * angle.cos(), 0.5, r * angle.sin()],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [0.5, 0.5, 0.5],
                    mesh: MeshRef::Sphere {
                        color: [1.0, 0.85, 0.0, 1.0],
                        radius: 0.3,
                    },
                    components: vec![ComponentDef::Item {
                        item_id: format!("banana-{i}"),
                        item_name: "Banana".into(),
                    }],
                    layer: None,
                });
            }

            // River + hazards
            entities.push(entity(
                "river",
                [0.0, -0.3, arena_size * 0.3],
                [8.0, 0.1, 20.0],
                [0.1, 0.3, 0.6, 0.7],
                vec![ComponentDef::Trigger {
                    kind: "slow".into(),
                    data: "Kawa da!".into(),
                }],
            ));
            entities.push(entity(
                "vine-bridge",
                [0.0, 0.5, arena_size * 0.3],
                [2.0, 0.3, 6.0],
                [0.3, 0.2, 0.08, 1.0],
                vec![ComponentDef::Physics { dynamic: false }],
            ));

            // Baby gorilla den
            entities.push(EntityDef {
                id: "baby-gorilla-den".into(),
                position: [15.0, 0.0, 10.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [4.0, 2.5, 4.0],
                mesh: MeshRef::Sphere {
                    color: [0.2, 0.15, 0.08, 0.8],
                    radius: 2.5,
                },
                components: vec![ComponentDef::Trigger {
                    kind: "story".into(),
                    data: "Ko-gorira ga maigo de naiteiru...".into(),
                }],
                layer: None,
            });

            // Burnt jungle zone (environmental storytelling)
            entities.push(entity(
                "burnt-zone",
                [arena_size * 0.35, 0.0, arena_size * 0.25],
                [15.0, 0.1, 15.0],
                [0.15, 0.1, 0.05, 1.0],
                vec![],
            ));
        }
        _ => {
            // Generic pillars
            for i in 0..4 {
                let angle = (i as f32 / 4.0) * std::f32::consts::TAU;
                let r = arena_size * 0.25;
                entities.push(entity(
                    &format!("pillar-{i}"),
                    [r * angle.cos(), 1.5, r * angle.sin()],
                    [1.0, 3.0, 1.0],
                    [0.5, 0.5, 0.55, 1.0],
                    vec![],
                ));
            }
        }
    }

    // NPC (guard/merchant for RPG, announcer for arcade)
    entities.push(entity(
        "npc-1",
        [arena_size * 0.3, 0.8, 0.0],
        [1.0, 1.6, 1.0],
        [0.9, 0.7, 0.1, 1.0],
        vec![ComponentDef::Npc {
            name: match game.genre {
                Genre::Rpg => "Blacksmith".into(),
                Genre::Simulation => "Advisor".into(),
                Genre::Brainrot => "Rizz Master".into(),
                Genre::Chase => "Ko-Gorira".into(),
                _ => "Announcer".into(),
            },
            waypoints: vec![[arena_size * 0.3, 0.8, -5.0], [arena_size * 0.3, 0.8, 5.0]],
        }],
    ));

    // Portal back to hub
    entities.push(entity(
        "portal-hub",
        [0.0, 1.5, arena_size * 0.45],
        [3.0, 3.0, 0.5],
        [0.5, 0.0, 1.0, 0.8],
        vec![ComponentDef::Portal {
            target_island: "hub".into(),
        }],
    ));

    // Items (HP potion + gems)
    entities.push(entity(
        "potion-1",
        [-3.0, 0.3, -3.0],
        [0.4, 0.6, 0.4],
        [1.0, 0.2, 0.3, 1.0],
        vec![ComponentDef::Item {
            item_id: "potion-hp".into(),
            item_name: "Health Potion".into(),
        }],
    ));
    entities.push(entity(
        "gem-1",
        [3.0, 0.3, 3.0],
        [0.5, 0.5, 0.5],
        [0.0, 0.5, 1.0, 1.0],
        vec![ComponentDef::Item {
            item_id: "gem-blue".into(),
            item_name: "Blue Gem".into(),
        }],
    ));

    IslandScene {
        context: None,
        ld_type: None,
        ld_id: None,
        name: game.title.clone(),
        genre: Some(format!("{:?}", game.genre).to_lowercase()),
        description: Some(game.description.clone()),
        max_players: Some(game.max_players),
        characters: vec![],
        entities,
        ambient_color: [0.03, 0.03, 0.05],
        sun_direction: [-1.0, -2.0, -1.0],
        sun_intensity: 3.0,
        camera_mode: None,
        layers: vec![],
        viewport: None,
        sun_color: None,
        point_lights: vec![],
        atmosphere: None,
        postfx_preset: None,
        ibl_env_map: None,
        shadow: None,
    }
}

fn entity(
    id: &str,
    pos: [f32; 3],
    scale: [f32; 3],
    color: [f32; 4],
    components: Vec<ComponentDef>,
) -> EntityDef {
    EntityDef {
        id: id.into(),
        position: pos,
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale,
        mesh: MeshRef::Cube { color },
        components,
        layer: None,
    }
}

/// Generate batch publish commands for all games (22 originals + 6 brainrot).
/// Returns (island_scene_jsonld, game_def) pairs.
pub fn generate_all_islands() -> Vec<(String, GameDef)> {
    godot_game_catalog()
        .into_iter()
        .map(|game| {
            let mut scene = game_to_island(&game);
            // Stamp JSON-LD metadata
            scene.context = Some("https://etzhayyim.com/ns/kami/scene".into());
            scene.ld_type = Some("IslandScene".into());
            scene.ld_id = Some(format!("urn:kami:island:{}", game.slug));
            let json = serde_json::to_string(&scene).unwrap();
            (json, game)
        })
        .collect()
}

/// Generate brainrot-only islands with characters.
pub fn generate_brainrot_islands() -> Vec<(String, GameDef)> {
    let brainrot_chars = brainrot_characters();
    godot_game_catalog()
        .into_iter()
        .filter(|g| matches!(g.genre, Genre::Brainrot))
        .map(|game| {
            let mut scene = game_to_island(&game);
            scene.context = Some("https://etzhayyim.com/ns/kami/scene".into());
            scene.ld_type = Some("IslandScene".into());
            scene.ld_id = Some(format!("urn:kami:island:{}", game.slug));
            // Attach relevant characters
            scene.characters = brainrot_chars
                .iter()
                .filter(|c| {
                    c.spawn_points.iter().any(|s| s == &game.slug)
                        || c.spawn_points.contains(&"all".to_string())
                })
                .cloned()
                .collect();
            let json = serde_json::to_string_pretty(&scene).unwrap();
            (json, game)
        })
        .collect()
}

// =============================================================================
// Brainrot Evolution — Pokémon-style multi-stage definitions
// =============================================================================

/// Evolution stage gate: social rank (Well-Becoming) + domain achievement must both be met.
#[derive(Debug, Clone)]
pub struct EvolutionStage {
    pub stage: u8,
    pub form_name: String,
    pub social_gate: String,
    pub domain_gate: String,
    pub scale: f32,
    /// Character appearance overrides at this stage (body build, accessories, etc.)
    pub body_override: Option<String>,
    pub accessory_override: Option<String>,
}

/// Full evolution chain for a brainrot character.
#[derive(Debug, Clone)]
pub struct BrainrotEvolution {
    pub character_id: String,
    pub character_enum: BrainrotCharacter,
    pub stages: Vec<EvolutionStage>,
}

/// All brainrot evolution chains.
pub fn brainrot_evolution_chains() -> Vec<BrainrotEvolution> {
    vec![
        BrainrotEvolution {
            character_id: "char-skibidi-commander".into(),
            character_enum: BrainrotCharacter::Skibidi,
            stages: vec![
                evo_stage(0, "Mini Toilet", "", "", 0.6, None, None),
                evo_stage(
                    1,
                    "Skibidi Soldier",
                    "kyu4",
                    "boss_kills>=50",
                    1.0,
                    Some("stocky"),
                    Some("sunglasses"),
                ),
                evo_stage(
                    2,
                    "Skibidi Tank",
                    "kyu1",
                    "a2a_raids>=10",
                    1.8,
                    Some("stocky"),
                    Some("sunglasses"),
                ),
                evo_stage(
                    3,
                    "Skibidi Titan",
                    "dan3",
                    "all_brainrot_a2a",
                    3.0,
                    Some("stocky"),
                    Some("mask"),
                ),
            ],
        },
        BrainrotEvolution {
            character_id: "char-sigma-male".into(),
            character_enum: BrainrotCharacter::Sigma,
            stages: vec![
                evo_stage(0, "Scrawny Kid", "", "", 0.7, Some("slim"), None),
                evo_stage(
                    1,
                    "Gym Bro",
                    "kyu5",
                    "streak>=7",
                    1.0,
                    Some("athletic"),
                    Some("sunglasses"),
                ),
                evo_stage(
                    2,
                    "Sigma Male",
                    "kyu3",
                    "streak>=30,pr>=10",
                    1.1,
                    Some("athletic"),
                    Some("sunglasses"),
                ),
                evo_stage(
                    3,
                    "Gigachad",
                    "kyu1",
                    "agents_trained>=5",
                    1.3,
                    Some("stocky"),
                    Some("sunglasses"),
                ),
                evo_stage(
                    4,
                    "Sigma Ascended",
                    "dan5",
                    "streak>=100,all_follow",
                    1.5,
                    Some("tall"),
                    None,
                ),
            ],
        },
        BrainrotEvolution {
            character_id: "char-ohio-boss".into(),
            character_enum: BrainrotCharacter::Ohio,
            stages: vec![
                evo_stage(0, "Ohio Anomaly", "", "", 1.0, Some("tall"), Some("mask")),
                evo_stage(
                    1,
                    "Ohio Nightmare",
                    "kyu3",
                    "anomaly_types>=12",
                    2.0,
                    Some("tall"),
                    Some("mask"),
                ),
                evo_stage(
                    2,
                    "Ohio Eldritch",
                    "dan2",
                    "survival_rate<20,all_export",
                    4.0,
                    Some("tall"),
                    Some("mask"),
                ),
            ],
        },
        BrainrotEvolution {
            character_id: "char-grimace".into(),
            character_enum: BrainrotCharacter::Grimace,
            stages: vec![
                evo_stage(0, "Purple Puddle", "", "", 0.5, Some("stocky"), None),
                evo_stage(
                    1,
                    "Grimace Blob",
                    "kyu4",
                    "recipes>=5",
                    1.0,
                    Some("stocky"),
                    None,
                ),
                evo_stage(
                    2,
                    "Grimace Tide",
                    "kyu1",
                    "poison_supply>=10",
                    1.8,
                    Some("stocky"),
                    None,
                ),
                evo_stage(
                    3,
                    "Grimace Singularity",
                    "dan4",
                    "all_agent_chaos_event",
                    2.5,
                    Some("stocky"),
                    None,
                ),
            ],
        },
        BrainrotEvolution {
            character_id: "char-rizz-master".into(),
            character_enum: BrainrotCharacter::Rizz,
            stages: vec![
                evo_stage(0, "Awkward Kid", "", "", 0.8, Some("slim"), None),
                evo_stage(
                    1,
                    "Rizz Master",
                    "kyu3",
                    "like_rate>=30",
                    1.0,
                    Some("slim"),
                    Some("earring"),
                ),
                evo_stage(
                    2,
                    "Rizz Sensei",
                    "dan1",
                    "agents_promoted>=5",
                    1.1,
                    Some("slim"),
                    Some("earring"),
                ),
            ],
        },
        BrainrotEvolution {
            character_id: "char-fanum".into(),
            character_enum: BrainrotCharacter::Fanum,
            stages: vec![
                evo_stage(0, "Street Kid", "", "", 0.8, Some("average"), Some("hat")),
                evo_stage(
                    1,
                    "Tax Collector",
                    "kyu4",
                    "food_types>=10",
                    1.0,
                    Some("average"),
                    Some("hat"),
                ),
                evo_stage(
                    2,
                    "Tax Baron",
                    "kyu1",
                    "all_supply_chain",
                    1.1,
                    Some("stocky"),
                    Some("hat"),
                ),
                evo_stage(
                    3,
                    "Fanum Mogul",
                    "dan3",
                    "economy_volume_threshold,redistribute>=100",
                    1.4,
                    Some("stocky"),
                    Some("hat"),
                ),
            ],
        },
    ]
}

fn evo_stage(
    stage: u8,
    name: &str,
    social: &str,
    domain: &str,
    scale: f32,
    body: Option<&str>,
    accessory: Option<&str>,
) -> EvolutionStage {
    EvolutionStage {
        stage,
        form_name: name.into(),
        social_gate: social.into(),
        domain_gate: domain.into(),
        scale,
        body_override: body.map(|s| s.into()),
        accessory_override: accessory.map(|s| s.into()),
    }
}

/// Brainrot character roster.
pub fn brainrot_characters() -> Vec<CharacterDef> {
    vec![
        CharacterDef {
            ld_type: Some("KamiCharacter".into()),
            id: "char-skibidi-commander".into(),
            name: "Skibidi Commander".into(),
            role: Some("boss".into()),
            appearance: CharacterAppearance {
                face: "square".into(),
                skin_hue: 0.08,
                skin_lightness: 0.65,
                eye: "wide".into(),
                eye_color_hue: 0.1,
                eye_size: 1.4,
                nose: "large".into(),
                mouth: "grin".into(),
                mouth_size: 1.5,
                hair: "buzz".into(),
                hair_color_hue: 0.08,
                hair_color_lightness: 0.2,
                body: "stocky".into(),
                height: 1.15,
                accessory1: "sunglasses".into(),
                accessory2: "none".into(),
            },
            spawn_points: vec!["skibidi".into(), "all".into()],
        },
        CharacterDef {
            ld_type: Some("KamiCharacter".into()),
            id: "char-sigma-male".into(),
            name: "Sigma Grindset".into(),
            role: Some("npc".into()),
            appearance: CharacterAppearance {
                face: "diamond".into(),
                skin_hue: 0.06,
                skin_lightness: 0.55,
                eye: "narrow".into(),
                eye_color_hue: 0.6,
                eye_size: 0.8,
                nose: "pointed".into(),
                mouth: "neutral".into(),
                mouth_size: 0.7,
                hair: "spiky".into(),
                hair_color_hue: 0.0,
                hair_color_lightness: 0.1,
                body: "athletic".into(),
                height: 1.1,
                accessory1: "sunglasses".into(),
                accessory2: "none".into(),
            },
            spawn_points: vec!["sigma".into(), "all".into()],
        },
        CharacterDef {
            ld_type: Some("KamiCharacter".into()),
            id: "char-ohio-boss".into(),
            name: "Ohio Final Boss".into(),
            role: Some("boss".into()),
            appearance: CharacterAppearance {
                face: "long".into(),
                skin_hue: 0.0,
                skin_lightness: 0.3,
                eye: "cat".into(),
                eye_color_hue: 0.0,
                eye_size: 1.3,
                nose: "flat".into(),
                mouth: "wide".into(),
                mouth_size: 1.4,
                hair: "mohawk".into(),
                hair_color_hue: 0.0,
                hair_color_lightness: 0.0,
                body: "tall".into(),
                height: 1.2,
                accessory1: "mask".into(),
                accessory2: "none".into(),
            },
            spawn_points: vec!["ohio".into()],
        },
        CharacterDef {
            ld_type: Some("KamiCharacter".into()),
            id: "char-grimace".into(),
            name: "Grimace".into(),
            role: Some("boss".into()),
            appearance: CharacterAppearance {
                face: "round".into(),
                skin_hue: 0.75,
                skin_lightness: 0.4,
                eye: "round".into(),
                eye_color_hue: 0.75,
                eye_size: 1.2,
                nose: "button".into(),
                mouth: "smile".into(),
                mouth_size: 1.3,
                hair: "bald".into(),
                hair_color_hue: 0.75,
                hair_color_lightness: 0.3,
                body: "stocky".into(),
                height: 1.15,
                accessory1: "none".into(),
                accessory2: "none".into(),
            },
            spawn_points: vec!["grimace".into()],
        },
        CharacterDef {
            ld_type: Some("KamiCharacter".into()),
            id: "char-rizz-master".into(),
            name: "Rizz Master".into(),
            role: Some("npc".into()),
            appearance: CharacterAppearance {
                face: "heart".into(),
                skin_hue: 0.07,
                skin_lightness: 0.6,
                eye: "almond".into(),
                eye_color_hue: 0.35,
                eye_size: 1.1,
                nose: "small".into(),
                mouth: "smile".into(),
                mouth_size: 1.2,
                hair: "wavy".into(),
                hair_color_hue: 0.08,
                hair_color_lightness: 0.3,
                body: "slim".into(),
                height: 1.05,
                accessory1: "earring".into(),
                accessory2: "none".into(),
            },
            spawn_points: vec!["rizz".into(), "all".into()],
        },
        CharacterDef {
            ld_type: Some("KamiCharacter".into()),
            id: "char-fanum".into(),
            name: "Fanum Tax Collector".into(),
            role: Some("npc".into()),
            appearance: CharacterAppearance {
                face: "oval".into(),
                skin_hue: 0.07,
                skin_lightness: 0.45,
                eye: "droopy".into(),
                eye_color_hue: 0.08,
                eye_size: 1.0,
                nose: "medium".into(),
                mouth: "pout".into(),
                mouth_size: 1.1,
                hair: "afro".into(),
                hair_color_hue: 0.0,
                hair_color_lightness: 0.15,
                body: "average".into(),
                height: 1.0,
                accessory1: "hat".into(),
                accessory2: "none".into(),
            },
            spawn_points: vec!["fanum".into(), "all".into()],
        },
        // YORO mascot — green blob with chef hat, big blue eyes, toothy grin
        CharacterDef {
            ld_type: Some("KamiCharacter".into()),
            id: "char-yoro-mascot".into(),
            name: "YORO".into(),
            role: Some("mascot".into()),
            appearance: CharacterAppearance {
                face: "round".into(),
                skin_hue: 0.33,
                skin_lightness: 0.55,
                eye: "round".into(),
                eye_color_hue: 0.58,
                eye_size: 1.4,
                nose: "button".into(),
                mouth: "grin".into(),
                mouth_size: 1.5,
                hair: "bald".into(),
                hair_color_hue: 0.0,
                hair_color_lightness: 0.9,
                body: "stocky".into(),
                height: 0.9,
                accessory1: "hat".into(),
                accessory2: "none".into(),
            },
            spawn_points: vec!["yoro".into(), "all".into()],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_28_games_generate_valid_islands() {
        let islands = generate_all_islands();
        assert_eq!(islands.len(), 28); // 22 originals + 6 brainrot
        for (json, game) in &islands {
            let parsed: IslandScene = serde_json::from_str(json).unwrap();
            assert_eq!(parsed.name, game.title);
            assert!(
                parsed.entities.len() >= 5,
                "game {} has too few entities",
                game.slug
            );
            assert!(parsed.entities.iter().any(|e| e.id == "ground"));
            assert!(parsed.entities.iter().any(|e| e.id.starts_with("spawn-")));
            assert!(parsed.entities.iter().any(|e| e.id == "portal-hub"));
            assert!(parsed.entities.iter().any(|e| e.id == "npc-1"));
            // JSON-LD fields
            assert!(parsed.context.is_some());
            assert!(parsed.ld_type.as_deref() == Some("IslandScene"));
        }
    }

    #[test]
    fn io_multiplayer_has_orbs() {
        let agar = godot_game_catalog()
            .into_iter()
            .find(|g| g.slug == "agar")
            .unwrap();
        let scene = game_to_island(&agar);
        let orb_count = scene
            .entities
            .iter()
            .filter(|e| e.id.starts_with("orb-"))
            .count();
        assert_eq!(orb_count, 20);
    }

    #[test]
    fn rpg_has_castle() {
        let dungeon = godot_game_catalog()
            .into_iter()
            .find(|g| g.slug == "dungeonslave")
            .unwrap();
        let scene = game_to_island(&dungeon);
        assert!(scene.entities.iter().any(|e| e.id == "castle"));
    }

    #[test]
    fn brainrot_has_skibidi_toilet() {
        let skibidi = godot_game_catalog()
            .into_iter()
            .find(|g| g.slug == "skibidi")
            .unwrap();
        let scene = game_to_island(&skibidi);
        assert!(scene.entities.iter().any(|e| e.id == "skibidi-bowl"));
        assert!(scene.entities.iter().any(|e| e.id == "skibidi-head"));
        assert!(scene.entities.iter().any(|e| e.id == "sigma-throne"));
        assert!(scene.entities.iter().any(|e| e.id == "ohio-obelisk"));
        assert!(scene.entities.iter().any(|e| e.id == "grimace-body"));
        assert!(scene.entities.iter().any(|e| e.id == "rizz-podium"));
        assert!(scene.entities.iter().any(|e| e.id == "fanum-stall"));
    }

    #[test]
    fn brainrot_islands_have_characters() {
        let islands = generate_brainrot_islands();
        assert_eq!(islands.len(), 6);
        for (json, game) in &islands {
            let parsed: IslandScene = serde_json::from_str(json).unwrap();
            assert!(
                !parsed.characters.is_empty(),
                "brainrot game {} has no characters",
                game.slug
            );
            assert!(parsed.context.is_some());
        }
        // skibidi island should have Skibidi Commander
        let (skibidi_json, _) = islands.iter().find(|(_, g)| g.slug == "skibidi").unwrap();
        let skibidi: IslandScene = serde_json::from_str(skibidi_json).unwrap();
        assert!(
            skibidi
                .characters
                .iter()
                .any(|c| c.id == "char-skibidi-commander")
        );
    }

    #[test]
    fn puzzle_has_coloring_scene() {
        let cbn = godot_game_catalog()
            .into_iter()
            .find(|g| g.slug == "colorbynumber")
            .unwrap();
        let scene = game_to_island(&cbn);
        // Easel frame
        assert!(scene.entities.iter().any(|e| e.id == "easel-frame"));
        // Grid cells (8x8 = 64)
        let cell_count = scene
            .entities
            .iter()
            .filter(|e| e.id.starts_with("cell-"))
            .count();
        assert_eq!(cell_count, 64);
        // Paint items
        let paint_count = scene
            .entities
            .iter()
            .filter(|e| e.id.starts_with("paint-"))
            .count();
        assert_eq!(paint_count, 6);
        // Palette stand
        assert!(scene.entities.iter().any(|e| e.id == "palette-stand"));
        // Cells have paint trigger
        let cell_0 = scene.entities.iter().find(|e| e.id == "cell-0").unwrap();
        assert!(
            cell_0
                .components
                .iter()
                .any(|c| matches!(c, ComponentDef::Trigger { kind, .. } if kind == "paint"))
        );
    }

    #[test]
    fn brainrot_characters_valid() {
        let chars = brainrot_characters();
        assert_eq!(chars.len(), 7); // 6 brainrot + YORO mascot
        for c in &chars {
            assert!(!c.name.is_empty());
            assert!(!c.spawn_points.is_empty());
            assert!(c.appearance.skin_hue >= 0.0 && c.appearance.skin_hue <= 1.0);
            assert!(c.appearance.height >= 0.8 && c.appearance.height <= 1.2);
        }
    }

    #[test]
    fn brainrot_evolution_chains_valid() {
        let chains = brainrot_evolution_chains();
        assert_eq!(chains.len(), 6); // 6 brainrot characters
        // Verify stage counts match BrainrotCharacter::max_stage
        for chain in &chains {
            let expected = chain.character_enum.max_stage() as usize + 1;
            assert_eq!(
                chain.stages.len(),
                expected,
                "{} has {} stages, expected {}",
                chain.character_id,
                chain.stages.len(),
                expected
            );
            // Stage 0 should have no social gate
            assert!(
                chain.stages[0].social_gate.is_empty(),
                "{} stage 0 should have empty social_gate",
                chain.character_id
            );
            // Final stage should have dan gate
            let final_stage = &chain.stages[chain.stages.len() - 1];
            if chain.stages.len() > 2 {
                assert!(
                    final_stage.social_gate.starts_with("dan"),
                    "{} final stage should have dan gate, got '{}'",
                    chain.character_id,
                    final_stage.social_gate
                );
            }
            // Scales should be monotonically increasing
            for w in chain.stages.windows(2) {
                assert!(
                    w[1].scale >= w[0].scale,
                    "{} scale should increase: stage {} ({}) >= stage {} ({})",
                    chain.character_id,
                    w[1].stage,
                    w[1].scale,
                    w[0].stage,
                    w[0].scale
                );
            }
        }
    }

    #[test]
    fn evolution_chain_character_ids_match_roster() {
        let chains = brainrot_evolution_chains();
        let chars = brainrot_characters();
        for chain in &chains {
            assert!(
                chars.iter().any(|c| c.id == chain.character_id),
                "evolution chain {} not found in character roster",
                chain.character_id
            );
        }
    }
}
