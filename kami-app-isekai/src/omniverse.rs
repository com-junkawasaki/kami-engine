//! kami-app-isekai · Omniverse / PhysX / OpenUSD facade entry.
//!
//! This module wires the kami-engine nv-compat layer (`kami-usd`,
//! `kami-genesis`, `kami-articulated`) into the ISEKAI runtime so a USDA
//! scene description can drive both the visual layer (voxel sandbox +
//! sky + terrain) and the physics layer (PhysX-shaped `World` with
//! reduced-coordinate articulations).
//!
//! Constitutional invariants (ADR-2605261800 §D10.3 + §G7 of
//! ADR-2605262500): all NVIDIA-branded APIs are accessed exclusively
//! through the `kami-*` facade namespace. NO direct PhysX / OmniKit /
//! OpenUSD library imports.
//!
//! ```text
//!   USDA source ──► kami-usd::parse_usda ──► Stage
//!   Stage  ┐
//!          ├── PhysicsScene  ──► kami-genesis::World (PxScene-shaped)
//!          ├── Cartpole prim ──► kami-articulated::parse_urdf
//!          │                    ─► kami-genesis::Articulation
//!          │                       (PxArticulationReducedCoordinate)
//!          ├── Cube / Sphere / Plane prims ──► kami-pipelines voxels
//!          └── Xform                     ──► scene-graph transforms
//! ```

use kami_genesis::{ArticulationHandle, World as GenesisWorld};
use kami_usd::{PrimKind, Stage};

#[cfg(target_family = "wasm")]
use kami_app::{CameraMode, InputMode, KamiApp, Position};
#[cfg(target_family = "wasm")]
use log::Level;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

// Bundled URDF — same physical cartpole used by `kami-cartpole-wasm` so
// trained policies trained against either entry remain valid.
const BUNDLED_CARTPOLE_URDF: &str = include_str!(
    "../../../../70-tools/e7m-sim/scenes/cartpole/cartpole.urdf"
);

/// Built-in USDA used when the JS side does not supply a custom one.
/// One PhysicsScene + a ground plane + one Cartpole articulation that
/// spawns above the demo house at the same world coordinates as the
/// v3-demos paper-row so the camera framing matches the existing scenes.
pub const DEFAULT_ISEKAI_USDA: &str = r#"#usda 1.0
(
    upAxis = "Y"
    metersPerUnit = 1.0
)

def PhysicsScene "physics"
{
    vector3f physics:gravityDirection = (0, -1, 0)
    float physics:gravityMagnitude = 9.81
}

def Plane "ground"
{
    double3 xformOp:translate = (0, 0, 0)
    double width = 32.0
    double length = 32.0
}

def Cartpole "cart_alpha"
{
    double3 xformOp:translate = (-11, 33.5, 18)
    custom string urdf = "@./cartpole.urdf@"
}
"#;

/// nv-compat banner; useful for HUD strings and audit trails.
#[cfg(target_family = "wasm")]
#[wasm_bindgen(js_name = isekaiOmniverseBanner)]
pub fn isekai_omniverse_banner() -> String {
    format!(
        "kami-usd@{} (omni.usd compat) + kami-genesis@{} (PhysX 5 / isaacsim.core.api compat) — {}",
        kami_usd::PHASE,
        kami_genesis::PHASE,
        kami_usd::ADR
    )
}

/// JS-callable: return the bundled default USDA so the JS side can
/// display / edit / re-submit it.
#[cfg(target_family = "wasm")]
#[wasm_bindgen(js_name = isekaiOmniverseDefaultUsda)]
pub fn isekai_omniverse_default_usda() -> String {
    DEFAULT_ISEKAI_USDA.to_string()
}

