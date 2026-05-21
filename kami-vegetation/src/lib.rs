//! kami-vegetation: Decima-style procedural vegetation.
//!
//! Poisson-disk placement + biome rules (height + slope + splatmap) + GPU
//! instancing + wind-driven vertex animation.

pub mod species;
pub mod placement;
pub mod instance;
pub mod lod;
pub mod cull;
pub mod mesh;
pub mod taxonomy;

pub use species::{Species, SpeciesId, species_table};
pub use placement::{PlacementConfig, place_instances};
pub use instance::InstanceData;
pub use lod::{LodTier, classify_lod};
pub use cull::{cull_by_distance, cull_to_buffer};
