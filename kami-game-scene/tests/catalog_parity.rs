//! Parity test: the shipped `game_catalog.edn` must reproduce kami-game's compiled-in
//! `island_gen::godot_game_catalog()` entry-for-entry, IN ORDER (ADR-0046) — EDN as
//! source of truth, behaviour unchanged.
//!
//! The oracle is the REAL Rust: `godot_game_catalog()` (called here via
//! `builtin_catalog_specs`, not transcribed). `GameDef` / `Genre` derive no `PartialEq`,
//! so both sides are projected into the `PartialEq` mirror `GameDefSpec` and compared as
//! ordered `Vec<GameDefSpec>`.

use kami_game_scene::catalog::{
    builtin_catalog_specs, catalog_specs_from_edn, spec_to_game_def, GAME_CATALOG_EDN,
};

#[test]
fn catalog_edn_matches_builtin() {
    let loaded = catalog_specs_from_edn(GAME_CATALOG_EDN).expect("game_catalog.edn parses");
    let builtin = builtin_catalog_specs();

    assert_eq!(loaded.len(), builtin.len(), "entry count");
    assert_eq!(loaded.len(), 29, "all 29 games present in EDN");

    for (i, (g, w)) in loaded.iter().zip(builtin.iter()).enumerate() {
        assert_eq!(g, w, "entry[{i}] ({}) — slug/title/genre/max-players/description", w.slug);
    }

    // Whole ordered list equality (exact String / u32 / genre-id).
    assert_eq!(loaded, builtin, "full catalog parity (ordered)");
}

/// `spec_to_game_def` reconstructs the real `GameDef` whose fields equal the oracle's.
#[test]
fn spec_round_trips_through_game_def() {
    let loaded = catalog_specs_from_edn(GAME_CATALOG_EDN).unwrap();
    let builtin = builtin_catalog_specs();
    for (spec, want) in loaded.iter().zip(builtin.iter()) {
        let g = spec_to_game_def(spec);
        // Re-project the rebuilt GameDef and compare to the oracle spec.
        assert_eq!(
            kami_game_scene::catalog::GameDefSpec::from_game_def(&g),
            *want,
            "{}: GameDef round-trips through GameDefSpec",
            want.slug
        );
    }
}
