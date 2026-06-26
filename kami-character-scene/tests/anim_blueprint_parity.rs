//! Parity test: the shipped `anim_blueprint.edn` must rebuild kami-character's
//! compiled-in `AnimBlueprint::metahuman_default()` component-for-component (ADR-0046) —
//! EDN as source of truth, behaviour unchanged.
//!
//! The oracle is the REAL Rust: `AnimBlueprint::metahuman_default()` (called here, not
//! transcribed). `AnimBlueprint` derives no `PartialEq`, but every component
//! (`AnimParam` / `AnimLayer` / `BlendProfile`) derives `Serialize`, so the rebuilt
//! blueprint's parameters, layers and blend-profiles are compared structurally via
//! `serde_json` (object/map comparison is order-independent — safe for the param map).

use kami_character::anim_blueprint::AnimBlueprint;
use kami_character_scene::anim_blueprint::shipped_blueprint;

#[test]
fn anim_blueprint_edn_matches_builtin() {
    let oracle = AnimBlueprint::metahuman_default();
    let edn = shipped_blueprint().expect("shipped blueprint builds");

    // Parameters (HashMap<String, AnimParam>) — order-independent map equality.
    assert_eq!(
        serde_json::to_value(&edn.parameters).unwrap(),
        serde_json::to_value(&oracle.parameters).unwrap(),
        "parameters parity (name/type/value/default)"
    );

    // Layers (Vec<AnimLayer>) — states, blend spaces, transitions, conditions, runtime
    // fields (active_state / transition_progress / transition_target).
    assert_eq!(
        serde_json::to_value(&edn.layers).unwrap(),
        serde_json::to_value(&oracle.layers).unwrap(),
        "layers parity (states + transitions)"
    );

    // Blend profiles (Vec<BlendProfile>) — per-bone weight maps.
    assert_eq!(
        serde_json::to_value(&edn.blend_profiles).unwrap(),
        serde_json::to_value(&oracle.blend_profiles).unwrap(),
        "blend-profiles parity"
    );
}

/// Behaviour check: the EDN-built blueprint runs the same idle→locomotion transition as
/// the builtin under the same `is_moving` parameter. Identical data ⇒ identical FSM.
#[test]
fn rebuilt_blueprint_transitions_like_builtin() {
    let mut edn = shipped_blueprint().unwrap();
    let mut oracle = AnimBlueprint::metahuman_default();
    for bp in [&mut edn, &mut oracle] {
        assert_eq!(bp.layers[0].active_state, 0, "starts at idle");
        bp.set_param("is_moving", 1.0);
        for _ in 0..40 {
            bp.update(0.016);
        }
        assert_eq!(bp.layers[0].active_state, 1, "transitions to locomotion");
    }
}
