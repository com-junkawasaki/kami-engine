//! kami-shugyo (修行) — Isaac Lab-equivalent RL training framework.
//!
//! R1.1 PoC scope (ADR-2605261800):
//!   - `RLEnv` trait (reset / step / observation / action / reward / done)
//!   - `CartpoleEnv` loading scene.yaml + URDF
//!   - random-policy baseline runner
//!
//! API surface mirrors `isaaclab.envs.ManagerBasedRLEnv` (Isaac Lab 1.x).
//! See `nv-compat/isaaclab` for facade.

pub const ADR: &str = "ADR-2605261800";
pub const PHASE: &str = "R1.1-cartpole-poc";
pub const KAMI_NAME: &str = "e7m-shugyo";
pub const NV_COMPAT_TARGET: &str = "isaaclab.envs.ManagerBasedRLEnv";

mod cartpole_env;
mod dr;
mod ee_reach_env;
mod policy;
mod reach_env;
mod scene_cfg;
pub mod traits;
mod vectorized_env;

pub use cartpole_env::CartpoleEnv;
pub use dr::{DomainRandomizationCfg, Range};
pub use ee_reach_env::VectorizedEeReachEnv;
pub use policy::{LinearPolicy, evaluate, random_search, rescale_to_limits};
pub use reach_env::{ReachCfg, VectorizedReachEnv};
pub use scene_cfg::{SceneCfg, load_scene_yaml};
pub use traits::{RLEnv, StepResult, VecRLEnv, run_zero_action_rollout};
// `VecRLEnv` unifies all vectorized envs (Cartpole / joint reach / EE reach)
// under one `[num_envs, dim]` contract, so a trainer runs over any of them.
pub use vectorized_env::VectorizedCartpoleEnv;
