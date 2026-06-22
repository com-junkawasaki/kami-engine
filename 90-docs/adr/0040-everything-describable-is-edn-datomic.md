# ADR-0040 — Everything describable is EDN/Datomic; native is only the hot core

- Status: accepted
- Date: 2026-06-22
- Builds on: ADR-0038 (Rust base + CLJ/Datomic game layer), ADR-0039 (kototama + render-IR
  single entry), kami-webgpu ADR-0001 (declarative WebGPU from EDN)

## Context

kami-webgpu proved the pattern end to end on the web: the **render graph itself**
(shaders, targets, samplers, pipelines, an ordered `:passes` array) is EDN *data*, and a
thin executor (CLJS→WebGPU on the web, Rust→wgpu natively) interprets it. Shadows became
"just another pass in the array." Materials became per-instance EDN. Nothing about the
*look* lives in code anymore.

That is not special to rendering. The same split applies to the whole engine, and we want
it applied consistently so that one substrate — EDN held as Datomic datoms — describes a
game, and `as-of` / query / fork work across all of it.

## Decision

Draw one line through the entire engine:

- **Description = EDN data** (authored, stored as Datomic datoms, queryable, `as-of`-able,
  forkable, serializable, cross-platform). This is the *what*.
- **Behaviour = CLJ** compiled to WASM by kototama (ADR-0039). This is the *imperative
  per-frame logic* that genuinely needs control flow.
- **Hot execution = native Rust** (ECS iteration, wgpu, physics solve, audio mixing). The
  *how*, where speed is non-negotiable.

If a thing *describes* rather than *computes per element*, it is EDN. The executor reads
the data; the data never reads the executor.

### What becomes EDN (the catalog)

Already EDN: render graph (kami-webgpu), scene/world dressing, material/PBR, tuning
constants, USD→EDN assets. Next, by the same rule:

| Domain | EDN description | Native/CLJ executor |
|---|---|---|
| **ECS schema** | component & entity defs (Datomic-style attributes) | hecs/Rust storage |
| **Render graph** | passes, pipelines, targets, materials, WGSL — *done* | kami-webgpu / wgpu |
| **Input** | device→action maps, axes, contexts, gamepad bindings | kami-input |
| **Audio** | sound banks, mixer/bus graph, event→cue tables | kami-audio mixing |
| **Animation** | clips, state machines, blend trees, retarget maps | skeleton/anim eval |
| **Physics** | collision layers/masks, body & material params, constraints | solver |
| **Particles/VFX** | emitter & curve definitions | particle sim |
| **UI/HUD** | hiccup-style layout + data bindings | kami-ui-gpu |
| **AI/logic glue** | behaviour trees, FSMs, spawn/wave/quest/dialogue tables | tick systems (CLJ) |
| **Camera** | rigs, follow/look constraints, shake profiles | transform eval |
| **Net** | replication schema (what syncs, authority, interp) | transport |
| **Build/levels** | scene graph, prefab/spawn tables, content packs | loader |

The boundary test, per domain: *the per-frame inner loop* stays native; *the configuration
of that loop* is EDN. A behaviour tree is EDN; the tree-walker is native. A mixer graph is
EDN; the sample mixing is native. A render `:passes` array is EDN; the command recording is
native.

### One substrate

All of the above are datoms in Datomic/DataScript. Consequences for free: time-travel
(`as-of` a past frame/build), query (find every entity using material X, every pass writing
target Y), and **fork** (clone a game's data, edit, replay) — the CodePen model, but for the
whole engine, not just source text.

## Consequences

- New subsystems ship a **schema + an interpreter**, not a bespoke config format. Adding a
  capability means extending the EDN vocabulary and teaching the executor one verb.
- The web and native executors interpret the **same EDN**; divergence is a bug, not a port.
- Editors/tools operate on data (validate, diff, hot-reload, visualize) without engine code.
- Risk: an over-generic interpreter can outrun what's implemented. Mitigation — the executor
  ignores unknown keys and `log`s any silently-unsupported description; the vocabulary grows
  deliberately, one verb at a time, each verified.
