//! kami-rtx-native — from-scratch WGSL path tracer + differentiable rendering.
//!
//! R1.0 path reservation per ADR-2605261800 §D10.4.
//! **Contingent fallback** — activated only if Mitsuba 3 wgpu upstream PR
//! fails R1.2 viability gate (Cornell box 30 fps Chrome 121+, PSNR ≥35dB).
//! Activation requires Council Lv6+ ≥3 attestation per §D10.2.
//!
//! Built on top of kami-rt (WGSL ray-query + LBVH).

pub const ADR: &str = "ADR-2605261800";
pub const PHASE: &str = "R1.0-path-reservation";
pub const KAMI_NAME: &str = "kami-rtx-native";
pub const STATUS: &str = "contingent-fallback-pending-viability-gate";
pub const TRIGGERED_BY: &str = "Mitsuba 3 wgpu backend gate fail (R1.2)";
pub const NV_COMPAT_TARGETS: &[&str] = &["OptiX", "RTX Renderer"];
