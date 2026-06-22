# ADR-0041 — kami-clj-play3d adopts kami-webgpu-rs (incremental)

- Status: accepted (incremental migration in progress)
- Date: 2026-06-22
- Builds on: ADR-0040 (everything describable is EDN), kami-webgpu ADR-0001

## Context

`kami-webgpu-rs` is the native Rust/wgpu executor of the EDN render-IR — the same data the
web renders via CLJS→WebGPU. It now has a reusable `Renderer` (offscreen or surface),
shadows, PBR materials, and a winit `live` example proving windowed native rendering.

`kami-clj-play3d` is the existing native player: a 1900-line winit/wgpu app with its own
hand-written pipelines (sky, ground, water, character) plus the game host
(kami-script-runtime), gamepad (gilrs), and audio (rodio). It works and has richer visuals
than kami-webgpu-rs today.

A one-shot renderer swap would **regress** play3d's water/character/sky. So adoption is
incremental, not a rewrite.

## Decision

play3d depends on `kami-webgpu-rs` and migrates onto its data-driven `Renderer` in stages,
keeping a working build at every step. The kami-webgpu-rs winit `live` example is the
reference integration (surface + `Renderer::new` + `Renderer::draw` per frame).

Migration order (each step ships independently, verified by running the window):

1. **Dependency in place** (this ADR): `kami-webgpu-rs` is a path dep of play3d; the
   render-IR types (`Globals`, `Instance`) are the shared contract.
2. **Bridge**: map play3d's live entities + `scene.edn` profiles/props → `Vec<Instance>`
   (the host already produces positions/tags; profiles already carry colour + PBR).
3. **Opt-in data path**: an env flag (e.g. `KAMI_DATA_RENDERER=1`) routes the frame through
   `kami_webgpu_rs::Renderer::draw` into play3d's surface, alongside the existing renderer
   for side-by-side comparison.
4. **Feature parity in the EDN graph**: move play3d's extra looks into render-graph passes
   so the data-driven path matches — ground/water as materials+passes, sky already implicit
   in the clear + hemisphere ambient, character meshes as a mesh kind in the render-IR.
5. **Flip the default** to the data-driven renderer; retire the bespoke pipelines.

## Consequences

- play3d gains the shared executor without losing features mid-flight.
- Web and native render the **same EDN**; a look authored once works on both.
- Verification: kami-webgpu-rs stays golden-frame tested (headless + PNG); play3d's window
  is verified by running it (`cargo run -p kami-clj-play3d`) at each step.
- Until step 5, play3d keeps its own renderer; the kami-webgpu-rs dep is available and the
  contract (render-IR types) is shared.
