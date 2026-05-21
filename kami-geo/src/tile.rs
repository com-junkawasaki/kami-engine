//! Tile management: LRU cache, fetch state, LOD selection.

use crate::projection::{TileCoord, WorldPx};
use std::collections::HashMap;

/// Maximum tiles to keep in GPU memory.
pub const MAX_CACHED_TILES: usize = 512;

/// Tile state in the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileState {
    /// Fetch has been requested but not yet completed.
    Pending,
    /// Image decoded and GPU texture uploaded.
    Ready,
    /// Fetch failed (will retry after cooldown).
    Failed,
}

/// Per-tile metadata in the tile cache.
pub struct CachedTile {
    pub coord: TileCoord,
    pub state: TileState,
    /// Renderer mesh handle for the textured quad (set when Ready).
    pub mesh_handle: Option<u32>,
    /// Renderer texture handle (set when Ready).
    pub texture_handle: Option<u32>,
    /// Frame number when last used (for LRU eviction).
    pub last_used_frame: u64,
}

/// Tile manager tracks which tiles are loaded, pending, or evicted.
pub struct TileManager {
    pub cache: HashMap<TileCoord, CachedTile>,
    pub tile_url_template: String,
    frame_counter: u64,
}

impl TileManager {
    pub fn new(tile_url_template: String) -> Self {
        Self {
            cache: HashMap::with_capacity(MAX_CACHED_TILES),
            tile_url_template,
            frame_counter: 0,
        }
    }

    /// Advance frame counter (call once per frame).
    pub fn begin_frame(&mut self) {
        self.frame_counter += 1;
    }

    /// Mark a tile as needed this frame.  Returns true if the tile is Ready.
    pub fn touch(&mut self, coord: TileCoord) -> bool {
        if let Some(entry) = self.cache.get_mut(&coord) {
            entry.last_used_frame = self.frame_counter;
            entry.state == TileState::Ready
        } else {
            false
        }
    }

    /// Tiles that need to be fetched (not in cache or failed long ago).
    pub fn tiles_to_fetch(&mut self, visible: &[TileCoord]) -> Vec<TileCoord> {
        let mut to_fetch = Vec::new();
        for &coord in visible {
            if !self.cache.contains_key(&coord) {
                self.cache.insert(
                    coord,
                    CachedTile {
                        coord,
                        state: TileState::Pending,
                        mesh_handle: None,
                        texture_handle: None,
                        last_used_frame: self.frame_counter,
                    },
                );
                to_fetch.push(coord);
            }
        }
        to_fetch
    }

    /// Mark a tile as ready with its GPU handles.
    pub fn mark_ready(&mut self, coord: TileCoord, mesh_handle: u32, texture_handle: u32) {
        if let Some(entry) = self.cache.get_mut(&coord) {
            entry.state = TileState::Ready;
            entry.mesh_handle = Some(mesh_handle);
            entry.texture_handle = Some(texture_handle);
        }
    }

    /// Mark a tile fetch as failed.
    pub fn mark_failed(&mut self, coord: TileCoord) {
        if let Some(entry) = self.cache.get_mut(&coord) {
            entry.state = TileState::Failed;
        }
    }

    /// Evict least-recently-used tiles when cache exceeds limit.
    /// Returns evicted texture handles for GPU cleanup.
    pub fn evict(&mut self) -> Vec<u32> {
        if self.cache.len() <= MAX_CACHED_TILES {
            return Vec::new();
        }

        let mut entries: Vec<(TileCoord, u64, Option<u32>)> = self
            .cache
            .iter()
            .map(|(k, v)| (*k, v.last_used_frame, v.texture_handle))
            .collect();
        entries.sort_by_key(|e| e.1);

        let to_remove = self.cache.len() - MAX_CACHED_TILES;
        let mut evicted_textures = Vec::new();
        for (coord, _, tex) in entries.iter().take(to_remove) {
            if let Some(th) = tex {
                evicted_textures.push(*th);
            }
            self.cache.remove(coord);
        }
        evicted_textures
    }

    /// Get ready tiles that should be drawn.
    pub fn ready_tiles(&self) -> Vec<&CachedTile> {
        self.cache
            .values()
            .filter(|t| t.state == TileState::Ready)
            .collect()
    }
}

/// Build the world-space position (top-left corner) for a tile quad.
pub fn tile_world_position(coord: TileCoord, center_px: WorldPx) -> [f32; 3] {
    let origin = coord.origin_px();
    let x = (origin.x - center_px.x) as f32;
    let z = (origin.y - center_px.y) as f32;
    [x, 0.0, z]
}
