# kami-usd

OpenUSD WASM bind + Hydra render delegate — `omni.usd` API-compat target.

**Status**: R1.0 path reservation (ADR-2605261800).

## Scope

- tinyusdz (C++ → WASM via Emscripten) parser for Crate / ascii / binary USD
- Hydra render delegate routing USD imageable prims → KAMI `kami-render` pipelines
- USD layer / stage / prim / attribute API mirror in `nv-compat/omni-usd.ts`

## License

Apache 2.0 + Charter Compliance Rider v2.0.
