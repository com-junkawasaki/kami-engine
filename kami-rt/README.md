# kami-rt (hikari-rt 光)

WebGPU hardware ray tracing primitive — OptiX API-compat layer for KAMI engine.

**Status**: R1.0 path reservation (ADR-2605261800). No runtime code yet.

## Scope (R1.2 deliverable)

- WebGPU `ray-query` extension (Chrome 121+, iOS Safari 17+ where Metal RT available)
- WGSL software BVH fallback (LBVH compute build) when ray-query unavailable
- Acceleration structure build / refit / traversal
- OptiX API surface mirror in `20-actors/etzhayyim-sdk/src/nv-compat/optix.ts`

## Non-goals

- CUDA / OptiX SDK binary linking (ADR-2605261800 N3)
- NVIDIA RTX-only feature dependence (vendor-neutral GPU invariant)

## Gate (R1.2 G5)

PSNR ≥ 35dB vs Mitsuba 3 CUDA reference on Cornell box scene.

## License

Apache 2.0 + Charter Compliance Rider v2.0 (`/CHARTER-RIDER.md`).
