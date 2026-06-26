//! Parity test: the shipped `npc_tuning.edn` must reproduce kami-game's compiled-in NPC
//! tuning defaults (ADR-0046, "needs-refactor" case) — EDN as source of truth, behaviour
//! unchanged (the engine's `Default` is the magic-number constants that used to live in the
//! hot `tick()` loop).
//!
//! The oracle is the REAL Rust: `SkibidiTuning::default()` (via `builtin_skibidi_tuning`).
//! `SkibidiTuning` derives `PartialEq`, so the comparison is direct.

use kami_game::npc::{
    FanumTuning, GrimaceTuning, OhioTuning, RizzTuning, SigmaTuning, SkibidiTuning,
};
use kami_game_scene::npc::{
    shipped_fanum_tuning, shipped_grimace_tuning, shipped_ohio_tuning, shipped_rizz_tuning,
    shipped_sigma_tuning, shipped_skibidi_tuning,
};

/// Every brainrot npc's shipped EDN tuning == the engine `*Tuning::default()` (the
/// constants that used to live in the hot `tick()` loop). PartialEq, exact.
#[test]
fn npc_tuning_edn_matches_builtin() {
    assert_eq!(shipped_skibidi_tuning().unwrap(), SkibidiTuning::default(), "skibidi");
    assert_eq!(shipped_grimace_tuning().unwrap(), GrimaceTuning::default(), "grimace");
    assert_eq!(shipped_sigma_tuning().unwrap(), SigmaTuning::default(), "sigma");
    assert_eq!(shipped_ohio_tuning().unwrap(), OhioTuning::default(), "ohio");
    assert_eq!(shipped_fanum_tuning().unwrap(), FanumTuning::default(), "fanum");
    assert_eq!(shipped_rizz_tuning().unwrap(), RizzTuning::default(), "rizz");
}
