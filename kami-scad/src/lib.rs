//! kami-scad: OpenSCAD parser + CSG evaluator + pipeline orchestrator.
//!
//! Re-exports from sub-crates:
//!   kami-voxel  (volume layer)
//!   kami-sdf    (SDF generation)
//!   kami-mesher (mesh output)
//!   kami-gltf   (GLB export)
//!
//! Pipeline: OpenSCAD code → ScadEntity[] → SDF → VoxelVolume → Mesh/GLB

pub mod csg;
pub mod evaluator;
pub mod parser;

pub use csg::cylinder_mesh;
pub use evaluator::{ScadEntity, ScadPrimitive, evaluate};
pub use parser::{ScadNode, parse};

// Re-exports from sub-crates
pub use kami_gltf as gltf;
pub use kami_mesher as mesher;
pub use kami_sdf as sdf;
pub use kami_voxel as voxel;

/// Full pipeline: OpenSCAD code → GLB bytes.
pub fn scad_to_glb(code: &str, resolution: u32, bounds: f32, scale: f32) -> Vec<u8> {
    let entities = evaluate(code);
    let sdf_tree = entities_to_sdf(&entities);
    let volume = kami_sdf::sample_sdf(&sdf_tree, resolution, bounds);
    let mesh = kami_mesher::marching_cubes(&volume, scale);
    let color = entities.first().map(|e| e.color).unwrap_or([0.5; 4]);
    kami_gltf::export_glb(&mesh, color)
}

/// Full pipeline: OpenSCAD code → LoadedMesh.
pub fn scad_to_mesh(
    code: &str,
    resolution: u32,
    bounds: f32,
    scale: f32,
) -> kami_render::mesh::LoadedMesh {
    let entities = evaluate(code);
    let sdf_tree = entities_to_sdf(&entities);
    let volume = kami_sdf::sample_sdf(&sdf_tree, resolution, bounds);
    kami_mesher::marching_cubes(&volume, scale)
}

/// Convert ScadEntity list to SDF tree (smooth union).
pub fn entities_to_sdf(entities: &[ScadEntity]) -> kami_sdf::SdfNode {
    use glam::{Mat4, Quat, Vec3};
    let mut children = Vec::new();
    for e in entities {
        let prim = match &e.primitive {
            ScadPrimitive::Sphere { radius } => kami_sdf::SdfPrimitive::Sphere { radius: *radius },
            ScadPrimitive::Cube { size, .. } => kami_sdf::SdfPrimitive::Box {
                half_extents: Vec3::new(size[0] / 2.0, size[1] / 2.0, size[2] / 2.0),
            },
            ScadPrimitive::Cylinder { h, r1, r2, .. } => kami_sdf::SdfPrimitive::Cylinder {
                h: *h,
                r: (*r1 + *r2) / 2.0,
            },
        };
        let t = Mat4::from_scale_rotation_translation(
            Vec3::from(e.scale),
            Quat::from_xyzw(e.rotation[0], e.rotation[1], e.rotation[2], e.rotation[3]),
            Vec3::from(e.position),
        );
        children.push(kami_sdf::SdfNode::Primitive {
            prim,
            transform: t,
            color: e.color,
        });
    }
    kami_sdf::SdfNode::SmoothUnion { children, k: 0.1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_pipeline_scad_to_mesh() {
        let mesh = scad_to_mesh("sphere(r=1.0);", 8, 2.0, 0.5);
        assert!(mesh.vertex_count > 0);
    }

    #[test]
    fn full_pipeline_scad_to_glb() {
        let glb = scad_to_glb("sphere(r=1.0);", 8, 2.0, 0.5);
        assert!(glb.len() > 12);
        assert_eq!(&glb[0..4], &0x46546C67u32.to_le_bytes());
    }

    #[test]
    fn yoro_pipeline() {
        let code = r#"
            union() {
                color([0.34, 0.80, 0.01]) sphere(r=1.5);
                translate([0, 2.8, 0]) color([0.34, 0.80, 0.01]) sphere(r=1.4);
                translate([0, 3.9, 0]) color([0.93, 0.93, 0.95]) cube([1.3, 0.12, 1.3], center=true);
            }
        "#;
        let mesh = scad_to_mesh(code, 16, 4.0, 0.5);
        assert!(mesh.vertex_count > 0);
    }
}
