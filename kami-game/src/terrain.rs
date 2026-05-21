//! Heightmap terrain generation and LOD mesh output.

/// Heightmap-based terrain.
pub struct HeightmapTerrain {
    pub heights: Vec<f32>,
    pub width: u32,
    pub depth: u32,
    pub height_scale: f32,
    pub cell_size: f32,
}

/// Terrain mesh output (interleaved pos3 + norm3 + uv2 = 32B/vertex).
pub struct TerrainMesh {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub vertex_count: u32,
    pub index_count: u32,
}

impl HeightmapTerrain {
    pub fn new(width: u32, depth: u32, height_scale: f32, cell_size: f32) -> Self {
        Self {
            heights: vec![0.0; (width * depth) as usize],
            width,
            depth,
            height_scale,
            cell_size,
        }
    }

    /// Get height at grid position.
    fn height_at(&self, x: u32, z: u32) -> f32 {
        if x < self.width && z < self.depth {
            self.heights[(z * self.width + x) as usize] * self.height_scale
        } else {
            0.0
        }
    }

    /// Sample height at world position with bilinear interpolation.
    pub fn sample_height(&self, wx: f32, wz: f32) -> f32 {
        let gx = wx / self.cell_size;
        let gz = wz / self.cell_size;

        let x0 = gx.floor() as i32;
        let z0 = gz.floor() as i32;
        let fx = gx - gx.floor();
        let fz = gz - gz.floor();

        let clamp = |v: i32, max: u32| v.clamp(0, max as i32 - 1) as u32;
        let h00 = self.height_at(clamp(x0, self.width), clamp(z0, self.depth));
        let h10 = self.height_at(clamp(x0 + 1, self.width), clamp(z0, self.depth));
        let h01 = self.height_at(clamp(x0, self.width), clamp(z0 + 1, self.depth));
        let h11 = self.height_at(clamp(x0 + 1, self.width), clamp(z0 + 1, self.depth));

        let a = h00 * (1.0 - fx) + h10 * fx;
        let b = h01 * (1.0 - fx) + h11 * fx;
        a * (1.0 - fz) + b * fz
    }

    /// Generate full-resolution mesh.
    pub fn to_mesh(&self) -> TerrainMesh {
        self.to_mesh_lod(0)
    }

    /// Generate mesh at given LOD level. Level 0 = full, each level halves resolution.
    pub fn to_mesh_lod(&self, lod_level: u32) -> TerrainMesh {
        let step = 1u32 << lod_level;
        let cols = ((self.width - 1) / step) + 1;
        let rows = ((self.depth - 1) / step) + 1;

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        for iz in 0..rows {
            for ix in 0..cols {
                let gx = (ix * step).min(self.width - 1);
                let gz = (iz * step).min(self.depth - 1);

                let x = gx as f32 * self.cell_size;
                let z = gz as f32 * self.cell_size;
                let y = self.height_at(gx, gz);

                positions.extend_from_slice(&[x, y, z]);
                uvs.extend_from_slice(&[
                    gx as f32 / (self.width - 1).max(1) as f32,
                    gz as f32 / (self.depth - 1).max(1) as f32,
                ]);

                // Compute normal from neighbors (central difference)
                let hx0 = if gx > 0 {
                    self.height_at(gx - 1, gz)
                } else {
                    y
                };
                let hx1 = if gx + 1 < self.width {
                    self.height_at(gx + 1, gz)
                } else {
                    y
                };
                let hz0 = if gz > 0 {
                    self.height_at(gx, gz - 1)
                } else {
                    y
                };
                let hz1 = if gz + 1 < self.depth {
                    self.height_at(gx, gz + 1)
                } else {
                    y
                };

                let dx = hx1 - hx0;
                let dz = hz1 - hz0;
                let nx = -dx;
                let ny = 2.0 * self.cell_size;
                let nz = -dz;
                let len = (nx * nx + ny * ny + nz * nz).sqrt();
                normals.extend_from_slice(&[nx / len, ny / len, nz / len]);
            }
        }

        for iz in 0..rows - 1 {
            for ix in 0..cols - 1 {
                let a = iz * cols + ix;
                let b = a + cols;
                indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
            }
        }

        let vertex_count = (cols * rows) as u32;
        let index_count = indices.len() as u32;

        // Interleave
        let mut vertices = Vec::with_capacity(vertex_count as usize * 8);
        for i in 0..vertex_count as usize {
            vertices.extend_from_slice(&positions[i * 3..i * 3 + 3]);
            vertices.extend_from_slice(&normals[i * 3..i * 3 + 3]);
            vertices.extend_from_slice(&uvs[i * 2..i * 2 + 2]);
        }

        TerrainMesh {
            vertices,
            indices,
            vertex_count,
            index_count,
        }
    }

