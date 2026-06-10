# ADR-0034: Isaac-compat stack maturation — control, IK, DR, sensors, solver invariants

- Status: Accepted
- Date: 2026-06-10
- Scope: `kami-genesis`, `kami-shugyo`, `kami-sensor-sim`, `kami-articulated`
- Related: ADR-0033 (kami-genesis isaacsim.core.api surface — the starting point),
  ADR-2605261800 (clean-room nv-compat charter)

## Context

ADR-0033 established the clean-room `isaacsim.core.api` surface on a KAMI-native
solver. Beyond that single-env surface, the three Isaac-compat crates were still
shallow: the multi-env batch had only effort I/O, there was no inverse
kinematics, no trajectory control, no domain randomization, the RL framework
(`kami-shugyo`) was Cartpole-only with no training, the sensors
(`kami-sensor-sim`) were unconnected to the physics, and the dynamics/contact
solvers had untested invariants. To run realistic Isaac Lab manipulation/RL
workloads unchanged, the stack needed to mature into an integrated
physics → control → RL → perception pipeline — still clean-room, no NVIDIA code.

## Decision

Mature the three crates into a production-integrated stack, each addition
landing on the existing native solver with full, often adversarial, test
coverage. The clean-room invariant (no NVIDIA library/header/binary linked) is
preserved throughout; only public API names/shapes are mirrored.

### kami-genesis (`isaacsim.core.api` + PhysX-shaped dynamics)

- **Multi-env `ArticulationBatch`** with Isaac `[num_envs, n_dof]` tensor I/O:
  built from URDF; full actuator set — effort, implicit-PD (global/per-joint
  gains), velocity, combined position+velocity, position+velocity+**acceleration
  feedforward** (computed-torque trajectory tracking), and a
  gravity-compensation feedforward; observation accessors
  (`get_joint_*`, per-env `get_jacobians` / `get_world_poses`,
  `get_applied_joint_efforts`, `dof_names` / `get_dof_index` / `get_dof_limits`).
- **Inverse kinematics**: damped-least-squares position IK (`solve_ik`), tool-
  point IK (`solve_ik_point`), and full 6-DOF **pose** IK (`solve_ik_pose`,
  position + orientation on the 6×n geometric Jacobian).
- **Per-env physics domain randomization** (`randomize_physics` /
  `randomize_gravity` / `randomize_mass`): only the dynamics path varies per env;
  kinematics stays shared.
- **Single-env surface** additions: state setters, world clock, controller,
  `dof_names`/`dof_limits`, `get_world_velocity`.
- GPU cartpole/DP batch (`vectorized` / `WgpuBackend`) verified on Metal.

### kami-shugyo (`isaaclab.envs.ManagerBasedRLEnv`)

- General `VectorizedReachEnv` (joint-space) and `VectorizedEeReachEnv`
  (end-effector Cartesian, with tool offset + operational-space IK control)
  over `ArticulationBatch`, plus the original Cartpole envs, unified under a
  `VecRLEnv` trait so a trainer runs over any of them.
- Gradient-free training: goal-conditioned `LinearPolicy` + `random_search`
  (ARS-lite) — the framework learns, not just simulates.
- Normalized (squashed) actions rescaled to joint limits.
- Sim-to-real **domain randomization**: per-env gravity + mass physics DR,
  action noise, and observation (sensor) noise — all seeded/reproducible; obs
  noise is applied to the policy-facing `StepResult` (truth stays clean) and the
  trainer evaluates on it.
- Sensor-in-the-loop **touch-reach** (kami-sensor-sim `ContactSensor` →
  observation + reward bonus).

### kami-sensor-sim (`isaacsim.sensors`)

- All four sensors (Camera, Lidar, IMU, ContactSensor incl. `sample_all`
  multi-contact) implemented and each given an end-to-end `tests/*_on_genesis.rs`
  rig that reads kami-genesis link state on a moving robot.

### Solver correctness (find-and-fix + invariants)

- **Bug fixed**: `inverse_dynamics` omitted the joint viscous-damping torque, so
  it was not the exact inverse of `forward_dynamics`; computed-torque tracking on
  damped URDF arms drooped by `M⁻¹·d·q̇`. Adding the `+ d·q̇` term made tracking
  near-exact.
- **Contact**: split-impulse position solve — Baumgarte penetration push-out runs
  on a pseudo-velocity integrated into positions only, so it injects no kinetic
  energy (a deeply-penetrated body is eased out, not launched).
- **Invariants regression-tested on the real 6-DOF URDF**: inverse = forward
  dynamics round-trip, symmetric positive-definite CRBA mass matrix, kinetic-
  energy conservation coasting in zero gravity, contact passivity.

## Consequences

- A realistic Isaac Lab loop runs end-to-end on the native stack: physics →
  RL env (+ IK/trajectory control, DR) → perception, with the contact sensor
  independently confirming task success.
- ~275 tests pass across the three crates (292 with the GPU feature); all
  downstream consumers (kami-autodrive, kami-app-isekai, kami-cartpole-wasm)
  build with no regression.
- Crate `description`s, READMEs, and this ADR are the synchronized source of
  truth.

## Honestly still open

- GPU-parallel execution of the **general** (Featherstone) articulation batch —
  today a CPU loop; the cartpole/DP wgpu batch is the only GPU path.
- Residual ~10% energy gain on a fully-elastic bounce is semi-implicit-Euler
  velocity-reversal timing (a sub-stepped time-of-impact would remove it), not
  the contact bias.
- Per-env physics DR covers gravity + mass; friction/restitution DR and a
  unified single+multi-env `World` remain.
- The full Genesis 5-solver Taichi→wgpu bind (rigid only today).
- G5 validation against captured NVIDIA Isaac ground-truth CSV (analytic
  baseline until then).
