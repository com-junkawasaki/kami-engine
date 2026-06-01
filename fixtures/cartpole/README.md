# e7m-sim scene — `cartpole/`

**Status**: R1.1 reference scene placeholder per ADR-2605261800.

## Binding

- **Substrate ADR**: ADR-2605261800 — NVIDIA Omniverse stack API-compat layer (sub-charter under ADR-2605261600).
- **Physics backend**: kami-genesis (Genesis Apache-2.0, rigid solver).
- **Renderer**: kami-render (KAMI wgpu PBR; R1.2 will swap in kami-pbrt + kami-rt).
- **nv-compat target**: `isaacsim.core.api.World` + `isaaclab.envs.ManagerBasedRLEnv` (`Isaac-Cartpole-v0` task).

## R1.1 deliverable

PPO training loop on Cartpole, 1000 episodes, reward curve within ±10% of Isaac
Sim baseline (G5 quality gate).

## Files (R1.1 reservation)

```
40-engine/kami-engine/fixtures/cartpole/
├── README.md                  # this file (R1.0)
├── cartpole.urdf              # 2-DoF (slider + revolute), kami-articulated input (R1.0)
├── scene.yaml                 # Isaac Lab-compat scene config (R1.0)
└── task.py                    # ManagerBasedRLEnv task definition (R1.1, deferred)
```

## Drop-in test (R1.5+, after kami-shugyo lands)

```python
# original Isaac Lab Cartpole training script
from isaaclab.envs import ManagerBasedRLEnv
from isaaclab_tasks.manager_based.classic.cartpole.cartpole_env_cfg import CartpoleEnvCfg

cfg = CartpoleEnvCfg()
env = ManagerBasedRLEnv(cfg=cfg)
# ... PPO training loop ...

# nv-compat version (import paths only change)
from pymagatama.nv_compat.isaaclab.envs import ManagerBasedRLEnv
from pymagatama.nv_compat.isaaclab_tasks.manager_based.classic.cartpole.cartpole_env_cfg import CartpoleEnvCfg

cfg = CartpoleEnvCfg()
env = ManagerBasedRLEnv(cfg=cfg)
# ... same PPO training loop ...
```

## G5 quality gate (R1.1)

| Metric | Target | Method |
|---|---|---|
| Reward curve final mean | within ±10% of Isaac Sim baseline | 1000 episodes, 5 seeds each |
| Episode length convergence | matches Isaac Sim within 2 std | 200-step cap |
| Wall-clock per episode | report only (no gate at R1.1) | for capacity planning |

## Honest gap (R1.1 known issues)

- Genesis WebGPU backend non-existent upstream. R1.1 starts on CPU (Taichi cpu
  backend → WASM). WebGPU promotion deferred until upstream PR lands.
- Mitsuba 3 not used at R1.1 (render is KAMI wgpu PBR only). Differentiable
  rendering swap happens at R1.2.
- nv-compat `isaaclab.*` facade is R1.5 deliverable; R1.1 uses canonical
  `pymagatama.sim` API directly.

## License

Apache 2.0 + Charter Compliance Rider v2.0 (`/CHARTER-RIDER.md`).
