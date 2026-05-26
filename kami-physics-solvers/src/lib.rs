//! kami-physics-solvers — 5-solver physics engine, from-scratch WGSL compute.
//!
//! R1.0 path reservation per ADR-2605261800 §D10.4.
//! **Contingent fallback** — activated only if Genesis WebGPU viability gate
//! fails at R1.1 (rigid) or R1.8 (MPM/SPH/FEM/PBD). Activation requires
//! Council Lv6+ ≥3 attestation per §D10.2.
//!
//! Once activated, this crate replaces kami-genesis as the physics backend
//! WITHOUT changing the nv-compat API facade (§D10.3 invariant).

pub const ADR: &str = "ADR-2605261800";
pub const PHASE: &str = "R1.0-path-reservation";
pub const KAMI_NAME: &str = "kami-physics-solvers";
pub const STATUS: &str = "contingent-fallback-pending-viability-gate";
pub const TRIGGERED_BY: &str = "Genesis WebGPU gate fail (R1.1 rigid OR R1.8 MPM/SPH/FEM/PBD)";
pub const NV_COMPAT_TARGETS: &[&str] = &["isaacsim.core.api", "PhysX"];

pub const SOLVERS: &[&str] = &["rigid", "mpm", "sph", "fem", "pbd"];
