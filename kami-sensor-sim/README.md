# kami-sensor-sim

Robotics sensor synthesis on WebGPU compute — `isaacsim.sensors` API-compat target.

**Status**: R1.x active — all four sensor types implemented (CPU; WebGPU-compute
acceleration is the remaining R1.6 work). `isaacsim.sensors.{Camera, LidarRtx,
IMUSensor, ContactSensor}` API shapes.

## Implemented (R1.x)

- **Camera** — pinhole intrinsics/extrinsics + 3D-point projection + depth image;
  `look_at` mounting + observes a kami-genesis link (see `tests/camera_on_genesis.rs`:
  projected depth tracks a driven cart's distance along the optical axis).
- **Lidar** — analytic ray-vs-primitive scene (VLP-16 intrinsics; 16 / 64-beam /
  solid-state patterns), mountable on a kami-genesis link via its `view`
  transform (see `tests/lidar_on_genesis.rs`: a cart-mounted lidar's range to a
  fixed obstacle tracks the cart's motion).
- **IMU** — body-frame proper acceleration (finite-diff) + angular velocity +
  orientation. Physics-engine-agnostic: takes world-frame link state as input,
  so it reads straight off `kami_genesis::ArticulationView::{get_world_pose,
  get_world_velocity}` (see `tests/imu_on_genesis.rs` for the end-to-end rig).
- **ContactSensor** — link-sphere vs scene-primitive overlap → `in_contact`,
  penetration depth + normal. `sample` returns the nearest surface; `sample_all`
  reports every simultaneous contact (deepest-first), as Isaac does. Mountable
  on a kami-genesis link (see `tests/contact_on_genesis.rs`: a cart driven into
  an obstacle trips contact and the penetration grows with depth).

All four sensors have an end-to-end `tests/*_on_genesis.rs` rig confirming they
read kami-genesis link state and sense the physics scene correctly.

## Remaining (R1.6, wadachi-sim DriveSim parity)

- WebGPU-compute acceleration (lidar ray-query against scene BVH; raster depth)
- Contact wiring from kami-genesis collision events

## License

Apache 2.0 + Charter Compliance Rider v2.0.
