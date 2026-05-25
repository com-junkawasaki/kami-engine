# kami-genesis

Genesis physics backend bind — Isaac Sim articulation API-compat target.

**Status**: R1.0 path reservation (ADR-2605261800 §D2).
**Upstream**: Genesis-Embodied-AI/Genesis (Apache-2.0).
**Fork policy**: upstream-only (no religious-corp fork per §D8).

## Why Genesis (vs MuJoCo MJX)

Genesis ships 5 solvers in a single backend (rigid / MPM / SPH / FEM / PBD).
MJX is rigid-only, requiring 4 additional backend integrations.

## Integration path (R1.1+)

```
Python user code (Isaac Sim API-compat)
  → pymagatama.nv_compat.isaacsim.core.api.World
    → kami-genesis bind
      → Genesis Python API
        → Taichi IR
          → Vulkan SPIR-V
            → wgpu compute pipeline (WebGPU)
              → KAMI scene render
```

**CPU fallback**: Taichi `cpu` backend → WASM (Emscripten) for browser without WebGPU.

## R1.1 PoC plan (Cartpole)

1. Vendor Genesis @ `lib/genesis/` (charter-rider-applicator skip pattern)
2. Write minimal Rust → Python bridge (PyO3 or wasm-bindgen + Pyodide)
3. Load Cartpole URDF from `70-tools/e7m-sim/scenes/cartpole/cartpole.urdf`
4. Run PPO training for 1000 episodes
5. Compare reward curve vs Isaac Sim baseline (target: ±10%)

## Known gaps (honest scoring)

| Gap | R1.1 impact |
|---|---|
| Genesis WebGPU backend non-existent upstream | Need to contribute. R1.1 may need CPU-only first. |
| Taichi → wgpu transpile is research-grade | Wave-1: only rigid solver, deferred MPM/SPH/FEM/PBD to R1.8 |
| Python-in-WASM (Pyodide) startup ~3-5s | Mitigation: pre-warm in Worker before user interaction |

## License

Apache 2.0 + Charter Compliance Rider v2.0.
