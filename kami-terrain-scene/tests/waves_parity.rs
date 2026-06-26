//! Parity test: the shipped `waves.edn` must reproduce kami-terrain's compiled-in
//! `water::default_waves()` wave-for-wave, IN ORDER (ADR-0046) — EDN as source of truth,
//! behaviour unchanged.
//!
//! The oracle is the REAL Rust: `default_waves()` (via `builtin_wave_specs`, not
//! transcribed). `GerstnerWave` is `Pod`/`Zeroable` (no `PartialEq`), so both sides are
//! projected into the `PartialEq` `GerstnerWaveSpec` mirror and compared as ordered
//! vectors. Wave values are exact decimal literals — exact `==`.

use kami_terrain_scene::waves::{
    builtin_wave_specs, spec_to_wave, wave_specs_from_edn, GerstnerWaveSpec, WAVES_EDN,
};

#[test]
fn waves_edn_matches_builtin() {
    let loaded = wave_specs_from_edn(WAVES_EDN).expect("waves.edn parses");
    let builtin = builtin_wave_specs();

    assert_eq!(loaded.len(), builtin.len(), "wave count");
    assert_eq!(loaded.len(), 4, "all 4 waves present");

    for (i, (g, w)) in loaded.iter().zip(builtin.iter()).enumerate() {
        assert_eq!(g, w, "wave[{i}] (direction/amplitude/wavelength/speed/steepness)");
    }
    assert_eq!(loaded, builtin, "full waves parity (ordered)");
}

/// `spec_to_wave` reconstructs the real `GerstnerWave` whose re-projected spec equals the
/// oracle's, with `_pad` zeroed.
#[test]
fn spec_round_trips_through_wave() {
    let loaded = wave_specs_from_edn(WAVES_EDN).unwrap();
    let builtin = builtin_wave_specs();
    for (spec, want) in loaded.iter().zip(builtin.iter()) {
        let wave = spec_to_wave(spec);
        assert_eq!(wave._pad, [0.0; 2], "_pad reconstructed as zero");
        assert_eq!(GerstnerWaveSpec::from_wave(&wave), *want, "wave round-trips through spec");
    }
}