    /// Generate terrain from simple value noise (no external dependency).
    pub fn from_noise(width: u32, depth: u32, seed: u64) -> Self {
        let mut terrain = Self::new(width, depth, 1.0, 1.0);
        let mut rng = seed;
        for z in 0..depth {
            for x in 0..width {
                // Simple hash-based noise
                rng = rng
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let r = ((rng >> 33) as f32) / (u32::MAX as f32);

                // Layer octaves for more natural terrain
                let fx = x as f32 / width as f32;
                let fz = z as f32 / depth as f32;
                let h = r * 0.3
                    + (fx * 3.14 * 2.0).sin() * 0.2
                    + (fz * 3.14 * 3.0).cos() * 0.15
                    + ((fx + fz) * 3.14 * 5.0).sin() * 0.1;

                terrain.heights[(z * width + x) as usize] = h.max(0.0);
            }
        }
        terrain
    }

    /// Generate from raw R16 heightmap data (16-bit unsigned, little-endian).
    pub fn from_r16(data: &[u8], width: u32, depth: u32, height_scale: f32) -> Self {
        let mut terrain = Self::new(width, depth, height_scale, 1.0);
        let expected = (width * depth * 2) as usize;
        let len = data.len().min(expected);
        for i in 0..len / 2 {
            let val = u16::from_le_bytes([data[i * 2], data[i * 2 + 1]]);
            terrain.heights[i] = val as f32 / 65535.0;
        }
        terrain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_terrain_mesh() {
        let terrain = HeightmapTerrain::new(4, 4, 1.0, 1.0);
        let mesh = terrain.to_mesh();
        assert_eq!(mesh.vertex_count, 16); // 4×4
        assert_eq!(mesh.index_count, 9 * 6); // 3×3 quads × 6 indices
        assert_eq!(mesh.vertices.len(), 16 * 8); // 8 floats per vertex
    }

    #[test]
    fn lod_reduces_vertices() {
        let terrain = HeightmapTerrain::new(17, 17, 1.0, 1.0);
        let lod0 = terrain.to_mesh_lod(0);
        let lod1 = terrain.to_mesh_lod(1);
        let lod2 = terrain.to_mesh_lod(2);
        assert!(
            lod1.vertex_count < lod0.vertex_count,
            "lod1 {} should be < lod0 {}",
            lod1.vertex_count,
            lod0.vertex_count
        );
        assert!(
            lod2.vertex_count < lod1.vertex_count,
            "lod2 {} should be < lod1 {}",
            lod2.vertex_count,
            lod1.vertex_count
        );
    }

    #[test]
    fn sample_height_bilinear() {
        let mut terrain = HeightmapTerrain::new(3, 3, 1.0, 1.0);
        terrain.heights = vec![0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0];
        let center = terrain.sample_height(1.0, 1.0);
        assert!((center - 4.0).abs() < 0.01, "center height: {}", center);
        let mid = terrain.sample_height(0.5, 0.5);
        assert!((mid - 1.0).abs() < 0.01, "mid height: {}", mid);
    }

    #[test]
    fn from_noise_generates() {
        let terrain = HeightmapTerrain::from_noise(32, 32, 42);
        assert_eq!(terrain.heights.len(), 32 * 32);
        assert!(terrain.heights.iter().any(|&h| h > 0.0));
        let mesh = terrain.to_mesh();
        assert!(mesh.vertex_count > 0);
    }

    #[test]
    fn from_r16_parse() {
        // 2x2 heightmap, all max value
        let data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let terrain = HeightmapTerrain::from_r16(&data, 2, 2, 10.0);
        assert_eq!(terrain.heights.len(), 4);
        assert!((terrain.heights[0] - 1.0).abs() < 0.001);
    }
}
