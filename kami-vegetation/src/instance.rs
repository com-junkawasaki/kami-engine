//! Per-plant instance data (GPU-uploadable).

use bytemuck::{Pod, Zeroable};

/// Per-instance data (8 floats = 32 bytes, GPU-aligned).
/// Uploaded once at scene generation; stays static (wind animation is shader-side).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct InstanceData {
    /// World-space position.
    pub position: [f32; 3],
    /// Uniform scale (vertical).
    pub scale: f32,
    /// Rotation around Y axis (radians).
    pub rotation: f32,
    /// Species ID (see SpeciesId enum).
    pub species: f32,
    /// Random wind phase offset [0, 2π].
    pub wind_phase: f32,
    /// Random color variation [-0.15, +0.15].
    pub color_tint: f32,
}

impl InstanceData {
    pub const STRIDE: usize = 32;
}
