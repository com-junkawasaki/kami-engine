//! Parity test: the shipped `brainrot_evolution.edn` must reproduce kami-game's
//! compiled-in `island_gen::brainrot_evolution_chains()` chain-for-chain, stage-for-stage,
//! IN ORDER (ADR-0046) — EDN as source of truth, behaviour unchanged.
//!
//! The oracle is the REAL Rust: `brainrot_evolution_chains()` (via `builtin_chain_specs`,
//! not transcribed). `BrainrotEvolution` / `EvolutionStage` derive no `PartialEq`, so both
//! sides are projected into the `PartialEq` mirrors and compared as ordered vectors.

use kami_game_scene::brainrot::{
    builtin_chain_specs, chain_specs_from_edn, spec_to_chain, EvolutionChainSpec,
    BRAINROT_EVOLUTION_EDN,
};

#[test]
fn brainrot_edn_matches_builtin() {
    let loaded = chain_specs_from_edn(BRAINROT_EVOLUTION_EDN).expect("brainrot_evolution.edn parses");
    let builtin = builtin_chain_specs();

    assert_eq!(loaded.len(), builtin.len(), "chain count");
    assert_eq!(loaded.len(), 6, "all 6 chains present");

    for (i, (g, w)) in loaded.iter().zip(builtin.iter()).enumerate() {
        assert_eq!(g.character_id, w.character_id, "chain[{i}] character_id");
        assert_eq!(g.character, w.character, "chain[{i}] character");
        assert_eq!(g.stages.len(), w.stages.len(), "chain[{i}] ({}) stage count", w.character_id);
        for (j, (gs, ws)) in g.stages.iter().zip(w.stages.iter()).enumerate() {
            assert_eq!(gs, ws, "chain[{i}].stage[{j}] (gates/scale/overrides)");
        }
        assert_eq!(g, w, "chain[{i}] full parity");
    }

    // Whole ordered list equality.
    assert_eq!(loaded, builtin, "full brainrot-evolution parity (ordered)");
}

/// `spec_to_chain` reconstructs the real `BrainrotEvolution` whose re-projected spec
/// equals the oracle's.
#[test]
fn spec_round_trips_through_chain() {
    let loaded = chain_specs_from_edn(BRAINROT_EVOLUTION_EDN).unwrap();
    let builtin = builtin_chain_specs();
    for (spec, want) in loaded.iter().zip(builtin.iter()) {
        let chain = spec_to_chain(spec);
        assert_eq!(
            EvolutionChainSpec::from_chain(&chain),
            *want,
            "{}: BrainrotEvolution round-trips through spec",
            want.character_id
        );
    }
}
