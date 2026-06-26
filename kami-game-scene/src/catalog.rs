//! Game-catalog data tier — `kami-game`'s Godot game catalog
//! (`island_gen::godot_game_catalog()`) as parity-tested EDN.
//!
//! Scene generation (`game_to_island`) stays native Rust; only the init-time
//! **description** — the per-game metadata table (slug / title / genre / max-players /
//! description) — moves to EDN (ADR-0046 / ADR-0038). [`catalog_from_edn`] rebuilds the
//! real [`kami_game::island_gen::GameDef`] list, asserted entry-for-entry `==` the
//! compiled-in `godot_game_catalog()` in `tests/catalog_parity.rs`.
//!
//! `GameDef` / `Genre` derive no `PartialEq`, so the data tier carries a local
//! [`GameDefSpec`] `PartialEq` mirror used for parity (`:genre` ↔ a hyphenated keyword id).

use std::collections::BTreeMap;

use kami_game::island_gen::{godot_game_catalog, GameDef, Genre};
use kami_scene::{kw_key, mget, root_map, EdnValue};

/// The canonical game-catalog CONFIG shipped with this crate.
pub const GAME_CATALOG_EDN: &str = include_str!("../data/game_catalog.edn");

/// Errors raised while loading the game catalog from EDN.
#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    /// The EDN source did not parse to a top-level map.
    #[error("game-catalog EDN root is not a map")]
    NotAMap,
    /// The `:game/catalog` table was missing or not a vector.
    #[error("`:game/catalog` missing or not a vector")]
    NoCatalog,
}

/// PartialEq mirror of [`GameDef`] (which derives none), for parity assertions.
#[derive(Debug, Clone, PartialEq)]
pub struct GameDefSpec {
    /// Stable game slug (e.g. `"agar"`).
    pub slug: String,
    /// Display title.
    pub title: String,
    /// Hyphenated [`Genre`] keyword id (e.g. `"io-multiplayer"`).
    pub genre: String,
    /// Max concurrent players.
    pub max_players: u32,
    /// One-line description.
    pub description: String,
}

/// The hyphenated `:genre` keyword id for a [`Genre`] variant.
pub fn genre_id(g: &Genre) -> &'static str {
    match g {
        Genre::IoMultiplayer => "io-multiplayer",
        Genre::Puzzle => "puzzle",
        Genre::Rpg => "rpg",
        Genre::Simulation => "simulation",
        Genre::VisualNovel => "visual-novel",
        Genre::Card => "card",
        Genre::Arcade => "arcade",
        Genre::Adult => "adult",
        Genre::Brainrot => "brainrot",
        Genre::Chase => "chase",
    }
}

/// Inverse of [`genre_id`]; unknown ids fall back to `IoMultiplayer` (tolerant).
pub fn genre_from_id(id: &str) -> Genre {
    match id {
        "puzzle" => Genre::Puzzle,
        "rpg" => Genre::Rpg,
        "simulation" => Genre::Simulation,
        "visual-novel" => Genre::VisualNovel,
        "card" => Genre::Card,
        "arcade" => Genre::Arcade,
        "adult" => Genre::Adult,
        "brainrot" => Genre::Brainrot,
        "chase" => Genre::Chase,
        _ => Genre::IoMultiplayer,
    }
}

impl GameDefSpec {
    /// Read the spec straight off the real engine struct (the parity oracle source).
    pub fn from_game_def(g: &GameDef) -> Self {
        Self {
            slug: g.slug.clone(),
            title: g.title.clone(),
            genre: genre_id(&g.genre).to_string(),
            max_players: g.max_players,
            description: g.description.clone(),
        }
    }

    /// Build a spec from one catalog entry's EDN map (tolerant: missing → default,
    /// int coerces to u32).
    pub fn from_map(m: &BTreeMap<EdnValue, EdnValue>) -> Self {
        let s = |k: &str| {
            mget(m, k)
                .and_then(|v| v.as_string())
                .unwrap_or("")
                .to_string()
        };
        let genre = mget(m, "genre").and_then(kw_key).unwrap_or_default();
        let max_players = mget(m, "max-players")
            .and_then(|v| v.as_integer())
            .unwrap_or(0)
            .max(0) as u32;
        Self {
            slug: s("slug"),
            title: s("title"),
            genre,
            max_players,
            description: s("description"),
        }
    }
}

/// Reconstruct the real [`GameDef`] from a [`GameDefSpec`].
pub fn spec_to_game_def(s: &GameDefSpec) -> GameDef {
    GameDef {
        slug: s.slug.clone(),
        title: s.title.clone(),
        genre: genre_from_id(&s.genre),
        max_players: s.max_players,
        description: s.description.clone(),
    }
}

/// Parse the `:game/catalog` table from EDN `src` into ordered [`GameDefSpec`]s.
pub fn catalog_specs_from_edn(src: &str) -> Result<Vec<GameDefSpec>, CatalogError> {
    let root = root_map(src).ok_or(CatalogError::NotAMap)?;
    let entries = mget(&root, "game/catalog")
        .and_then(|v| v.as_vector())
        .ok_or(CatalogError::NoCatalog)?;
    Ok(entries
        .iter()
        .filter_map(|e| e.as_map().map(GameDefSpec::from_map))
        .collect())
}

/// Parse the `:game/catalog` table from EDN `src` into the real [`GameDef`] list.
pub fn catalog_from_edn(src: &str) -> Result<Vec<GameDef>, CatalogError> {
    Ok(catalog_specs_from_edn(src)?
        .iter()
        .map(spec_to_game_def)
        .collect())
}

/// The compiled-in oracle: `godot_game_catalog()` projected into [`GameDefSpec`]s.
pub fn builtin_catalog_specs() -> Vec<GameDefSpec> {
    godot_game_catalog()
        .iter()
        .map(GameDefSpec::from_game_def)
        .collect()
}

/// Convenience: the catalog from the crate-shipped [`GAME_CATALOG_EDN`].
pub fn shipped_catalog() -> Result<Vec<GameDef>, CatalogError> {
    catalog_from_edn(GAME_CATALOG_EDN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_count_matches_builtin() {
        let loaded = catalog_specs_from_edn(GAME_CATALOG_EDN).expect("game_catalog.edn parses");
        assert_eq!(loaded.len(), builtin_catalog_specs().len());
        assert_eq!(loaded.len(), 29);
    }

    #[test]
    fn genre_id_round_trips() {
        for g in [
            Genre::IoMultiplayer,
            Genre::Puzzle,
            Genre::Rpg,
            Genre::Simulation,
            Genre::VisualNovel,
            Genre::Card,
            Genre::Arcade,
            Genre::Adult,
            Genre::Brainrot,
            Genre::Chase,
        ] {
            assert_eq!(genre_id(&genre_from_id(genre_id(&g))), genre_id(&g));
        }
    }

    #[test]
    fn non_map_root_is_an_error() {
        assert!(matches!(catalog_specs_from_edn("42"), Err(CatalogError::NotAMap)));
    }

    #[test]
    fn missing_table_is_an_error() {
        assert!(matches!(
            catalog_specs_from_edn("{:other 1}"),
            Err(CatalogError::NoCatalog)
        ));
    }
}
