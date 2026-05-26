//! kami-usd-native — from-scratch Rust USD parser + composition engine.
//!
//! R1.0 path reservation per ADR-2605261800 §D10.4.
//! **Contingent fallback** — activated only if tinyusdz WASM fails R1.1
//! viability gate (iPhone 12+ 10MB USD parse ≤2s). Activation requires
//! Council Lv6+ ≥3 attestation per §D10.2.

pub const ADR: &str = "ADR-2605261800";
pub const PHASE: &str = "R1.0-path-reservation";
pub const KAMI_NAME: &str = "kami-usd-native";
pub const STATUS: &str = "contingent-fallback-pending-viability-gate";
pub const TRIGGERED_BY: &str = "tinyusdz WASM gate fail (R1.1)";
pub const NV_COMPAT_TARGETS: &[&str] = &["omni.usd", "pxr.Usd", "pxr.UsdGeom"];

pub const SUPPORTED_FORMATS: &[&str] = &["usda", "usdc", "usdz"];
