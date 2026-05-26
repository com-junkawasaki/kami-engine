# kami-rtx-native

**Contingent fallback** path tracer + differentiable rendering, from-scratch
on `kami-rt` (WGSL ray-query + LBVH).

**Status**: R1.0 path reservation (ADR-2605261800 §D10.4).
**Activation**: Council Lv6+ ≥3 attestation after Mitsuba 3 wgpu upstream PR
viability gate failure at R1.2 (Cornell box 30 fps Chrome 121+, PSNR ≥35dB).

## Scope (if activated)

- Forward path tracing (uni-directional with MIS, next-event estimation)
- Differentiable rendering (reverse-mode auto-diff via WGSL compute)
- OptiX-equivalent acceleration structure API on top of kami-rt
- RTX Renderer-equivalent denoiser (OIDN-style WGSL port)

## API surface

Mirrors:
- `optix.h` C API (OptixDeviceContext / OptixPipeline / OptixModule /
  OptixProgramGroup / OptixShaderBindingTable / OptixLaunchParams)
- RTX Renderer Python API surface (per Omniverse Kit docs)

nv-compat facade unchanged on backend swap from kami-pbrt to this crate
(§D10.3 invariant).

## License

Apache 2.0 + Charter Compliance Rider v2.0.
