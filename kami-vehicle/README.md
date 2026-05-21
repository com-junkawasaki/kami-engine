# kami-vehicle

BeamNG-grade soft-body vehicle physics for the KAMI engine.
Pure Rust + glam + serde, no GPU dependency. Live demo:
<https://driver.gftd.ai>.

## Granularity

Each car is a *cloud of mass nodes connected by beams* — the structure
BeamNG.drive uses. The default sedan ships with:

| Element | Count | Notes |
|---|---|---|
| Mass nodes | **~86** | 24 chassis (floor / belt / roof) + 4 cargo (engine / battery / tank) + 2 subframe (chassis-centre, hub-height — control-arm pivots) + 8 wheel hubs + 48 tire-ring nodes |
| Beams | **~220** | chassis frame + crush zones + cabin pillars + subframe struts + suspension (twin coils + control arms + lateral tie + upper strut) + tire side-walls + tire tread |
| Triangles | **48** | filled body shell — floor / sides / windshield / rear window / hood / trunk / fascia (rendered as Lambert-shaded panels) |
| Wheels | **4** | hub-axle + 12-node tire ring + Pacejka tire model + pressure-modulated side-wall beams + filled wheel disc + tread band |
| Powertrain | full | engine torque curve / clutch (with kinematic coupling) / 6-speed gearbox / front + rear differential / FWD-RWD-AWD driveline |

## Garage (`kami_vehicle::models::garage`)

| Kind | Layout | Engine | Tuning |
|---|---|---|---|
| `Sedan` | FWD | NA 2.0L | mid-size baseline |
| `Hatchback` | FWD | NA 1.5L | shorter wheelbase, lighter |
| `Suv` | AWD (LSD f+r) | turbo 2.0L | tall, heavy, more reduction |
| `Sports` | RWD | turbo 2.0L | low slung, sticky tires (D=1.20) |
| `Pickup` | RWD | V6 (custom curve) | long wheelbase, high torque |
| `Bus` | RWD | diesel (custom curve) | massive low-rpm torque |

Build with `kami_vehicle::build_vehicle(VehicleKind::Sedan)`.

## Integrators

Two interchangeable algorithms via `Vehicle::set_integrator_mode(IntegratorMode::*)`:

| Mode | Method | When to use |
|---|---|---|
| `Xpbd` (default) | Extended Position-Based Dynamics, 30 Gauss-Seidel iterations + tire-as-PBD-constraint + rigid-chassis projection (Müller 2005 shape match, translation-only) | Production driving, unconditionally stable, fast |
| `Implicit` | Implicit Euler `(M + dt² K) v = M v_old + dt F` solved with sparse Conjugate Gradient (60 iters, 1e-4 tolerance) | Reference / experimental — handles cyclic constraint graphs natively, slower |

Internal substep rate **2000 Hz** (sub-step config in `IntegratorConfig`),
render rate **independent**.

## Surfaces (`SurfaceKind` + `MapGround`)

8 pre-tuned surface presets, each with a `(friction_mu, grip_modifier)` tuple
that modulates Pacejka peak grip:

| Surface | μ | grip | tint |
|---|---|---|---|
| `AsphaltDry` | 1.00 | 1.00 | dark grey |
| `AsphaltWet` | 0.70 | 0.70 | blue-grey |
| `Gravel` | 0.55 | 0.55 | brown |
| `Sand` | 0.40 | 0.45 | yellow |
| `Snow` | 0.30 | 0.35 | white |
| `Ice` | **0.10** | **0.10** | light blue |
| `Mud` | 0.35 | 0.40 | dark brown |
| `Grass` | 0.55 | 0.60 | green |

`MapGround::demo_circuit()` ships a reference layout (asphalt main road
with wet / ice / snow patches; off-road sand / gravel / mud / grass
zones on the sides).

## Module map

```
node.rs           — mass point (position / velocity / mass / drag / friction)
beam.rs           — pairwise spring-damper; types: Normal | Bounded |
                    Hydro | Pressured | Support; plastic-deformation +
                    break model
triangle.rs       — aero / body-panel surface (3 nodes)
wheel.rs          — hub + tire ring + Pacejka 1996 magic formula
powertrain.rs     — Engine / Clutch / Gearbox / Differential / Driveline
                    (FWD/RWD/AWD) + reference torque curves
controls.rs       — driver inputs (throttle / brake / steer / clutch /
                    handbrake / gear)
ground.rs         — Ground trait + FlatGround + ClosureGround + MapGround
                    + SurfaceKind (8 presets)
integrator.rs     — sub-step scheduler (max_dt = 0.5 ms)
implicit.rs       — implicit-Euler integrator + Conjugate Gradient solver
rigid_chassis.rs  — Müller-style shape-matching constraint
vehicle.rs        — composite Vehicle + step() + IntegratorMode dispatch
builder.rs        — programmatic vehicle assembly
jbeam.rs          — JBeam-subset JSON loader
models/
  sedan.rs        — parametric sedan generator (24 + 8 + 48 nodes; cabin
                    pillars; primary coil + bump-stop; subframe pivots;
                    body-panel triangulation)
  garage.rs       — VehicleKind enum + 6 presets
```

