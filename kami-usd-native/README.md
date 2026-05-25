# kami-usd-native

**Contingent fallback** USD parser + composition engine, written from scratch
in Rust.

**Status**: R1.0 path reservation (ADR-2605261800 §D10.4).
**Activation**: Council Lv6+ ≥3 attestation after tinyusdz WASM viability gate
failure at R1.1 (iPhone 12+ Safari 10MB USD parse ≤2s).

## Scope (if activated)

- `.usda` (ascii) parser
- `.usdc` (Crate binary) parser
- `.usdz` (zip package) reader
- USD composition engine (layers / sublayers / references / payloads / variants /
  specializes / inherits)
- Schema-aware traversal (Usd / UsdGeom / UsdShade / UsdPhysics / UsdLux)

## API surface

Mirrors:
- `omni.usd` Python API (Omniverse Kit)
- `pxr.Usd / pxr.UsdGeom / pxr.UsdShade / pxr.UsdPhysics / pxr.UsdLux`
  (Pixar OpenUSD)

nv-compat facade unchanged on backend swap from kami-usd to this crate
(§D10.3 invariant).

## License

Apache 2.0 + Charter Compliance Rider v2.0.