/// Run the ISEKAI omniverse entry.
///
/// `canvas_id`  — WebGPU canvas DOM id.
/// `usda_src`   — USDA stage text. Pass empty string to use
///                `DEFAULT_ISEKAI_USDA`.
#[cfg(target_family = "wasm")]
#[wasm_bindgen(js_name = runIsekaiOmniverse)]
pub async fn run_isekai_omniverse(
    canvas_id: &str,
    usda_src: &str,
) -> Result<(), JsValue> {
    use crate::pipelines;
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(Level::Info);

    let stage = parse_or_default(usda_src);
    log::info!(
        "[isekai-omniverse] stage prims={} up_axis={:?} mpu={}",
        stage.prims.len(),
        stage.up_axis,
        stage.meters_per_unit
    );

    // Build PhysX-shaped World from PhysicsScene prim (or default 9.81 -Y).
    let mut world = build_world_from_stage(&stage);
    let articulations: Vec<(String, ArticulationHandle, [f32; 3])> =
        spawn_articulations(&mut world, &stage).map_err(|e| JsValue::from_str(&e))?;

    let spawn = Position::new(-5.0, 35.5, 18.5);

    let app = KamiApp::new_web(canvas_id)
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .with_label("isekai-omniverse")
        .with_hud_publish(true)
        .with_camera(CameraMode::FirstPerson {
            spawn,
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: -0.25,
        })
        .with_input(InputMode::WasdFps);

    // Visual layer: re-use the same sky + streaming terrain + atlas
    // adapters the v3-demos scene already validates. Voxel world gives
    // the cartpole something to land on.
    let sky = pipelines::SkyAdapter::new(app.render_context());
    let terrain = pipelines::TerrainAdapter::streaming(
        app.render_context(),
        kami_terrain::BiomePreset::Plains,
        42.0,
        128,
        2,
    );
    let voxels = crate::build_voxel_world(app.render_context());
    let voxels_for_probe = voxels.clone();
    let voxels_for_wall = voxels.clone();
    crate::build_demo_house(app.render_context(), &voxels);

    // Atlas sprite layer renders the cart + pole tip so we don't need a
    // separate articulated-body mesh pipeline for the R0 cut. Pole is a
    // bobbing sparkle; cart is a flame_medium with neutral tint.
    let atlas = kami_pipelines::AtlasVisAdapter::new(app.render_context(), 256);
    let atlas_for_tick = atlas.clone();

    // Mutable physics handle moved into the closure. World owns the
    // articulations; we read state each frame and emit atlas sprites at
    // the resolved cart + pole world positions.
    let mut tick: u64 = 0;
    let mut world_cell = std::cell::RefCell::new(world);
    // `move` captures world_cell + articulations.
    let articulations_for_tick = articulations.clone();
    let app = app
        .with_floor_probe(move |p| voxels_for_probe.sample_floor(p))
        .with_eye_height(1.8)
        .with_collider_probe(move |min, max| voxels_for_wall.aabb_solid(min, max))
        .with_player_radius(0.35)
        .with_gravity(0.0)
        .with_jump_impulse(0.0)
        .with_pipeline(sky)
        .with_pipeline(terrain)
        .with_pipeline(voxels)
        .with_pipeline(atlas)
        .on_update(move |_world_ecs, _camera, _dt| {
            tick = tick.wrapping_add(1);

            // PhysX-style World::step() per frame. R0: α=0 step — physics
            // runs but force is left at 0 unless a JS-side controller
            // injects via the (future) controller bridge.
            let mut w = world_cell.borrow_mut();
            w.step();

            // For each articulation, read the cartpole state and emit a
            // pair of atlas sprites at (cart, pole_tip) world positions.
            for (name, handle, origin) in &articulations_for_tick {
                let cart_x_off = w
                    .get(*handle)
                    .ok()
                    .and_then(|a| a.cartpole_state())
                    .map(|s| s.x)
                    .unwrap_or(0.0);
                let pole_theta = w
                    .get(*handle)
                    .ok()
                    .and_then(|a| a.cartpole_state())
                    .map(|s| s.theta)
                    .unwrap_or(0.0);

                let cart_pos = glam::Vec3::new(
                    origin[0] + cart_x_off,
                    origin[1] + 0.5,
                    origin[2],
                );
                // Pole tip at length 0.5 m around revolute joint above cart.
                let pole_tip = cart_pos
                    + glam::Vec3::new(
                        pole_theta.sin() * 0.5,
                        pole_theta.cos() * 0.5,
                        0.0,
                    );

                atlas_for_tick.emit_static(
                    cart_pos,
                    kami_pipelines::atlas_slot::FLAME_MEDIUM,
                    [0.6, 0.6, 0.95],
                    1.4,
                    0.18,
                );
                atlas_for_tick.emit_bobbing(
                    pole_tip,
                    kami_pipelines::atlas_slot::SPARKLE_STAR,
                    [1.0, 0.95, 0.55],
                    0.9,
                    0.22,
                    tick as f32 * 0.4,
                );

                if tick % 120 == 0 {
                    log::info!(
                        "[isekai-omniverse] articulation `{}` x={:.3} theta={:.3}",
                        name,
                        cart_x_off,
                        pole_theta
                    );
                }
            }
        });

    log::info!(
        "[isekai-omniverse] backend={:?} banner=`{}`",
        app.backend(),
        format!(
            "kami-usd@{} kami-genesis@{}",
            kami_usd::PHASE,
            kami_genesis::PHASE
        )
    );
    app.run().await.map_err(|e| JsValue::from_str(&e.to_string()))
}

