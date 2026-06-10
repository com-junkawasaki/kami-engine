# kami-shugyo (e7m-shugyo 修行)

Isaac Lab-equivalent RL training framework — `isaaclab.envs.ManagerBasedRLEnv`
API-compat target.

**Status**: R1.x active — vectorized gym envs over kami-genesis implemented.
**Task DSL source**: `70-tools/isaac-lab-task-port/` (sole NVIDIA stack carve-out
per ADR-2605261600).

## Implemented (R1.x)

- `RLEnv` gym trait + `StepResult` (reset / step / observation / action / reward
  / terminated / truncated), mirroring `ManagerBasedRLEnv`.
- **`VecRLEnv`** — the vectorized contract (`[num_envs, dim]` tensor I/O) all
  vectorized envs implement, so a trainer (`run_zero_action_rollout` and real
  policies) runs polymorphically over Cartpole / joint reach / EE reach alike.
- `CartpoleEnv` / `VectorizedCartpoleEnv` — scene.yaml + URDF, the classic
  control baseline.
- **`VectorizedReachEnv`** — a general joint-space reach task over **any URDF
  arm**, wrapping `kami_genesis::ArticulationBatch` (`[num_envs, n_dof]` tensor
  I/O). Action = joint position targets; obs = `[q, q̇, q_goal − q]`; reward =
  `−‖q − q_goal‖² − w·‖a‖²`; computed-torque actuator by default. Seed-
  deterministic per-env goal sampling, success/truncation termination. The
  scaffold the joint-space reach baseline.
- **`VectorizedEeReachEnv`** — an end-effector **Cartesian** reach task: the goal
  is a 3-D world point (sampled reachable via random-config FK), reward is
  EE-to-goal distance, obs adds the EE pose (`[q, q̇, ee_pos, goal − ee_pos]`).
  The direct precursor to Franka pick-and-place; exposes a reference IK solution.
  **Touch-reach** (sensor-in-the-loop): set `ReachCfg.contact_radius` to put a
  kami-sensor-sim `ContactSensor` in the loop — the goal becomes a touchable
  region adding an `in_contact` observation term (`observe_contact`) and a
  reward bonus (`contact_bonus`) on arrival. **Operational-space control**:
  `set_cartesian_goals` + `solve_ik_to_goals` reach an arbitrary (user-specified)
  Cartesian target via kami-genesis batched IK, not just FK-sampled goals.
  A `ReachCfg.tool_offset` moves the controlled point to a gripper tip in the EE
  link frame — observation, reward, goal sampling and IK all track the tool point.
- `DomainRandomizationCfg` — per-env Cartpole physics sampling. The articulation
  reach envs add sim-to-real DR: **action-noise** (`ReachCfg.action_noise_std`,
  reproducible per-step actuator noise) and **per-env physics**
  (`ReachCfg.gravity_dr` + `mass_dr`, re-randomised each `reset_all` via the
  batch's `randomize_physics`: per-env gravity scale + body mass/inertia scale).
  Plus **observation (sensor) noise** (`ReachCfg.obs_noise_std`) on the
  policy-facing `StepResult` (`observations_flat` stays clean ground truth; the
  trainer's `evaluate` reads the noisy step output so it shapes learning).
  All seeded/reproducible.
- **Training** — a goal-conditioned `LinearPolicy` (`a = W·obs + b`) plus
  `random_search`, a gradient-free hill-climbing / ARS-lite optimizer that
  improves the vectorized return under a fixed goal distribution (deterministic,
  no autodiff). Proves the framework *learns*, not just simulates.
- **Normalized actions** — `rescale_to_limits` maps a squashed policy's `[-1,1]`
  output to joint limits (Isaac Lab convention); `VectorizedReachEnv` applies it
  when `ReachCfg.normalized_actions` is set, reading limits from the batch's
  `get_dof_limits`.

## Full-stack integration

`tests/perception_rl.rs` composes all three Isaac-compat crates end-to-end:
kami-genesis (physics) → kami-shugyo (`VectorizedEeReachEnv` + policy) →
kami-sensor-sim (`ContactSensor`). The env is driven to its Cartesian goals and
the contact sensor — fed the env's end-effector observation — independently
*perceives* task success (EE within tolerance of each goal).

## Remaining (R1.5 deliverable)

- Manager-based task DSL (declarative observation / action / reward / termination)
- Curriculum learning hooks
- Franka pick-and-place reference task (extend EE reach with grasp + place phases)

## License

Apache 2.0 + Charter Compliance Rider v2.0.
