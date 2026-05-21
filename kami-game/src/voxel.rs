//! Voxel volume storage: block types, chunk management, KAMI Column serialization.

use std::collections::HashMap;

pub const CHUNK_SIZE: usize = 16;
const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BlockType {
    Air = 0,
    Dirt = 1,
    Grass = 2,
    Stone = 3,
    Water = 4,
    Sand = 5,
    Wood = 6,
    Leaf = 7,
    Ore = 8,
    Brick = 9,
    Glass = 10,
    Metal = 11,
    Snow = 12,
    Lava = 13,
    Ice = 14,
    Gravel = 15,
}

impl BlockType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Dirt,
            2 => Self::Grass,
            3 => Self::Stone,
            4 => Self::Water,
            5 => Self::Sand,
            6 => Self::Wood,
            7 => Self::Leaf,
            8 => Self::Ore,
            9 => Self::Brick,
            10 => Self::Glass,
            11 => Self::Metal,
            12 => Self::Snow,
            13 => Self::Lava,
            14 => Self::Ice,
            15 => Self::Gravel,
            _ => Self::Air,
        }
    }

    pub fn is_solid(self) -> bool {
        self != Self::Air
    }

    pub fn is_transparent(self) -> bool {
        matches!(self, Self::Air | Self::Water | Self::Glass | Self::Ice)
    }
}

/// 16x16x16 voxel chunk.
pub struct VoxelChunk {
    blocks: [u8; CHUNK_VOLUME],
    dirty: bool,
}

impl VoxelChunk {
    pub fn new() -> Self {
        Self {
            blocks: [0; CHUNK_VOLUME],
            dirty: true,
        }
    }

    /// Create a solid chunk filled with one block type.
    pub fn solid(block: BlockType) -> Self {
        Self {
            blocks: [block as u8; CHUNK_VOLUME],
            dirty: true,
        }
    }

    #[inline]
    fn index(x: usize, y: usize, z: usize) -> usize {
        y * CHUNK_SIZE * CHUNK_SIZE + z * CHUNK_SIZE + x
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockType {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return BlockType::Air;
        }
        BlockType::from_u8(self.blocks[Self::index(x, y, z)])
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            self.blocks[Self::index(x, y, z)] = block as u8;
            self.dirty = true;
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Serialize to KAMI Column data (U8, stride 1, len = 4096).
    pub fn to_column(&self) -> Vec<u8> {
        self.blocks.to_vec()
    }

    /// Deserialize from KAMI Column data.
    pub fn from_column(data: &[u8]) -> Self {
        let mut blocks = [0u8; CHUNK_VOLUME];
        let len = data.len().min(CHUNK_VOLUME);
        blocks[..len].copy_from_slice(&data[..len]);
        Self {
            blocks,
            dirty: true,
        }
    }

    /// Count non-air blocks.
    pub fn solid_count(&self) -> usize {
        self.blocks.iter().filter(|&&b| b != 0).count()
    }
}

impl Default for VoxelChunk {
    fn default() -> Self {
        Self::new()
    }
}

/// Default block palette: block type → RGBA color.
pub fn default_palette() -> Vec<[f32; 4]> {
    vec![
        [0.0, 0.0, 0.0, 0.0],   // Air (transparent)
        [0.55, 0.35, 0.2, 1.0], // Dirt
        [0.3, 0.6, 0.2, 1.0],   // Grass
        [0.5, 0.5, 0.5, 1.0],   // Stone
        [0.2, 0.3, 0.8, 0.7],   // Water
        [0.85, 0.75, 0.5, 1.0], // Sand
        [0.45, 0.3, 0.15, 1.0], // Wood
        [0.2, 0.5, 0.1, 0.9],   // Leaf
        [0.6, 0.5, 0.3, 1.0],   // Ore
        [0.7, 0.3, 0.2, 1.0],   // Brick
        [0.8, 0.9, 1.0, 0.4],   // Glass
        [0.7, 0.7, 0.75, 1.0],  // Metal
        [0.95, 0.95, 1.0, 1.0], // Snow
        [1.0, 0.3, 0.0, 1.0],   // Lava
        [0.6, 0.8, 1.0, 0.8],   // Ice
        [0.6, 0.55, 0.5, 1.0],  // Gravel
    ]
}

/// World made of chunks at integer grid positions.
pub struct VoxelWorld {
    pub chunks: HashMap<[i32; 3], VoxelChunk>,
    pub palette: Vec<[f32; 4]>,
}

impl VoxelWorld {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            palette: default_palette(),
        }
    }

    pub fn set_block(&mut self, wx: i32, wy: i32, wz: i32, block: BlockType) {
        let cx = wx.div_euclid(CHUNK_SIZE as i32);
        let cy = wy.div_euclid(CHUNK_SIZE as i32);
        let cz = wz.div_euclid(CHUNK_SIZE as i32);
        let lx = wx.rem_euclid(CHUNK_SIZE as i32) as usize;
        let ly = wy.rem_euclid(CHUNK_SIZE as i32) as usize;
        let lz = wz.rem_euclid(CHUNK_SIZE as i32) as usize;

        let chunk = self
            .chunks
            .entry([cx, cy, cz])
            .or_insert_with(VoxelChunk::new);
        chunk.set(lx, ly, lz, block);
    }

    pub fn get_block(&self, wx: i32, wy: i32, wz: i32) -> BlockType {
        let cx = wx.div_euclid(CHUNK_SIZE as i32);
        let cy = wy.div_euclid(CHUNK_SIZE as i32);
        let cz = wz.div_euclid(CHUNK_SIZE as i32);
        let lx = wx.rem_euclid(CHUNK_SIZE as i32) as usize;
        let ly = wy.rem_euclid(CHUNK_SIZE as i32) as usize;
        let lz = wz.rem_euclid(CHUNK_SIZE as i32) as usize;

        self.chunks
            .get(&[cx, cy, cz])
            .map(|c| c.get(lx, ly, lz))
            .unwrap_or(BlockType::Air)
    }

    pub fn dirty_chunks(&self) -> Vec<[i32; 3]> {
        self.chunks
            .iter()
            .filter(|(_, c)| c.is_dirty())
            .map(|(k, _)| *k)
            .collect()
    }

    /// Generate flat terrain: layers of stone/dirt/grass.
    pub fn generate_flat(width: i32, depth: i32, height: i32) -> Self {
        let mut world = Self::new();
        for y in 0..height {
            let block = if y < height - 2 {
                BlockType::Stone
            } else if y < height - 1 {
                BlockType::Dirt
            } else {
                BlockType::Grass
            };
            for z in 0..depth {
                for x in 0..width {
                    world.set_block(x, y, z, block);
                }
            }
        }
        world
    }
}

