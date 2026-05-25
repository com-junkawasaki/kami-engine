//! kami-genesis — Genesis physics backend bind for KAMI / e7m-sim.
//!
//! R1.0 path reservation per ADR-2605261800 §D2.
//! Backend = Genesis-Embodied-AI/Genesis (Apache-2.0), 5 solvers in one engine.
//! Integration path: Genesis Python API → Taichi IR → Vulkan SPIR-V → wgpu.
//!
//! R1.1 deliverable = Cartpole reward curve ±10% vs Isaac Sim baseline (1000 episodes).

pub const ADR: &str = "ADR-2605261800";
pub const PHASE: &str = "R1.0-path-reservation";
pub const KAMI_NAME: &str = "kami-genesis";
pub const NV_COMPAT_TARGET: &str = "isaacsim.core.api (articulation)";
pub const UPSTREAM_REPO: &str = "Genesis-Embodied-AI/Genesis";
pub const FORK_POLICY: &str = "upstream-only-no-fork";

pub const SOLVERS: &[&str] = &["rigid", "mpm", "sph", "fem", "pbd"];

/// Solver coverage planned for R1 sub-phases.
pub fn solver_for_phase(phase: &str) -> Option<&'static str> {
    match phase {
        "R1.1" => Some("rigid"),
        "R1.8" => Some("mpm"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r1_1_uses_rigid_solver() {
        assert_eq!(solver_for_phase("R1.1"), Some("rigid"));
    }

    #[test]
    fn solvers_list_includes_all_five() {
        assert_eq!(SOLVERS.len(), 5);
    }
}
