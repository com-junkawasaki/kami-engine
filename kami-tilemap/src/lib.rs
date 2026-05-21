//! kami-tilemap: 2D tilemap renderer.
//!
//! Tile layers, auto-tile rules, animated tiles, collision map.
//! Designed for RPG/platformer/strategy games.

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Single tile.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tile {
    pub id: u16,   // tile atlas index (0 = empty)
    pub flags: u8, // bit 0: solid, bit 1: flip_x, bit 2: flip_y, bit 3: animated
}

impl Tile {
    pub const EMPTY: Tile = Tile { id: 0, flags: 0 };
    pub fn solid(id: u16) -> Self {
        Tile { id, flags: 1 }
    }
    pub fn is_empty(&self) -> bool {
        self.id == 0
    }
    pub fn is_solid(&self) -> bool {
        self.flags & 1 != 0
    }
}

/// Tile layer (one z-level).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileLayer {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>, // row-major [y * width + x]
    pub visible: bool,
    pub opacity: f32,
    pub z_order: i32,
}

impl TileLayer {
    pub fn new(name: &str, w: u32, h: u32) -> Self {
        Self {
            name: name.to_string(),
            width: w,
            height: h,
            tiles: vec![Tile::EMPTY; (w * h) as usize],
            visible: true,
            opacity: 1.0,
            z_order: 0,
        }
    }

    pub fn get(&self, x: u32, y: u32) -> Tile {
        if x < self.width && y < self.height {
            self.tiles[(y * self.width + x) as usize]
        } else {
            Tile::EMPTY
        }
    }

    pub fn set(&mut self, x: u32, y: u32, tile: Tile) {
        if x < self.width && y < self.height {
            self.tiles[(y * self.width + x) as usize] = tile;
        }
    }
}

/// Tilemap: collection of layers + metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tilemap {
    pub tile_size: f32, // pixels per tile
    pub layers: Vec<TileLayer>,
}

impl Tilemap {
    pub fn new(tile_size: f32) -> Self {
        Self {
            tile_size,
            layers: Vec::new(),
        }
    }

    /// World position → tile coordinate.
    pub fn world_to_tile(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.tile_size).floor() as i32,
            (pos.y / self.tile_size).floor() as i32,
        )
    }

    /// Tile coordinate → world center position.
    pub fn tile_to_world(&self, tx: i32, ty: i32) -> Vec2 {
        Vec2::new(
            (tx as f32 + 0.5) * self.tile_size,
            (ty as f32 + 0.5) * self.tile_size,
        )
    }

    /// Check if any layer has a solid tile at (tx, ty).
    pub fn is_solid(&self, tx: i32, ty: i32) -> bool {
        self.layers.iter().any(|l| {
            if tx >= 0 && ty >= 0 {
                l.get(tx as u32, ty as u32).is_solid()
            } else {
                false
            }
        })
    }

    /// Generate instanced tile positions for GPU rendering.
    pub fn to_instances(&self, layer_idx: usize) -> Vec<[f32; 4]> {
        let layer = &self.layers[layer_idx];
        let mut instances = Vec::new();
        for y in 0..layer.height {
            for x in 0..layer.width {
                let tile = layer.get(x, y);
                if tile.is_empty() {
                    continue;
                }
                instances.push([
                    x as f32 * self.tile_size,
                    y as f32 * self.tile_size,
                    self.tile_size,
                    tile.id as f32,
                ]);
            }
        }
        instances
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tilemap() {
        let mut map = Tilemap::new(16.0);
        let mut layer = TileLayer::new("ground", 10, 10);
        layer.set(5, 3, Tile::solid(1));
        map.layers.push(layer);
        assert!(map.is_solid(5, 3));
        assert!(!map.is_solid(0, 0));
        let (tx, ty) = map.world_to_tile(Vec2::new(85.0, 50.0));
        assert_eq!((tx, ty), (5, 3));
    }
}
