//! Parity test: the shipped `npc_tuning.edn` must reproduce kami-game's compiled-in NPC
//! tuning defaults (ADR-0046, "needs-refactor" case) — EDN as source of truth, behaviour
//! unchanged (the engine's `Default` is the magic-number constants that used to live in the
//! hot `tick()` loop).
//!
//! The oracle is the REAL Rust: `SkibidiTuning::default()` (via `builtin_skibidi_tuning`).
//! `SkibidiTuning` derives `PartialEq`, so the comparison is direct.

use kami_game_scene::npc::{builtin_skibidi_tuning, shipped_skibidi_tuning};

#[test]
fn skibidi_tuning_edn_matches_builtin() {
    let edn = shipped_skibidi_tuning().expect("npc_tuning.edn parses");
    assert_eq!(
        edn,
        builtin_skibidi_tuning(),
        "skibidi tuning is driven by npc_tuning.edn, == SkibidiTuning::default()"
    );
}
