//! Gym-style RL environment trait.
//!
//! Designed to mirror the `isaaclab.envs.ManagerBasedRLEnv` contract:
//!   - `reset()` returns initial observation
//!   - `step(action)` returns `(observation, reward, terminated, truncated, info)`
//!
//! For R1.1 the observation + action are flat `Vec<f32>` for simplicity.
//! Vectorized envs implement [`VecRLEnv`] with `[num_envs, dim]` tensor I/O.

#[derive(Debug, Clone, PartialEq)]
pub struct StepResult {
    pub observation: Vec<f32>,
    pub reward: f32,
    pub terminated: bool,
    pub truncated: bool,
}

pub trait RLEnv {
    /// Reset to initial state with optional random `seed`. Returns initial obs.
    fn reset(&mut self, seed: Option<u64>) -> Vec<f32>;

    /// Apply `action`, advance one decimation × dt of simulation, return result.
    fn step(&mut self, action: &[f32]) -> StepResult;

    /// Dimensionality of the flat observation vector.
    fn observation_dim(&self) -> usize;

    /// Dimensionality of the flat action vector.
    fn action_dim(&self) -> usize;
}

/// Vectorized RL environment contract — `num_envs` independent copies stepped in
/// lockstep with `[num_envs, dim]` env-major flat tensors, mirroring Isaac Lab's
/// `ManagerBasedRLEnv` batched API. A trainer written against this trait runs
/// over any env (Cartpole, joint-space reach, EE Cartesian reach, …).
pub trait VecRLEnv {
    /// Number of parallel environments.
    fn num_envs(&self) -> usize;

    /// Per-env observation width (flat tensor is `num_envs * this`).
    fn observation_dim_per_env(&self) -> usize;

    /// Per-env action width (flat action tensor is `num_envs * this`).
    fn action_dim_per_env(&self) -> usize;

    /// Reset all envs (seeded). Returns the `[num_envs, obs_dim]` observation.
    fn reset_all(&mut self, base_seed: Option<u64>) -> Vec<f32>;

    /// Step all envs with a `[num_envs, action_dim]` flat action tensor.
    fn step_all(&mut self, actions: &[f32]) -> Vec<StepResult>;

    /// Current `[num_envs, obs_dim]` observation without advancing.
    fn observations_flat(&self) -> Vec<f32>;
}

/// Trainer-agnostic rollout: reset, then step a zero-action policy for `steps`
/// control ticks, returning the total reward summed over envs and ticks. A
/// minimal harness proving any [`VecRLEnv`] composes with a generic training
/// loop (real policies replace the zero action with their own).
pub fn run_zero_action_rollout<E: VecRLEnv>(env: &mut E, steps: usize, seed: Option<u64>) -> f32 {
    env.reset_all(seed);
    let action = vec![0.0_f32; env.num_envs() * env.action_dim_per_env()];
    let mut total = 0.0_f32;
    for _ in 0..steps {
        for s in env.step_all(&action) {
            total += s.reward;
        }
    }
    total
}
