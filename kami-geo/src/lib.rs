//! kami-geo: GIS primitives for KAMI Engine map rendering.
//!
//! Web Mercator projection, tile math, raster tile mesh, GeoJSON → mesh,
//! billboard definitions. No wasm-bindgen dependency — pure logic crate.

pub mod billboard;
pub mod mesh;
pub mod projection;
pub mod tile;
