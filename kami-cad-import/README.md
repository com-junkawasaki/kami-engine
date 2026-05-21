# kami-cad-import

CAD source → vehicle part graph → JBeam topology + CycloneDX 1.5 SBOM.

Bridges `kami-cad` (BREP / assembly) and `kami-scad` (parametric source) to
`kami-vehicle` (soft-body sim) and `sbom.etzhayyim.com` (CVE / recall pipeline).

See [ADR 2605051430](../../../90-docs/adr/2605051430-drive-cad-jbeam-sbom-pipeline.md).

## Why two outputs from one part graph

Without a single source of truth the simulator and the SBOM drift apart.
A part that exists in physics but not in the SBOM hides supplier-quality
risk; a part that exists in the SBOM but not in physics produces fake CVE
matches. Same `VehicleAssembly`, two emitters, no drift.

## Pipeline

```
STEP / glTF / OpenSCAD source
   │
   ▼
VehicleAssembly { parts[VehiclePart], hardpoints[Hardpoint] }
   │   │
   │   ├─► jbeam_emit::emit  → kami-vehicle JBeam JSON
   │   └─► sbom::emit        → CycloneDX 1.5 JSON  →  sbom.etzhayyim.com
```

Phase 1 (this crate) ships the data model + emitters + a programmatic
synthetic-sedan example. Phase 1.1 lands the STEP / glTF / OpenSCAD
ingest adapters; Phase 2 enriches the JBeam topology with surface-sampled
hull nodes for body panels and dedicated wheel-ring scaffolding.

## Example

```bash
cargo run -p kami-cad-import --example synth_sedan
```

Produces both a JBeam JSON (loadable by `kami-vehicle::jbeam`) and a
CycloneDX 1.5 SBOM (ingestable by `sbom.etzhayyim.com/register-sbom`) for an
8-part synthetic sedan (chassis + hood + trunk + windshield + engine +
4 wheels) with 9 hardpoints (hinges + bolts + adhesive bond).

## Test

```bash
cargo test -p kami-cad-import
```

## Provenance is mandatory

Every `VehiclePart` MUST declare `source { uri, sha256, license }`. Both
emitters refuse to produce output otherwise. License clearance is forced
at ingest, not at deploy.

## CycloneDX field map

| CDX field | Source |
|---|---|
| `bom-ref` | `VehiclePart::id` |
| `type` | `"device"` (CDX 1.5 first-class hardware type) |
| `purl` | synthesized: `pkg:gftd-vehicle/{vehicleId}/part/{partId}@{rev}?supplier=...&material=...&kind=...` |
| `manufacturer` | `Supplier::name` when set |
| `cpe` | `Supplier::cpe` when set |
| `licenses` | `[{ "expression": Source.license }]` |
| `evidence.identity.concludedValue` | `Source.sha256` |
| `properties` | `cdx:gftd:vehicle:{break_group, mass_kg, material, kind, parent, supplier_mpn, source_uri}` |
| `dependencies` | parent edges + bidirectional hardpoint edges |
