//! Billboard definitions for markers, labels, and icons.
//!
//! Billboards are screen-space quads that always face the camera.
//! The renderer handles the actual GPU pipeline; this module defines the data.

/// A billboard instance to be rendered.
#[derive(Debug, Clone)]
pub struct BillboardDef {
    /// World-space position (X east, Y up, Z south).
    pub position: [f32; 3],
    /// Screen-space size in CSS pixels [width, height].
    pub size: [f32; 2],
    /// Anchor offset from center: (0,0) = center, (0,-0.5) = bottom-center.
    pub anchor: [f32; 2],
    /// RGBA tint color.
    pub color: [f32; 4],
    /// Optional texture atlas index (0 = solid color).
    pub atlas_index: u32,
}

impl Default for BillboardDef {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            size: [16.0, 16.0],
            anchor: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            atlas_index: 0,
        }
    }
}

/// GPU-side billboard instance data (48 bytes).
/// Laid out for a single vertex buffer with per-instance step mode.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BillboardInstance {
    pub position: [f32; 3],
    pub atlas_index: f32, // as u32 bits, but f32 for alignment
    pub size: [f32; 2],
    pub anchor: [f32; 2],
    pub color: [f32; 4],
}

impl From<&BillboardDef> for BillboardInstance {
    fn from(def: &BillboardDef) -> Self {
        Self {
            position: def.position,
            atlas_index: f32::from_bits(def.atlas_index),
            size: def.size,
            anchor: def.anchor,
            color: def.color,
        }
    }
}
