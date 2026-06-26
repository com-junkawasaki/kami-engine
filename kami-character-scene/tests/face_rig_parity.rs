//! Parity test: the shipped `face_rig.edn` must rebuild kami-character's compiled-in
//! MetaHuman FACS face rig node-for-node (ADR-0046) — EDN as source of truth, behaviour
//! unchanged.
//!
//! The oracle is the REAL Rust: `ControlRig::metahuman_face_rig()` (called here, not
//! transcribed). `RigNode` / `RigNodeType` derive no `PartialEq` but do derive
//! `Serialize`, so the two node graphs are compared structurally via `serde_json`.

use kami_character::control_rig::ControlRig;
use kami_character_scene::face_rig::shipped_face_rig;

/// Every node the EDN rebuilds equals the corresponding node in `metahuman_face_rig()`.
#[test]
fn face_rig_edn_matches_builtin_nodes() {
    let oracle = ControlRig::metahuman_face_rig();
    let edn = shipped_face_rig().expect("shipped face rig builds");

    assert_eq!(edn.nodes.len(), oracle.nodes.len(), "node count parity");
    assert_eq!(edn.eval_order, oracle.eval_order, "eval order parity");

    // RigNode has no PartialEq but derives Serialize → compare structurally via JSON.
    let edn_json = serde_json::to_value(&edn.nodes).expect("serialize edn nodes");
    let oracle_json = serde_json::to_value(&oracle.nodes).expect("serialize oracle nodes");
    assert_eq!(
        edn_json, oracle_json,
        "face-rig nodes are node-for-node identical to metahuman_face_rig()"
    );
}

/// Behaviour check: the EDN-built rig drives the same bones as the builtin under the
/// same FACS controls (smile + jaw). Identical graphs ⇒ identical evaluation.
#[test]
fn rebuilt_rig_evaluates_like_builtin() {
    let mut edn = shipped_face_rig().unwrap();
    let mut oracle = ControlRig::metahuman_face_rig();
    for r in [&mut edn, &mut oracle] {
        r.set_control("AU12_L", 0.8);
        r.set_control("AU12_R", 0.8);
        r.set_control("AU26", 0.3);
        r.evaluate();
    }
    for bone in [34usize, 35, 8] {
        assert_eq!(
            edn.bone_outputs.contains_key(&bone),
            oracle.bone_outputs.contains_key(&bone),
            "bone {bone} driven by both rigs"
        );
        assert!(edn.bone_outputs.contains_key(&bone), "bone {bone} present");
    }
}
