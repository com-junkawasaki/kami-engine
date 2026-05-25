//! kami-sensor-sim — camera / lidar / IMU / contact sensor synth.
//!
//! R1.0 path reservation per ADR-2605261800.

pub const ADR: &str = "ADR-2605261800";
pub const PHASE: &str = "R1.0-path-reservation";
pub const KAMI_NAME: &str = "kami-sensor-sim";
pub const NV_COMPAT_TARGET: &str = "isaacsim.sensors";
pub const SUPPORTED_SENSORS: &[&str] = &["camera", "lidar", "imu", "contact"];
