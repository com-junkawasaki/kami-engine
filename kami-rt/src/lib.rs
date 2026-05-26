//! hikari-rt (光) — WebGPU hardware ray tracing primitive.
//!
//! R1.0 path reservation per ADR-2605261800. No runtime code yet.
//! Concrete implementation begins R1.2 (PSNR ≥ 35dB vs Mitsuba 3 CUDA reference).

pub const ADR: &str = "ADR-2605261800";
pub const PHASE: &str = "R1.0-path-reservation";
pub const KAMI_NAME: &str = "hikari-rt";
pub const NV_COMPAT_TARGET: &str = "OptiX";
