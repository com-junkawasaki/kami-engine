//! kami-rtx (kami-pbrt) — Mitsuba 3 WebGPU bind, RTX Renderer API-compat target.
//!
//! R1.0 path reservation per ADR-2605261800. No runtime code yet.
//! Upstream-only strategy: Mitsuba 3 Dr.Jit → wgpu backend is contributed via
//! upstream PR (no religious-corp fork; 90-day hold then re-evaluate per ADR §D3).

pub const ADR: &str = "ADR-2605261800";
pub const PHASE: &str = "R1.0-path-reservation";
pub const KAMI_NAME: &str = "kami-rtx";
pub const NV_COMPAT_TARGET: &str = "RTX Renderer";
pub const UPSTREAM_REPO: &str = "mitsuba-renderer/mitsuba3";
pub const FORK_POLICY: &str = "upstream-only-90-day-hold";
