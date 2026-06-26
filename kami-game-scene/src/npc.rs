//! NPC-tuning data tier — kami-game's brainrot NPC tuning (the magic numbers extracted
//! from the hot `tick()` loops into engine tuning structs) as parity-tested EDN.
//!
//! ADR-0046 "needs-refactor" case: `kami_game::npc::SkibidiTuning` (etc.) holds the phase
//! durations + motion rates the per-frame integrator reads; its `Default` reproduces the
//! original hardcoded constants. Here that tuning becomes EDN data. The hot integrator
//! stays native Rust; only the init-time tuning moves to EDN. [`skibidi_tuning_from_edn`]
//! rebuilds the real [`SkibidiTuning`], asserted `==` `SkibidiTuning::default()` in
//! `tests/npc_parity.rs`.

use std::collections::BTreeMap;

use kami_game::npc::{
    FanumTuning, GrimaceTuning, OhioTuning, RizzTuning, SigmaTuning, SkibidiTuning,
};
use kami_scene::{mget, num, root_map, EdnValue};

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

/// Read the `:npc/<key>` sub-map from EDN `src`.
fn npc_map(src: &str, key: &'static str) -> Result<BTreeMap<EdnValue, EdnValue>, NpcError> {
    let root = root_map(src).ok_or(NpcError::NotAMap)?;
    mget(&root, key)
        .and_then(|v| v.as_map())
        .cloned()
        .ok_or(NpcError::NoTable(key))
}

/// Parse `:npc/grimace` into the real [`GrimaceTuning`] (absent field → engine default).
pub fn grimace_tuning_from_edn(src: &str) -> Result<GrimaceTuning, NpcError> {
    let m = npc_map(src, "npc/grimace")?;
    let d = GrimaceTuning::default();
    let or = |k: &str, f: f32| mget(&m, k).map(|v| num(Some(v))).unwrap_or(f);
    Ok(GrimaceTuning {
        speed: or("speed", d.speed),
        puddle_interval: or("puddle-interval", d.puddle_interval),
        wobble_rate: or("wobble-rate", d.wobble_rate),
        wobble_amp: or("wobble-amp", d.wobble_amp),
    })
}

/// Parse `:npc/sigma` into the real [`SigmaTuning`] (absent field → engine default).
pub fn sigma_tuning_from_edn(src: &str) -> Result<SigmaTuning, NpcError> {
    let m = npc_map(src, "npc/sigma")?;
    let d = SigmaTuning::default();
    let or = |k: &str, f: f32| mget(&m, k).map(|v| num(Some(v))).unwrap_or(f);
    Ok(SigmaTuning {
        nod_trigger_dist: or("nod-trigger-dist", d.nod_trigger_dist),
        nod_duration: or("nod-duration", d.nod_duration),
        nod_pitch_deg: or("nod-pitch-deg", d.nod_pitch_deg),
    })
}

/// Parse `:npc/ohio` into the real [`OhioTuning`] (absent field → engine default).
pub fn ohio_tuning_from_edn(src: &str) -> Result<OhioTuning, NpcError> {
    let m = npc_map(src, "npc/ohio")?;
    let d = OhioTuning::default();
    let or = |k: &str, f: f32| mget(&m, k).map(|v| num(Some(v))).unwrap_or(f);
    Ok(OhioTuning {
        yaw_rate: or("yaw-rate", d.yaw_rate),
        teleport_interval: or("teleport-interval", d.teleport_interval),
        teleport_radius: or("teleport-radius", d.teleport_radius),
        damage_cube_dist: or("damage-cube-dist", d.damage_cube_dist),
    })
}

/// Parse `:npc/fanum` into the real [`FanumTuning`] (absent field → engine default).
pub fn fanum_tuning_from_edn(src: &str) -> Result<FanumTuning, NpcError> {
    let m = npc_map(src, "npc/fanum")?;
    let d = FanumTuning::default();
    let or = |k: &str, f: f32| mget(&m, k).map(|v| num(Some(v))).unwrap_or(f);
    Ok(FanumTuning {
        steal_cooldown: or("steal-cooldown", d.steal_cooldown),
        arrive_dist: or("arrive-dist", d.arrive_dist),
        patrol_speed: or("patrol-speed", d.patrol_speed),
    })
}

/// Parse `:npc/rizz` into the real [`RizzTuning`] (absent field → engine default).
pub fn rizz_tuning_from_edn(src: &str) -> Result<RizzTuning, NpcError> {
    let m = npc_map(src, "npc/rizz")?;
    let d = RizzTuning::default();
    let or = |k: &str, f: f32| mget(&m, k).map(|v| num(Some(v))).unwrap_or(f);
    Ok(RizzTuning {
        approach_dist: or("approach-dist", d.approach_dist),
        approach_speed: or("approach-speed", d.approach_speed),
        charm_pitch_deg: or("charm-pitch-deg", d.charm_pitch_deg),
        charm_duration: or("charm-duration", d.charm_duration),
        walkaway_radius: or("walkaway-radius", d.walkaway_radius),
        walkaway_speed: or("walkaway-speed", d.walkaway_speed),
        walkaway_arrive_dist: or("walkaway-arrive-dist", d.walkaway_arrive_dist),
    })
}

/// Convenience loaders for the remaining brainrot npcs from the shipped EDN.
pub fn shipped_grimace_tuning() -> Result<GrimaceTuning, NpcError> {
    grimace_tuning_from_edn(NPC_TUNING_EDN)
}
pub fn shipped_sigma_tuning() -> Result<SigmaTuning, NpcError> {
    sigma_tuning_from_edn(NPC_TUNING_EDN)
}
pub fn shipped_ohio_tuning() -> Result<OhioTuning, NpcError> {
    ohio_tuning_from_edn(NPC_TUNING_EDN)
}
pub fn shipped_fanum_tuning() -> Result<FanumTuning, NpcError> {
    fanum_tuning_from_edn(NPC_TUNING_EDN)
}
pub fn shipped_rizz_tuning() -> Result<RizzTuning, NpcError> {
    rizz_tuning_from_edn(NPC_TUNING_EDN)
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
