//! Web Mercator (EPSG:3857) projection utilities.
//!
//! Converts between WGS84 (lng/lat degrees) and world-space coordinates
//! used by the KAMI map renderer.  World origin = (0, 0) at tile (0,0)
//! of the current zoom level; one tile = 1.0 world unit.

use std::f64::consts::PI;

/// WGS84 coordinate.
#[derive(Debug, Clone, Copy)]
pub struct LngLat {
    pub lng: f64,
    pub lat: f64,
}

impl LngLat {
    pub fn new(lng: f64, lat: f64) -> Self {
        Self { lng, lat }
    }
}

/// Pixel coordinate at a given zoom (256 px per tile).
#[derive(Debug, Clone, Copy)]
pub struct WorldPx {
    pub x: f64,
    pub y: f64,
}

/// Convert WGS84 → world pixel at the given zoom level.
/// Origin (0,0) = top-left of the world, +x east, +y south.
pub fn lng_lat_to_world_px(ll: LngLat, zoom: f64) -> WorldPx {
    let scale = 256.0 * 2.0_f64.powf(zoom);
    let x = (ll.lng + 180.0) / 360.0 * scale;
    let lat_rad = ll.lat.to_radians();
    let y = (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / PI) / 2.0 * scale;
    WorldPx { x, y }
}

/// Convert world pixel → WGS84 at the given zoom level.
pub fn world_px_to_lng_lat(wp: WorldPx, zoom: f64) -> LngLat {
    let scale = 256.0 * 2.0_f64.powf(zoom);
    let lng = wp.x / scale * 360.0 - 180.0;
    let n = PI - 2.0 * PI * wp.y / scale;
    let lat = (0.5 * (n.exp() - (-n).exp())).atan().to_degrees();
    LngLat { lng, lat }
}

/// Clamp latitude to the Web Mercator representable range.
pub fn clamp_lat(lat: f64) -> f64 {
    lat.clamp(-85.051_128_78, 85.051_128_78)
}

/// Convert world pixel → 3D world position for the renderer.
/// The map camera looks down the -Y axis.
/// World coords: X = east, Z = south, Y = up (elevation).
/// `center_px` is subtracted so the camera center sits at (0, 0, 0).
pub fn world_px_to_3d(wp: WorldPx, center_px: WorldPx) -> [f32; 3] {
    let x = (wp.x - center_px.x) as f32;
    let z = (wp.y - center_px.y) as f32;
    [x, 0.0, z]
}

/// Tile coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileCoord {
    pub z: u32,
    pub x: u32,
    pub y: u32,
}

impl TileCoord {
    /// Top-left world pixel of this tile.
    pub fn origin_px(&self) -> WorldPx {
        let tile_size = 256.0;
        WorldPx {
            x: self.x as f64 * tile_size,
            y: self.y as f64 * tile_size,
        }
    }

    /// Build a URL from a template like `https://tile.openstreetmap.org/{z}/{x}/{y}.png`.
    pub fn url(&self, template: &str) -> String {
        template
            .replace("{z}", &self.z.to_string())
            .replace("{x}", &self.x.to_string())
            .replace("{y}", &self.y.to_string())
    }

    /// Geographic bounds of this Web Mercator tile as (west, south, east, north).
    pub fn lng_lat_bounds(&self) -> (f64, f64, f64, f64) {
        let n = 2.0_f64.powi(self.z as i32);
        let west = self.x as f64 / n * 360.0 - 180.0;
        let east = (self.x as f64 + 1.0) / n * 360.0 - 180.0;
        let north = mercator_tile_y_to_lat(self.y as f64, n);
        let south = mercator_tile_y_to_lat(self.y as f64 + 1.0, n);
        (west, south, east, north)
    }

    pub fn center_lng_lat(&self) -> LngLat {
        let (west, south, east, north) = self.lng_lat_bounds();
        LngLat::new((west + east) * 0.5, (south + north) * 0.5)
    }
}

fn mercator_tile_y_to_lat(y: f64, n: f64) -> f64 {
    let merc_n = PI * (1.0 - 2.0 * y / n);
    merc_n.sinh().atan().to_degrees()
}

/// Compute the set of tile coordinates visible in the given viewport.
///
/// `center` — WGS84 center of the viewport.
/// `zoom` — fractional zoom level (0..22).
/// `width`, `height` — viewport size in CSS pixels.
///
/// Returns tiles for the integer zoom floor, padded by 1 tile each side.
pub fn visible_tiles(center: LngLat, zoom: f64, width: u32, height: u32) -> Vec<TileCoord> {
    let iz = zoom.floor() as u32;
    let max_tile = 1u32 << iz;

    let center_px = lng_lat_to_world_px(center, iz as f64);
    let hw = width as f64 / 2.0;
    let hh = height as f64 / 2.0;

    // Pixel bounds + 1 tile padding
    let pad = 256.0;
    let left = (center_px.x - hw - pad).max(0.0);
    let right = (center_px.x + hw + pad).min(max_tile as f64 * 256.0);
    let top = (center_px.y - hh - pad).max(0.0);
    let bottom = (center_px.y + hh + pad).min(max_tile as f64 * 256.0);

    let x_min = (left / 256.0).floor() as u32;
    let x_max = ((right / 256.0).ceil() as u32).min(max_tile);
    let y_min = (top / 256.0).floor() as u32;
    let y_max = ((bottom / 256.0).ceil() as u32).min(max_tile);

    let mut tiles = Vec::with_capacity(((x_max - x_min) * (y_max - y_min)) as usize);
    for ty in y_min..y_max {
        for tx in x_min..x_max {
            tiles.push(TileCoord {
                z: iz,
                x: tx,
                y: ty,
            });
        }
    }
    tiles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_projection() {
        let ll = LngLat::new(139.7671, 35.6812);
        let wp = lng_lat_to_world_px(ll, 12.0);
        let ll2 = world_px_to_lng_lat(wp, 12.0);
        assert!((ll.lng - ll2.lng).abs() < 1e-6);
        assert!((ll.lat - ll2.lat).abs() < 1e-6);
    }

    #[test]
    fn tile_url() {
        let tc = TileCoord {
            z: 12,
            x: 3637,
            y: 1612,
        };
        let url = tc.url("https://tile.openstreetmap.org/{z}/{x}/{y}.png");
        assert_eq!(url, "https://tile.openstreetmap.org/12/3637/1612.png");
    }

    #[test]
    fn visible_tiles_basic() {
        let tiles = visible_tiles(LngLat::new(0.0, 0.0), 2.0, 512, 512);
        assert!(!tiles.is_empty());
        for t in &tiles {
            assert_eq!(t.z, 2);
            assert!(t.x < 4);
            assert!(t.y < 4);
        }
    }
}
