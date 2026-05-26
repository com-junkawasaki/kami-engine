# kami-sensor-sim

Robotics sensor synthesis on WebGPU compute — `isaacsim.sensors` API-compat target.

**Status**: R1.0 path reservation (ADR-2605261800).

## Scope (R1.6 deliverable for wadachi-sim DriveSim parity)

- RGB / depth camera (kami-render + depth attachment)
- Lidar (wgpu ray casting; 16-beam / 64-beam / solid-state)
- IMU (rigid body state from kami-genesis)
- Contact / force sensor

## License

Apache 2.0 + Charter Compliance Rider v2.0.
