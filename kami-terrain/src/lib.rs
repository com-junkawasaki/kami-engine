//! kami-terrain: Decima-style heightmap terrain engine.
//!
//! Procedural generation (value noise + FBM), clipmap LOD, splatmap material
//! blending, and chunk-based mesh generation for open-world rendering.
//!
//! Design reference: Guerrilla Games Decima Engine (Horizon Zero Dawn).

pub mod heightmap;
pub mod chunk;
pub mod noise;
pub mod splatmap;
pub mod water;
pub mod biome;

pub use heightmap::{Heightmap, HeightmapConfig};
pub use chunk::{TerrainChunk, TerrainVertex, generate_chunk_mesh};
pub use noise::fbm_noise;
pub use splatmap::Splatmap;
pub use water::{WaterConfig, WaterVertex, GerstnerWave, generate_water_mesh, default_waves, waves_from_wind};
pub use biome::{BiomePreset, SplatThresholds, MaterialPalette};
