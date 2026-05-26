//! Gym-style RL environment trait.
//!
//! Designed to mirror the `isaaclab.envs.ManagerBasedRLEnv` contract:
//!   - `reset()` returns initial observation
//!   - `step(action)` returns `(observation, reward, terminated, truncated, info)`
//!
//! For R1.1 the observation + action are flat `Vec<f32>` for simplicity.
//! R1.5+ will introduce tensor batching for vectorized envs.

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
