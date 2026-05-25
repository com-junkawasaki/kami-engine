# kami-physics-solvers

**Contingent fallback** 5-solver physics engine, from-scratch WGSL compute.

**Status**: R1.0 path reservation (ADR-2605261800 §D10.4).
**Activation**: Council Lv6+ ≥3 attestation after Genesis WebGPU viability
gate failure (R1.1 rigid OR R1.8 MPM/SPH/FEM/PBD per §D10.1).

## Scope (if activated)

| Solver | Algorithm reference | Use case |
|---|---|---|
| rigid | Featherstone articulated body + XPBD constraints | wadachi / suki / sarutahiko vehicle dynamics |
| MPM | Material Point Method (Stomakhin et al.) | igata megacasting granular flow |
| SPH | Smoothed Particle Hydrodynamics | watatsumi water sim |
| FEM | Finite Element Method (linear corotational) | makura foam compression |
| PBD | Position-Based Dynamics | hagukumi cloth / textile |

## API surface (mirrors PhysX + Isaac Sim articulation)

When activated, mirrors:

- `isaacsim.core.api.{World, Articulation, RigidPrim, JointAPI}` (Isaac Sim)
- `PxScene / PxRigidDynamic / PxArticulationReducedCoordinate / PxJoint / PxShape` (PhysX 5)

The nv-compat facade (`@etzhayyim/sdk/nv-compat/{isaacsim,physx}` and
`pymagatama.nv_compat.{isaacsim,physx}`) is **unchanged** when the backend
swaps from kami-genesis to this crate (§D10.3 invariant).

## Honest scoping

From-scratch is **5-10× the effort** of binding to Genesis. Activating this
crate commits religious-corp to multi-year robotics R&D. See §D10.5.

## License

Apache 2.0 + Charter Compliance Rider v2.0.
