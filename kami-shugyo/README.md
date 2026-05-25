# kami-shugyo (e7m-shugyo 修行)

Isaac Lab-equivalent RL training framework — `isaaclab.envs.ManagerBasedRLEnv`
API-compat target.

**Status**: R1.0 path reservation (ADR-2605261800).
**Task DSL source**: `70-tools/isaac-lab-task-port/` (sole NVIDIA stack carve-out
per ADR-2605261600).

## Scope (R1.5 deliverable)

- Gym-style env wrapping kami-genesis World
- Manager-based task DSL (observation / action / reward / termination)
- Curriculum learning hooks
- Franka pick-and-place reference task

## License

Apache 2.0 + Charter Compliance Rider v2.0.
