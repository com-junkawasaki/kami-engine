# kami-cam-scene

Data-tier authoring surface for **`kami-cam` stock-material presets** (ADR-0046 /
ADR-0038). The five hardcoded `CamMaterial::{aluminum_6061, steel_1045,
titanium_ti6al4v, abs_plastic, wood_oak}()` builders become canonical EDN, loaded back
into the real `kami_cam::CamMaterial` struct.

```edn
;; data/materials.edn
{:cam/materials
 {:aluminum-6061    {:name "Aluminum 6061-T6"   :density 2.70 :hardness 95.0}
  :steel-1045       {:name "Steel 1045"         :density 7.87 :hardness 163.0}
  :titanium-ti6al4v {:name "Titanium Ti-6Al-4V" :density 4.43 :hardness 334.0}
  :abs-plastic      {:name "ABS Plastic"        :density 1.04 :hardness 10.0}
  :wood-oak         {:name "Oak (Red)"          :density 0.66 :hardness 6.0}}}
```

```rust
use kami_cam_scene::shipped_material;
let alu = shipped_material("aluminum-6061").unwrap(); // real kami_cam::CamMaterial
```

## The recipe (ADR-0046)

1. **Canonical EDN** in `data/materials.edn`, shipped via `include_str!` as
   `MATERIALS_EDN` — the source of truth.
2. **`from_edn` loaders** built on the tolerant `kami-scene` accessors (hyphen-keyword
   ids `:aluminum-6061`, int↔float coercion, missing key → default). They return the
   engine's own `CamMaterial`.
3. **`kami-cam` is untouched.** Its hardcoded builders stay as `builtin_material()` —
   both the runtime fallback **and** the parity oracle.
4. **Parity tests are the contract** (`tests/materials_parity.rs`): every shipped EDN
   value is asserted `==` the value from the *real Rust* (`CamMaterial::aluminum_6061()`
   …, never transcribed). `CamMaterial` has no `PartialEq`, so a local `CamMaterialSpec`
   mirror carries it. Decimal literals parse to the same f64 as the Rust literals, so
   parity is exact.
5. **Additive, isolated.** New crate + one `[workspace] members` line; `kami-cam` stays
   edn-free (the `kami-scene` dep lives only here).

## Verify

```bash
cargo test-native -p kami-cam-scene
```

The same EDN a native loader reads is plain data a CLJS/Datomic authoring brain can
produce, fork, and `as-of` — the ADR-0040 substrate, now covering CAM stock too.
