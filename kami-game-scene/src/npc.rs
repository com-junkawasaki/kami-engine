//! NPC-tuning data tier — kami-game's brainrot NPC tuning (the magic numbers extracted
//! from the hot `tick()` loops into engine tuning structs) as parity-tested EDN.
//!
//! ADR-0046 "needs-refactor" case: `kami_game::npc::SkibidiTuning` (etc.) holds the phase
//! durations + motion rates the per-frame integrator reads; its `Default` reproduces the
//! original hardcoded constants. Here that tuning becomes EDN data. The hot integrator
//! stays native Rust; only the init-time tuning moves to EDN. [`skibidi_tuning_from_edn`]
//! rebuilds the real [`SkibidiTuning`], asserted `==` `SkibidiTuning::default()` in
//! `tests/npc_parity.rs`.

use kami_game::npc::SkibidiTuning;
use kami_scene::{mget, num, root_map};

/// The canonical NPC-tuning CONFIG shipped with this crate.
pub const NPC_TUNING_EDN: &str = include_str!("../data/npc_tuning.edn");

/// Errors raised while loading NPC tuning from EDN.
#[derive(Debug, thiserror::Error)]
pub enum NpcError {
    /// The EDN source did not parse to a top-level map.
    #[error("npc-tuning EDN root is not a map")]
    NotAMap,
    /// The expected tuning key was missing or not a map.
    #[error("`{0}` missing or not a map")]
    NoTable(&'static str),
}

/// Parse `:npc/skibidi` from EDN `src` into the real [`SkibidiTuning`]. Tolerant: a field
/// the EDN omits keeps the engine [`SkibidiTuning::default`], so a partial map merges onto
/// the default.
pub fn skibidi_tuning_from_edn(src: &str) -> Result<SkibidiTuning, NpcError> {
    let root = root_map(src).ok_or(NpcError::NotAMap)?;
    let m = mget(&root, "npc/skibidi")
        .and_then(|v| v.as_map())
        .ok_or(NpcError::NoTable("npc/skibidi"))?;
    let d = SkibidiTuning::default();
    let or = |key: &str, fallback: f32| match mget(m, key) {
        Some(v) => num(Some(v)),
        None => fallback,
    };
    Ok(SkibidiTuning {
        rise_dur: or("rise-dur", d.rise_dur),
        hold_dur: or("hold-dur", d.hold_dur),
        drop_dur: or("drop-dur", d.drop_dur),
        wait_dur: or("wait-dur", d.wait_dur),
        rise_dy_rate: or("rise-dy-rate", d.rise_dy_rate),
        yaw_freq: or("yaw-freq", d.yaw_freq),
        yaw_amp: or("yaw-amp", d.yaw_amp),
        drop_dy_rate: or("drop-dy-rate", d.drop_dy_rate),
    })
}

/// The compiled-in oracle: `SkibidiTuning::default()`.
pub fn builtin_skibidi_tuning() -> SkibidiTuning {
    SkibidiTuning::default()
}

/// Convenience: the Skibidi tuning from the crate-shipped [`NPC_TUNING_EDN`].
pub fn shipped_skibidi_tuning() -> Result<SkibidiTuning, NpcError> {
    skibidi_tuning_from_edn(NPC_TUNING_EDN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_map_root_is_an_error() {
        assert!(matches!(skibidi_tuning_from_edn("42"), Err(NpcError::NotAMap)));
    }

    #[test]
    fn missing_table_is_an_error() {
        assert!(matches!(
            skibidi_tuning_from_edn("{:x 1}"),
            Err(NpcError::NoTable(_))
        ));
    }

    #[test]
    fn omitted_field_inherits_default() {
        let t = skibidi_tuning_from_edn("{:npc/skibidi {:rise-dur 3.0}}").unwrap();
        assert_eq!(t.rise_dur, 3.0);
        assert_eq!(t.yaw_freq, SkibidiTuning::default().yaw_freq, "absent → default");
    }
}
