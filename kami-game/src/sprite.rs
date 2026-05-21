//! Sprite2D: 2D sprite representation for side-scroll mode.
//! Converts to SceneEntity (Plane mesh + material) for rendering via existing PBR pipeline.

use std::f32::consts::PI;

/// A 2D sprite with position, size, layer depth, and optional texture.
#[derive(Clone, Debug)]
pub struct Sprite2D {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub layer: f32,
    pub color: [f32; 4],
    pub texture_key: Option<String>,
    pub flip_x: bool,
    pub frame: u32,
    pub frames_total: u32,
}

impl Default for Sprite2D {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            size: [1.0, 1.0],
            layer: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            texture_key: None,
            flip_x: false,
            frame: 0,
            frames_total: 1,
        }
    }
}

/// Z-depth layer definition for parallax scrolling.
#[derive(Clone, Debug)]
pub struct Layer2D {
    pub name: String,
    pub z: f32,
    pub parallax: f32,
    pub color: Option<[f32; 4]>,
}

/// Viewport definition for 2D mode.
#[derive(Clone, Debug)]
pub struct Viewport2D {
    pub width: f32,
    pub height: f32,
    pub pixels_per_unit: f32,
}

impl Default for Viewport2D {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 450.0,
            pixels_per_unit: 32.0,
        }
    }
}

/// Apply parallax offset to an entity's X position based on camera and layer parallax factor.
pub fn parallax_offset(entity_x: f32, camera_x: f32, parallax: f32) -> f32 {
    camera_x + (entity_x - camera_x) * parallax
}
