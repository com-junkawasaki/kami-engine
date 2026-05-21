//! Wheel — hub + tire-ring node group with Pacejka magic-formula tire model.
//!
//! Granularity follows BeamNG: a wheel is *not* a rigid disc. It is a small
//! rigid hub (4-6 nodes) plus a flexible tire ring (typically 12-20 nodes
//! arranged on a circle) connected by pressured side-wall beams. Pressure
//! changes the rest length of the side-wall beams, so blowouts and run-flats
//! emerge naturally from the soft body.
//!
//! Steering and drive are NOT applied as forces on the ring nodes (this would
//! deform the wheel). Instead:
//!   * the **steer angle** is fed as `Hydro` extension into the steering tie-rod
//!     beams,
//!   * the **drive torque** spins the hub-rim system as a separate scalar
//!     `angular_velocity`, and
//!   * the **tire patch force** is computed by Pacejka and applied to the hub
//!     centre as a force + counter-torque (decelerates the wheel under load).
//!
//! This split is exactly what BeamNG does and is the reason their tires can
//! squeal, slide, and lock-up under braking without the soft body blowing up.

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::node::NodeId;

/// How a wheel's horizontal Pacejka tire force is delivered into the
/// soft-body node cloud.
///
/// `Hub` (default, classic kami-vehicle behaviour): the longitudinal +
/// lateral force is split 50/50 between the two axle hub nodes. The tire
/// ring nodes — when present — are passive body. Stable, well-tested.
///
/// `TireRing` (Phase 2.5, opt-in via JBeam `tire_nodes`): the force is
/// routed primarily through whichever ring node is currently the contact
/// patch (lowest-Y), with cosine-weighted bleed to its two angular
/// neighbours. A residual share still goes to the axle pair so engine
/// torque and the XPBD tire-vertical constraint keep functioning. Used
/// by vehicles emitted from `kami-cad-import` once their JBeam wheel
/// slots declare the 12-node tire ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WheelContactMode {
    Hub,
    TireRing,
}

impl Default for WheelContactMode {
    fn default() -> Self {
        Self::Hub
    }
}

pub type WheelId = u32;

/// Pacejka 1996 / "Magic Formula" coefficients (simplified — long+lat only).
///
/// Reference shape: `F = D * sin(C * atan(B * slip - E * (B*slip - atan(B*slip))))`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PacejkaParams {
    pub b_long: f32,
    pub c_long: f32,
    pub d_long: f32,
    pub e_long: f32,
    pub b_lat: f32,
    pub c_lat: f32,
    pub d_lat: f32,
    pub e_lat: f32,
}

impl PacejkaParams {
    /// Reasonable starting point for a road tire on dry asphalt.
    pub fn road_dry() -> Self {
        Self {
            b_long: 10.0,
            c_long: 1.65,
            d_long: 1.0,
            e_long: 0.97,
            b_lat: 8.5,
            c_lat: 1.30,
            d_lat: 1.0,
            e_lat: 0.97,
        }
    }