fn parse_or_default(usda_src: &str) -> Stage {
    let trimmed = usda_src.trim();
    let src = if trimmed.is_empty() {
        DEFAULT_ISEKAI_USDA
    } else {
        trimmed
    };
    match kami_usd::parse_usda(src) {
        Ok(s) => s,
        Err(e) => {
            log::warn!(
                "[isekai-omniverse] USDA parse failed ({}); falling back to default stage",
                e
            );
            kami_usd::parse_usda(DEFAULT_ISEKAI_USDA).expect("default USDA must parse")
        }
    }
}

fn build_world_from_stage(stage: &Stage) -> GenesisWorld {
    let (gravity, dt) = stage
        .prims
        .iter()
        .find_map(|p| {
            if let PrimKind::PhysicsScene { gravity } = p.kind {
                // Use Y-axis component as scalar magnitude (kami-genesis World
                // models gravity as a scalar along -Y; full vector is a future
                // R2 extension once non-Y up-axis stages land).
                Some((gravity[1].abs(), 1.0 / 60.0))
            } else {
                None
            }
        })
        .unwrap_or((9.81, 1.0 / 60.0));
    GenesisWorld::new(gravity, dt)
}

/// Walk the stage, materialise every `Cartpole` prim as a kami-genesis
/// articulation (PhysX `PxArticulationReducedCoordinate` shape) and
/// return per-articulation (path, handle, world-origin) triples.
fn spawn_articulations(
    world: &mut GenesisWorld,
    stage: &Stage,
) -> Result<Vec<(String, ArticulationHandle, [f32; 3])>, String> {
    let mut out = Vec::new();
    for prim in &stage.prims {
        if let PrimKind::Cartpole { .. } = &prim.kind {
            // R0: always use the bundled cartpole URDF, regardless of
            // the `urdf` attr value. R1 will fetch via substrate so the
            // USDA truly authorises the URDF binding.
            let sys = kami_articulated::parse_urdf(BUNDLED_CARTPOLE_URDF)
                .map_err(|e| format!("parse_urdf: {e}"))?;
            let handle = world
                .add_articulation(sys)
                .map_err(|e| format!("add_articulation: {e}"))?;
            log::info!(
                "[isekai-omniverse] spawned articulation `{}` at {:?}",
                prim.path,
                prim.xform.translate
            );
            out.push((prim.path.clone(), handle, prim.xform.translate));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_usda_parses_with_one_cartpole() {
        let st = kami_usd::parse_usda(DEFAULT_ISEKAI_USDA).expect("parse");
        let carts: Vec<_> = st
            .prims
            .iter()
            .filter(|p| matches!(p.kind, PrimKind::Cartpole { .. }))
            .collect();
        assert_eq!(carts.len(), 1);
    }

    #[test]
    fn build_world_reads_gravity_from_stage() {
        let st = kami_usd::parse_usda(DEFAULT_ISEKAI_USDA).expect("parse");
        let w = build_world_from_stage(&st);
        assert!((w.gravity - 9.81).abs() < 1e-3);
    }
}