## Quick start

```rust
use kami_vehicle::{
    build_vehicle, VehicleKind,
    ground::MapGround,
};

let mut car = build_vehicle(VehicleKind::Sports);
let map = MapGround::demo_circuit();

car.controls.throttle = 1.0;
car.powertrain.gearbox.current_gear = 1;
car.powertrain.gearbox.shift_progress = 1.0;

for _ in 0..240 {
    car.step(1.0 / 60.0, &map);
}
println!("speed: {:.1} km/h", car.speed() * 3.6);
```

## Parts API (BeamNG-style detach / repair)

Beams carry `break_group: Option<u32>`. The reference sedan groups are:

| Group | Members |
|---|---|
| 1 | Floor longitudinals + cross-members + diagonals |
| 2 | Cabin pillars + belt-line |
| 3 | Roof |
| 4 | Engine bay / cargo bracing |
| 5 | Subframe support struts |

API:

```rust
vehicle.break_group(3);     // detach roof
vehicle.repair_group(3);    // reattach roof
vehicle.repair_all();       // body-shop respray
```

## Rendering interface

The crate is pure data — rendering is the consumer's job. The browser
demo (`kami-app-car-sim`) reads:

* `vehicle.nodes[].position` — vertex positions for line / point geometry
* `vehicle.beams[].{n1, n2, broken, deform.break_limit, current_length, effective_length}` — wireframe edges with stress-based colouring
* `vehicle.triangles[].{n1, n2, n3, group}` — filled body panels (Body / Wing → paint, Window → translucent blue, Underbody → dark grey)
* `vehicle.wheels[].{tire_nodes, axle_n1, axle_n2}` — wheel-disc triangulation

## Tests

54 unit tests + 1 doctest. Coverage:

* node accounting (anchor mass, inv-mass)
* beam spring + damper sign convention, plastic flow, break threshold,
  bounded / hydro / pressured / support types
* Pacejka tire (zero slip, locked wheel, friction-circle clamp)
* powertrain (torque curve interpolation, idle controller, clutch slip,
  gearbox ratios incl. reverse, open / locked / LSD differentials,
  driveline distribution)
* ground (flat plane + closure-driven slope)
* integrator sub-step scheduling
* JBeam JSON load + position-recovered rest length + error reporting
* sedan composition (node / beam / wheel counts, mass band, settle,
  rolls forward, AWD distribution)
* garage (all 6 vehicle kinds settle without breaking > 20 beams)
* rigid-chassis shape match (identity polar decomp, pure-rotation
  recovery, outer-product correctness)
* implicit-Euler CG (trivial diagonal, single-beam stability)

```
cargo test -p kami-vehicle
```

## Architecture notes (lessons learned)

The path from "explicit Euler with springs" to today's stable XPBD took
several iterations. Key bugs and fixes are documented inline in the
crate but the headline ones:

* **Explicit Euler exploded** with stiff frame springs — Courant
  restriction. Fixed by moving to XPBD (unconditionally stable).
* **PBD constraint cycles drifted** (chassis sank under sustained
  gravity) — fixed by adding a per-frame rigid-chassis projection.
* **Wheels lost ground contact** under throttle — `TIRE_K = 5 MN/m`
  gave only 0.7 mm static pen; any 1 cm oscillation lifted the hub off
  the ground. Softened to 100 kN/m → 3.7 cm static pen, hubs stay
  planted under chassis dynamics.
* **Engine wouldn't rev under throttle** — the clutch was transmitting
  the full engine torque (`engine_load = 0`) so the engine had no
  inertia load. Added a kinematic coupling term that pulls
  `engine_omega` toward `gearbox_input_omega` proportional to clutch
  engagement (~0.5 s lock-up).
* **Cruise speed capped at 2 km/h** — global `VEL_DAMPING = 0.998` per
  substep × 33 substeps per frame compounded to 6.3 % loss per frame
  (98 % per second!). Reduced to 0.99995 → 0.17 %/frame, equivalent to
  a tiny air-drag coefficient.
* **Lower control arms slammed the chassis upward** — diagonal arm
  geometry meant 6 cm vertical chassis sink → 19.6 % beam strain →
  enormous spring force. Fixed with the subframe pivot trick: arms
  attach to a chassis-centre node at hub height, so the arm becomes
  almost horizontal and chassis vertical motion barely changes its
  length (< 0.5 % strain).
