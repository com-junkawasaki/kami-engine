# ADR-0033: kami-genesis clean-room `isaacsim.core.api` surface maturation

- Status: Accepted
- Date: 2026-06-09
- Scope: `kami-genesis` (`isaac_api`, `world`, `controllers`), `kami-articulated`
- Related: ADR-2605261800 §D2/§D10.1/§2(b) N1..N9 (clean-room nv-compat charter,
  monorepo), ADR-2605261600 §G5 (quantitative quality gate)

## Context

`kami-genesis` exposes a **clean-room** mirror of NVIDIA Isaac Sim 4.x's
`isaacsim.core.api` so application code written against Isaac runs unchanged,
while **no NVIDIA library, header, or binary is linked or referenced** — all
dynamics are solved by the KAMI-native reduced-coordinate solvers
(`world` / `planar_chain` / `articulation3d` / `cartpole`). The charter
invariant (ADR-2605261800 §2(b) N1..N9 NEVER) is method-name/shape mirroring
only.

The surface had drifted behind the solver. `isaac_api.rs` covered the `World`
lifecycle and read accessors, but real Isaac robot/RL code immediately needs
four things that were missing — and the package metadata + README still
described an "R1.1 PoC: closed-form Cartpole", understating the implemented
solver (Cartpole, DoublePendulum, RNEA+CRBA PlanarChain, Featherstone
Spatial3d). The honest-scoring docs and the code had diverged.

## Decision

Mature the `isaacsim.core.api` surface to the minimum that makes a real Isaac
robot-control / RL-reset loop run unchanged, each layer landing on the existing
native solver with no new physics and full test coverage. Keep the docs honest:
every added method is mirrored from the public, documented Isaac 4.x surface;
everything still missing is listed as a gap.

Added surface (all single-env, `num_envs = 1`):

1. **State setters** — `Articulation.set_joint_positions` / `set_joint_velocities`
   (the RL `reset()`-to-distribution / teleport path), generic across all four
   topologies, position and velocity independently settable.

2. **World clock** — `World.current_time` / `current_time_step_index` /
   `get_physics_dt`, with the clock rewinding on `reset()`.

3. **Articulation controller** — `articulation.get_articulation_controller()`
   returning a view that drives via `apply_action(ArticulationAction)` (PD
   position + velocity + feedforward-effort law, effort clamp) and is
   configurable via `set_gains` / `set_max_efforts`. One controller is created
   per prim on `add`, with **drive parameters seeded from the URDF** the way
   Isaac loads them from USD/URDF: `<limit effort>` → `max_efforts`,
   `<dynamics damping>` → `kd`. Stiffness `kp` has no standard URDF field, so it
   stays 0 until `set_gains` (or a future USD-drive `stiffness`).

4. **DOF name mapping** — `Articulation.dof_names` (ordered actuated-joint
   names, one per DOF, aligned with the joint-position array) and
   `get_dof_index(name)`, so actions can target a joint by name.

The borrow seam for the controller view holds disjoint mutable borrows of the
per-prim controller map and the world, so a single `apply_action` call reads
joint state and stages torques — matching Isaac's call shape exactly.

## Consequences

- A canonical Isaac control loop now runs unchanged on the native solver:
  `ctrl = art.get_articulation_controller(); ctrl.set_gains(...);
  ctrl.apply_action(ArticulationAction(joint_positions=...)); world.step()`.
- The RL `reset()` path (sample a start state, set positions/velocities, zero
  the clock) is expressible without reaching past the Isaac surface.
- `kami-genesis` package `description`, `README.md` (implemented-vs-reserved
  surface table), and this ADR are the synchronized source of truth; the stale
  "R1.1 PoC" framing is retired.
- Test coverage for the surface went from 4 → 14 `isaac_api` tests; the crate
  lib suite is **153 passing / 0 failing** (`--no-default-features`).

## Still reserved (honest scope)

- Multi-env (`num_envs > 1`) at the `isaac_api` surface. The GPU-batched
  1024-env paths exist (`vectorized` / `WgpuBackend`, validated bit-identical to
  the scalar loop with ≥10× speedup in `r1_2_scorecard`) but are not yet wired
  into the Isaac-shaped `[num_envs, n_dof]` array semantics.
- PD stiffness auto-load (needs USD drive parsing).
- The full Genesis 5-solver Taichi→wgpu bind (rigid only today; MPM standalone;
  SPH/FEM/PBD deferred to R1.8 per ADR-2605261800 §D7).
- G5 validation against captured NVIDIA Isaac ground-truth CSV (the harness
  scores against the committed analytic baseline until that drop-in lands).
