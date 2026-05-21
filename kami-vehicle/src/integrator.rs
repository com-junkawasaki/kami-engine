//! Integrator — drives the soft body forward in time.
//!
//! BeamNG runs the soft body at 2000 Hz internally and renders at 60+ Hz. We
//! follow the same pattern: each call to `Integrator::step(dt)` is one render
//! tick and internally substeps to keep `internal_dt <= max_dt` (default
//! 0.5 ms). We use semi-implicit Euler — cheap, stable for stiff springs, and
//! conservative when paired with high stiffness.
//!
//! Per substep, the order is:
//!   1. Reset per-node forces.
//!   2. Apply gravity, drag, and per-node misc forces.
//!   3. Apply beam forces (and update plastic deformation).
//!   4. Apply ground contact + tire forces.
//!   5. Integrate `v += F/m * dt`, then `x += v * dt`.
//!   6. Update wheel angular velocities from drive / brake / patch torques.

use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct IntegratorConfig {
    pub gravity: Vec3,
    /// Maximum allowed internal substep duration (s). 5e-4 = 2000 Hz.
    pub max_dt: f32,
    /// Maximum substeps per `step()` call (safety against pathological dt).
    pub max_substeps: u32,
}

impl Default for IntegratorConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            max_dt: 5e-4, // 2 kHz
            max_substeps: 64,
        }
    }
}

/// Compute how many substeps fit into `dt` while staying within `max_dt`.
pub fn substep_count(dt: f32, cfg: &IntegratorConfig) -> (u32, f32) {
    if dt <= 0.0 {
        return (0, 0.0);
    }
    let n = (dt / cfg.max_dt).ceil() as u32;
    let n = n.clamp(1, cfg.max_substeps);
    (n, dt / n as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substeps_at_60hz_use_about_34_steps() {
        let cfg = IntegratorConfig::default();
        let (n, sub) = substep_count(1.0 / 60.0, &cfg);
        assert!(n >= 30 && n <= 64);
        assert!(sub <= cfg.max_dt + 1e-9);
    }

    #[test]
    fn substep_count_clamps_at_max() {
        let cfg = IntegratorConfig::default();
        // Pathological huge dt -> capped.
        let (n, _) = substep_count(10.0, &cfg);
        assert_eq!(n, cfg.max_substeps);
    }

    #[test]
    fn substep_count_zero_dt_yields_zero_steps() {
        let cfg = IntegratorConfig::default();
        let (n, _) = substep_count(0.0, &cfg);
        assert_eq!(n, 0);
    }
}
