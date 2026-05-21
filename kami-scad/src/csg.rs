//! CSG mesh generation: cylinder + future boolean ops.

use kami_render::mesh::{LoadedMesh, interleave};

/// Generate a cylinder mesh (truncated cone). Y-axis aligned, centered at origin.
pub fn cylinder_mesh(
    h: f32,
    r1: f32,
    r2: f32,
    slices: u32,
) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<u32>) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let half_h = h / 2.0;

    // Side vertices
    for i in 0..=slices {
        let theta = 2.0 * std::f32::consts::PI * i as f32 / slices as f32;
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        let u = i as f32 / slices as f32;

        // Bottom ring
        let x0 = r1 * cos_t;
        let z0 = r1 * sin_t;
        positions.extend_from_slice(&[x0, -half_h, z0]);

        // Top ring
        let x1 = r2 * cos_t;
        let z1 = r2 * sin_t;
        positions.extend_from_slice(&[x1, half_h, z1]);

        // Normal: approximate for truncated cone
        let slope = (r1 - r2) / h;
        let nx = cos_t;
        let ny = slope;
        let nz = sin_t;
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        normals.extend_from_slice(&[nx / len, ny / len, nz / len]);
        normals.extend_from_slice(&[nx / len, ny / len, nz / len]);

        uvs.extend_from_slice(&[u, 0.0]);
        uvs.extend_from_slice(&[u, 1.0]);
    }

    // Side indices
    for i in 0..slices {
        let b = i * 2;
        indices.extend_from_slice(&[b, b + 2, b + 1, b + 1, b + 2, b + 3]);
    }

    // Bottom cap
    let center_bottom = positions.len() as u32 / 3;
    positions.extend_from_slice(&[0.0, -half_h, 0.0]);
    normals.extend_from_slice(&[0.0, -1.0, 0.0]);
    uvs.extend_from_slice(&[0.5, 0.5]);

    for i in 0..slices {
        let theta = 2.0 * std::f32::consts::PI * i as f32 / slices as f32;
        positions.extend_from_slice(&[r1 * theta.cos(), -half_h, r1 * theta.sin()]);
        normals.extend_from_slice(&[0.0, -1.0, 0.0]);
        uvs.extend_from_slice(&[0.5 + 0.5 * theta.cos(), 0.5 + 0.5 * theta.sin()]);
    }

    for i in 0..slices {
        let next = if i + 1 < slices { i + 1 } else { 0 };
        indices.extend_from_slice(&[
            center_bottom,
            center_bottom + 1 + next,
            center_bottom + 1 + i,
        ]);
    }

    // Top cap
    let center_top = positions.len() as u32 / 3;
    positions.extend_from_slice(&[0.0, half_h, 0.0]);
    normals.extend_from_slice(&[0.0, 1.0, 0.0]);
    uvs.extend_from_slice(&[0.5, 0.5]);

    for i in 0..slices {
        let theta = 2.0 * std::f32::consts::PI * i as f32 / slices as f32;
        positions.extend_from_slice(&[r2 * theta.cos(), half_h, r2 * theta.sin()]);
        normals.extend_from_slice(&[0.0, 1.0, 0.0]);
        uvs.extend_from_slice(&[0.5 + 0.5 * theta.cos(), 0.5 + 0.5 * theta.sin()]);
    }

    for i in 0..slices {
        let next = if i + 1 < slices { i + 1 } else { 0 };
        indices.extend_from_slice(&[center_top, center_top + 1 + i, center_top + 1 + next]);
    }

    (positions, normals, uvs, indices)
}

/// Convert cylinder mesh to LoadedMesh.
pub fn cylinder_loaded(h: f32, r1: f32, r2: f32, slices: u32) -> LoadedMesh {
    let (pos, norm, uv, idx) = cylinder_mesh(h, r1, r2, slices);
    kami_render::mesh::loaded_mesh(&pos, &norm, &uv, &idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cylinder_basic() {
        let (pos, norm, uv, idx) = cylinder_mesh(2.0, 1.0, 1.0, 16);
        assert!(!pos.is_empty());
        assert!(!idx.is_empty());
        assert_eq!(pos.len() / 3, norm.len() / 3);
        assert_eq!(pos.len() / 3, uv.len() / 2);
    }

    #[test]
    fn cylinder_cone() {
        let (pos, _, _, idx) = cylinder_mesh(3.0, 1.0, 0.0, 8);
        assert!(!pos.is_empty());
        assert!(!idx.is_empty());
    }

    #[test]
    fn cylinder_loaded_mesh() {
        let mesh = cylinder_loaded(2.0, 0.5, 0.5, 12);
        assert!(mesh.vertex_count > 0);
        assert!(mesh.index_count > 0);
        assert_eq!(mesh.vertices.len(), mesh.vertex_count as usize * 8);
    }
}