impl Default for VoxelWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_set_get() {
        let mut chunk = VoxelChunk::new();
        assert_eq!(chunk.get(0, 0, 0), BlockType::Air);
        chunk.set(5, 10, 3, BlockType::Stone);
        assert_eq!(chunk.get(5, 10, 3), BlockType::Stone);
        assert_eq!(chunk.get(0, 0, 0), BlockType::Air);
    }

    #[test]
    fn chunk_bounds() {
        let chunk = VoxelChunk::new();
        assert_eq!(chunk.get(16, 0, 0), BlockType::Air); // out of bounds
        assert_eq!(chunk.get(0, 16, 0), BlockType::Air);
    }

    #[test]
    fn chunk_column_roundtrip() {
        let mut chunk = VoxelChunk::new();
        chunk.set(0, 0, 0, BlockType::Grass);
        chunk.set(15, 15, 15, BlockType::Stone);
        let data = chunk.to_column();
        assert_eq!(data.len(), CHUNK_VOLUME);
        let restored = VoxelChunk::from_column(&data);
        assert_eq!(restored.get(0, 0, 0), BlockType::Grass);
        assert_eq!(restored.get(15, 15, 15), BlockType::Stone);
        assert_eq!(restored.get(1, 1, 1), BlockType::Air);
    }

    #[test]
    fn chunk_solid() {
        let chunk = VoxelChunk::solid(BlockType::Dirt);
        assert_eq!(chunk.get(0, 0, 0), BlockType::Dirt);
        assert_eq!(chunk.get(8, 8, 8), BlockType::Dirt);
        assert_eq!(chunk.solid_count(), CHUNK_VOLUME);
    }

    #[test]
    fn world_set_get() {
        let mut world = VoxelWorld::new();
        world.set_block(5, 10, 3, BlockType::Stone);
        assert_eq!(world.get_block(5, 10, 3), BlockType::Stone);
        assert_eq!(world.get_block(0, 0, 0), BlockType::Air);
    }

    #[test]
    fn world_negative_coords() {
        let mut world = VoxelWorld::new();
        world.set_block(-1, -1, -1, BlockType::Ore);
        assert_eq!(world.get_block(-1, -1, -1), BlockType::Ore);
    }

    #[test]
    fn world_flat_generation() {
        let world = VoxelWorld::generate_flat(16, 16, 4);
        assert_eq!(world.get_block(0, 0, 0), BlockType::Stone);
        assert_eq!(world.get_block(0, 2, 0), BlockType::Dirt);
        assert_eq!(world.get_block(0, 3, 0), BlockType::Grass);
        assert_eq!(world.get_block(0, 4, 0), BlockType::Air);
    }

    #[test]
    fn palette_size() {
        let palette = default_palette();
        assert_eq!(palette.len(), 16);
    }
}