    /// Wet asphalt: ~30% peak grip reduction, smoother fall-off past peak.
    pub fn road_wet() -> Self {
        Self {
            b_long: 8.0,
            c_long: 1.65,
            d_long: 0.70,
            e_long: 0.95,
            b_lat: 7.0,
            c_lat: 1.30,
            d_lat: 0.70,
            e_lat: 0.95,
            ..Self::road_dry()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wheel {
    pub id: WheelId,
    /// Rim / hub nodes (rigid wrt each other).
    pub hub_nodes: Vec<NodeId>,
    /// Tire ring nodes (flexible).
    pub tire_nodes: Vec<NodeId>,
    /// The two nodes that form the steering pivot axis (line through them
    /// = steering / spin axis).
    pub axle_n1: NodeId,
    pub axle_n2: NodeId,
    pub radius: f32,
    pub width: f32,
    /// Spin rate around the axle (rad/s, positive = forward).
    pub angular_velocity: f32,
    /// Steered angle in radians (signed).
    pub steer_angle: f32,
    pub max_steer_angle: f32,
    /// Brake torque magnitude (Nm) — applied opposite to angular velocity.
    pub brake_torque: f32,
    /// Drive torque (Nm) from the powertrain — set by the driveline each step.
    pub drive_torque: f32,
    /// Wheel + brake disc rotational inertia (kg·m²).
    pub spin_inertia: f32,
    /// Internal pressure in bar.
    pub pressure: f32,
    pub reference_pressure: f32,
    pub tire: PacejkaParams,
    /// Whether this wheel is currently on the ground (cached after step).
    pub grounded: bool,
    /// Last computed contact patch slip ratio / slip angle (telemetry).
    pub last_slip_ratio: f32,
    pub last_slip_angle: f32,
    /// How tire force is distributed into the body. Defaults to `Hub`
    /// for backward compatibility; the JBeam loader flips this to
    /// `TireRing` whenever the wheel slot ships a populated tire ring.
    #[serde(default)]
    pub contact_mode: WheelContactMode,
}

impl Wheel {
    pub fn new(id: WheelId, axle_n1: NodeId, axle_n2: NodeId, radius: f32, width: f32) -> Self {
        Self {
            id,
            hub_nodes: Vec::new(),
            tire_nodes: Vec::new(),
            axle_n1,
            axle_n2,
            radius,
            width,
            angular_velocity: 0.0,
            steer_angle: 0.0,
            max_steer_angle: 0.55, // ~31°, typical road car inner wheel
            brake_torque: 0.0,
            drive_torque: 0.0,
            spin_inertia: 1.5,
            pressure: 2.4,
            reference_pressure: 2.4,
            tire: PacejkaParams::road_dry(),
            grounded: false,
            last_slip_ratio: 0.0,
            last_slip_angle: 0.0,
            contact_mode: WheelContactMode::default(),
        }
    }

    /// Set the steering input, clamped to `±max_steer_angle`.
    pub fn set_steer(&mut self, target: f32) {
        self.steer_angle = target.clamp(-self.max_steer_angle, self.max_steer_angle);
    }
}

/// Inputs to the Pacejka tire model at a given contact patch.
#[derive(Debug, Clone, Copy)]
pub struct ContactInputs {
    /// Normal load (N), pressing the tire into the ground.
    pub fz: f32,
    /// Longitudinal velocity of the wheel centre projected onto the wheel
    /// heading axis (m/s).
    pub vx: f32,
    /// Lateral velocity (m/s).
    pub vy: f32,
    /// Spin velocity at the rolling radius (`omega * r`, m/s).
    pub vs: f32,
}

/// Force vector returned by the Pacejka evaluator, in the wheel's heading
/// frame: `+x` = forward (heading), `+y` = lateral (left), `+z` = up (normal).
#[derive(Debug, Clone, Copy, Default)]
pub struct ContactForces {
    pub fx: f32,
    pub fy: f32,
    pub slip_ratio: f32,
    pub slip_angle: f32,
}

/// Evaluate the tire force at the contact patch using a simplified Pacejka
/// magic formula in `(slip_ratio, slip_angle)`.
///
/// This is intentionally one function — keeping the tire model out of the
/// `Wheel` struct lets callers test it in isolation against published Pacejka
/// reference curves.
pub fn pacejka_force(p: &PacejkaParams, c: ContactInputs) -> ContactForces {
    // Slip ratio: (omega*r - v)/max(|v|, eps), clamped for numerical stability.
    let denom = c.vx.abs().max(0.5);
    let slip_ratio = ((c.vs - c.vx) / denom).clamp(-2.0, 2.0);

    // Slip angle: arctan(vy / vx) — undefined at v=0, so blend with a small
    // dead-band. Sign convention: positive slip angle => car is yawing into
    // the right, lateral force should be positive (push left).
    let slip_angle = if c.vx.abs() < 0.5 {
        c.vy.signum() * (c.vy.abs() / 0.5).min(1.0) * 0.20
    } else {
        (c.vy / c.vx.abs()).atan()
    };

    let mu = c.fz.max(0.0);

    let fx = magic_formula(slip_ratio, p.b_long, p.c_long, p.d_long, p.e_long) * mu;
    let fy = magic_formula(slip_angle, p.b_lat, p.c_lat, p.d_lat, p.e_lat) * mu;

    // Friction-circle clamp: total tangential force can't exceed mu * Fz.
    let limit = mu * p.d_long.max(p.d_lat);
    let mag = (fx * fx + fy * fy).sqrt();
    let (fx, fy) = if mag > limit && mag > 1e-3 {
        let s = limit / mag;
        (fx * s, fy * s)
    } else {
        (fx, fy)
    };

    ContactForces {
        fx,
        fy: -fy, // flip to body-frame +Y = left
        slip_ratio,
        slip_angle,
    }
}

#[inline]
fn magic_formula(slip: f32, b: f32, c: f32, d: f32, e: f32) -> f32 {
    let bs = b * slip;
    d * (c * (bs - e * (bs - bs.atan())).atan()).sin()
}

/// Lateral cornering coefficient — useful for callers that want a linear
/// tire model for ESP / stability code without re-running Pacejka.
pub fn cornering_stiffness(p: &PacejkaParams, fz: f32) -> f32 {
    p.b_lat * p.c_lat * p.d_lat * fz
}

/// World-space wheel heading frame.
///
/// Returns `(forward, left, up)` orthonormal basis given the steered yaw and
/// the chassis up vector.
pub fn wheel_frame(chassis_forward: Vec3, chassis_up: Vec3, steer_yaw: f32) -> (Vec3, Vec3, Vec3) {
    let up = chassis_up.normalize_or_zero();
    let cf = chassis_forward.normalize_or_zero();
    // Rotate cf around up by steer_yaw.
    let (s, cs) = steer_yaw.sin_cos();
    let right = cf.cross(up);
    let forward = (cf * cs + right * s).normalize_or_zero();
    let left = up.cross(forward).normalize_or_zero();
    (forward, left, up)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pacejka_zero_slip_zero_force() {
        let p = PacejkaParams::road_dry();
        let f = pacejka_force(
            &p,
            ContactInputs {
                fz: 4000.0,
                vx: 20.0,
                vy: 0.0,
                vs: 20.0,
            },
        );
        assert!(f.fx.abs() < 5.0);
        assert!(f.fy.abs() < 5.0);
    }

    #[test]
    fn pacejka_locked_wheel_pushes_back() {
        let p = PacejkaParams::road_dry();
        let f = pacejka_force(
            &p,
            ContactInputs {
                fz: 4000.0,
                vx: 20.0,
                vy: 0.0,
                vs: 0.0, // wheel locked
            },
        );
        // Slip ratio is -1, so longitudinal force should oppose motion (negative).
        assert!(f.fx < -1000.0);
    }

    #[test]
    fn pacejka_friction_circle_clamps_combined_force() {
        let p = PacejkaParams::road_dry();
        let f = pacejka_force(
            &p,
            ContactInputs {
                fz: 4000.0,
                vx: 5.0,
                vy: 8.0, // huge sideslip
                vs: 0.0, // and locked
            },
        );
        let total = (f.fx * f.fx + f.fy * f.fy).sqrt();
        // Should be clamped to ~mu * Fz = 4000 N (within a small rounding band).
        assert!(total <= 4000.0 * 1.05);
    }

    #[test]
    fn wheel_steer_clamps() {
        let mut w = Wheel::new(0, 0, 1, 0.32, 0.22);
        w.max_steer_angle = 0.4;
        w.set_steer(1.0);
        assert!((w.steer_angle - 0.4).abs() < 1e-6);
        w.set_steer(-1.0);
        assert!((w.steer_angle + 0.4).abs() < 1e-6);
    }

    #[test]
    fn wheel_frame_with_zero_steer_returns_chassis_basis() {
        // forward=+X, up=+Y, right-handed -> left = up × forward = -Z.
        let (f, l, u) = wheel_frame(Vec3::X, Vec3::Y, 0.0);
        assert!((f - Vec3::X).length() < 1e-5);
        assert!((u - Vec3::Y).length() < 1e-5);
        assert!((l - Vec3::NEG_Z).length() < 1e-5);
    }
}
