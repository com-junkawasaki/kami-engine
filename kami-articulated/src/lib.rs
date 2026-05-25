//! kami-articulated — URDF / MJCF / USD physics loader → kami-genesis articulation.
//!
//! R1.0 path reservation per ADR-2605261800. No runtime code yet.

pub const ADR: &str = "ADR-2605261800";
pub const PHASE: &str = "R1.0-path-reservation";
pub const KAMI_NAME: &str = "kami-articulated";
pub const NV_COMPAT_TARGET: &str = "isaacsim.core.prims.Articulation";
pub const SUPPORTED_FORMATS: &[&str] = &["urdf", "mjcf", "usd-physics"];
