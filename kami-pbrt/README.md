# kami-pbrt (kami-rtx)

Mitsuba 3 WebGPU backend bind — RTX Renderer API-compat layer.

**Status**: R1.0 path reservation (ADR-2605261800). No runtime code yet.
**Fork policy**: upstream-only (90-day PR hold per ADR §D3).

## Scope (R1.2 deliverable)

- Mitsuba 3 Dr.Jit → WebGPU compute pipeline (via upstream PR)
- Differentiable rendering (forward + reverse mode)
- Cornell box reference scene PSNR ≥ 35dB vs CUDA reference

## Non-goals

- Local Mitsuba 3 fork maintenance (ADR §D3)
- NVIDIA RTX SDK binary linking (ADR §N3)

## License

Apache 2.0 + Charter Compliance Rider v2.0. Vendored Mitsuba 3 (BSD-3) preserved
unmodified under `lib/mitsuba3/` when integration begins.
