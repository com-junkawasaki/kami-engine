//! SDF (Signed Distance Function) generation layer.
//! Defines primitives + CSG ops as distance functions, sampled into VoxelVolume.

use glam::{Mat4, Quat, Vec3};

/// SDF primitive — each returns signed distance from a point.
#[derive(Debug, Clone)]
pub enum SdfPrimitive {
    Sphere { radius: f32 },
    Box { half_extents: Vec3 },
    Cylinder { h: f32, r: f32 },
    Capsule { h: f32, r: f32 },
    Torus { major_r: f32, minor_r: f32 },
}

/// SDF CSG tree node.
#[derive(Debug, Clone)]
pub enum SdfNode {
    Primitive {
        prim: SdfPrimitive,
        transform: Mat4,
        color: [f32; 4],
    },
    Union(Vec<SdfNode>),
    Difference {
        base: Box<SdfNode>,
        subtract: Box<SdfNode>,
    },
    Intersection {
        a: Box<SdfNode>,
        b: Box<SdfNode>,
    },
    SmoothUnion {
        children: Vec<SdfNode>,
        k: f32,
    },
    /// NeRF-style density field: sampled from external data.
    DensityField {
        data: Vec<f32>,
        dims: [u32; 3],
        threshold: f32,
        color: [f32; 4],
    },
}

/// SDF evaluation result at a point.
#[derive(Debug, Clone, Copy)]
pub struct SdfSample {
    pub distance: f32,
    pub color: [f32; 4],
}

impl SdfPrimitive {
    pub fn distance(&self, p: Vec3) -> f32 {
        match self {
            SdfPrimitive::Sphere { radius } => p.length() - radius,
            SdfPrimitive::Box { half_extents } => {
                let q = p.abs() - *half_extents;
                q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0)
            }
            SdfPrimitive::Cylinder { h, r } => {
                let d = Vec3::new(
                    Vec3::new(p.x, 0.0, p.z).length() - r,
                    p.y.abs() - h / 2.0,
                    0.0,
                );
                d.x.max(d.y).min(0.0) + Vec3::new(d.x.max(0.0), d.y.max(0.0), 0.0).length()
            }
            SdfPrimitive::Capsule { h, r } => {
                let py = p.y.clamp(-h / 2.0, h / 2.0);
                (p - Vec3::new(0.0, py, 0.0)).length() - r
            }
            SdfPrimitive::Torus { major_r, minor_r } => {
                let q = Vec3::new(Vec3::new(p.x, 0.0, p.z).length() - major_r, p.y, 0.0);
                q.length() - minor_r
            }
        }
    }
}

impl SdfNode {
    pub fn sample(&self, p: Vec3) -> SdfSample {
        match self {
            SdfNode::Primitive {
                prim,
                transform,
                color,
            } => {
                let inv = transform.inverse();
                let local_p = inv.transform_point3(p);
                SdfSample {
                    distance: prim.distance(local_p),
                    color: *color,
                }
            }
            SdfNode::Union(children) => {
                let mut best = SdfSample {
                    distance: f32::MAX,
                    color: [0.5; 4],
                };
                for child in children {
                    let s = child.sample(p);
                    if s.distance < best.distance {
                        best = s;
                    }
                }
                best
            }
            SdfNode::Difference { base, subtract } => {
                let a = base.sample(p);
                let b = subtract.sample(p);
                SdfSample {
                    distance: a.distance.max(-b.distance),
                    color: a.color,
                }
            }
            SdfNode::Intersection { a, b } => {
                let sa = a.sample(p);
                let sb = b.sample(p);
                if sa.distance > sb.distance { sa } else { sb }
            }
            SdfNode::SmoothUnion { children, k } => {
                let mut best = SdfSample {
                    distance: f32::MAX,
                    color: [0.5; 4],
                };
                for child in children {
                    let s = child.sample(p);
                    if s.distance < best.distance {
                        // Smooth min
                        let h = (0.5 + 0.5 * (best.distance - s.distance) / k).clamp(0.0, 1.0);
                        best.distance =
                            best.distance * (1.0 - h) + s.distance * h - k * h * (1.0 - h);
                        best.color = s.color;
                    }
                }
                best
            }
            SdfNode::DensityField {
                data,
                dims,
                threshold,
                color,
            } => {
                // Trilinear sample from 3D grid
                let [dx, dy, dz] = *dims;
                let gx = ((p.x + 1.0) * 0.5 * (dx - 1) as f32).clamp(0.0, (dx - 1) as f32);
                let gy = ((p.y + 1.0) * 0.5 * (dy - 1) as f32).clamp(0.0, (dy - 1) as f32);
                let gz = ((p.z + 1.0) * 0.5 * (dz - 1) as f32).clamp(0.0, (dz - 1) as f32);
                let ix = gx as u32;
                let iy = gy as u32;
                let iz = gz as u32;
                let idx = |x: u32, y: u32, z: u32| -> f32 {
                    let x = x.min(dx - 1);
                    let y = y.min(dy - 1);
                    let z = z.min(dz - 1);
                    data[(z * dy * dx + y * dx + x) as usize]
                };
                let fx = gx.fract();
                let fy = gy.fract();
                let fz = gz.fract();
                let c000 = idx(ix, iy, iz);
                let c100 = idx(ix + 1, iy, iz);
                let c010 = idx(ix, iy + 1, iz);
                let c110 = idx(ix + 1, iy + 1, iz);
                let c001 = idx(ix, iy, iz + 1);
                let c101 = idx(ix + 1, iy, iz + 1);
                let c011 = idx(ix, iy + 1, iz + 1);
                let c111 = idx(ix + 1, iy + 1, iz + 1);
                let density = c000 * (1.0 - fx) * (1.0 - fy) * (1.0 - fz)
                    + c100 * fx * (1.0 - fy) * (1.0 - fz)
                    + c010 * (1.0 - fx) * fy * (1.0 - fz)
                    + c110 * fx * fy * (1.0 - fz)
                    + c001 * (1.0 - fx) * (1.0 - fy) * fz
                    + c101 * fx * (1.0 - fy) * fz
                    + c011 * (1.0 - fx) * fy * fz
                    + c111 * fx * fy * fz;
                SdfSample {
                    distance: *threshold - density,
                    color: *color,
                }
            }
        }
    }
}

/// Sample SDF into VoxelVolume.
pub fn sample_sdf(node: &SdfNode, resolution: u32, bounds: f32) -> kami_voxel::VoxelVolume {
    let mut volume = kami_voxel::VoxelVolume::new_dense(resolution, resolution, resolution);
    let step = bounds * 2.0 / resolution as f32;
    for z in 0..resolution {
        for y in 0..resolution {
            for x in 0..resolution {
                let p = Vec3::new(
                    -bounds + (x as f32 + 0.5) * step,
                    -bounds + (y as f32 + 0.5) * step,
                    -bounds + (z as f32 + 0.5) * step,
                );
                let s = node.sample(p);
                if s.distance <= 0.0 {
                    volume.set(
                        x,
                        y,
                        z,
                        kami_voxel::Voxel {
                            material: 1,
                            color: s.color,
                        },
                    );
                }
            }
        }
    }
    volume
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_sdf() {
        let s = SdfPrimitive::Sphere { radius: 1.0 };
        assert!((s.distance(Vec3::ZERO) - (-1.0)).abs() < 0.001);
        assert!((s.distance(Vec3::new(1.0, 0.0, 0.0))).abs() < 0.001);
        assert!((s.distance(Vec3::new(2.0, 0.0, 0.0)) - 1.0).abs() < 0.001);
    }

    #[test]
    fn box_sdf() {
        let b = SdfPrimitive::Box {
            half_extents: Vec3::ONE,
        };
        assert!(b.distance(Vec3::ZERO) < 0.0);
        assert!((b.distance(Vec3::new(1.0, 0.0, 0.0))).abs() < 0.001);
    }

    #[test]
    fn union_sdf() {
        let u = SdfNode::Union(vec![
            SdfNode::Primitive {
                prim: SdfPrimitive::Sphere { radius: 1.0 },
                transform: Mat4::IDENTITY,
                color: [1.0; 4],
            },
            SdfNode::Primitive {
                prim: SdfPrimitive::Sphere { radius: 1.0 },
                transform: Mat4::from_translation(Vec3::new(3.0, 0.0, 0.0)),
                color: [0.0, 1.0, 0.0, 1.0],
            },
        ]);
        assert!(u.sample(Vec3::ZERO).distance < 0.0);
        assert!(u.sample(Vec3::new(3.0, 0.0, 0.0)).distance < 0.0);
        assert!(u.sample(Vec3::new(1.5, 0.0, 0.0)).distance > 0.0);
    }

    #[test]
    fn sample_sdf_to_volume() {
        let node = SdfNode::Primitive {
            prim: SdfPrimitive::Sphere { radius: 0.5 },
            transform: Mat4::IDENTITY,
            color: [1.0, 0.0, 0.0, 1.0],
        };
        let vol = sample_sdf(&node, 16, 1.0);
        let filled = vol.count_filled();
        assert!(filled > 0);
        assert!(filled < 16 * 16 * 16);
    }

    #[test]
    fn difference_sdf() {
        let d = SdfNode::Difference {
            base: Box::new(SdfNode::Primitive {
                prim: SdfPrimitive::Sphere { radius: 1.0 },
                transform: Mat4::IDENTITY,
                color: [1.0; 4],
            }),
            subtract: Box::new(SdfNode::Primitive {
                prim: SdfPrimitive::Sphere { radius: 0.5 },
                transform: Mat4::IDENTITY,
                color: [1.0; 4],
            }),
        };
        assert!(d.sample(Vec3::ZERO).distance > 0.0);
        assert!(d.sample(Vec3::new(0.8, 0.0, 0.0)).distance < 0.0);
    }

    #[test]
    fn jsonld_sphere() {
        let node =
            parse_sdf_jsonld(r##"{"@type":"Sphere","r":1.5,"pos":[0,1.2,0],"color":"#58CC02"}"##)
                .unwrap();
        let s = node.sample(Vec3::new(0.0, 1.2, 0.0));
        assert!(s.distance < 0.0);
    }

    #[test]
    fn jsonld_smooth_union() {
        let json = r##"{
            "@type":"SmoothUnion","k":0.3,
            "children":[
                {"@type":"Sphere","r":1.5,"pos":[0,1.2,0],"color":"#58CC02"},
                {"@type":"Sphere","r":1.4,"pos":[0,2.8,0],"color":"#58CC02"}
            ]
        }"##;
        let node = parse_sdf_jsonld(json).unwrap();
        assert!(node.sample(Vec3::new(0.0, 2.0, 0.0)).distance < 0.0);
    }

    #[test]
    fn jsonld_with_ref() {
        let json = r##"{
            "@type":"Union",
            "defs":{"eye":{"@type":"Sphere","r":0.5,"scale":[1,1,0.5],"color":"white"}},
            "children":[
                {"$ref":"eye","pos":[-0.6,2.9,1.1]},
                {"$ref":"eye","pos":[0.6,2.9,1.1]}
            ]
        }"##;
        let node = parse_sdf_jsonld(json).unwrap();
        assert!(node.sample(Vec3::new(-0.6, 2.9, 1.1)).distance < 0.0);
        assert!(node.sample(Vec3::new(0.6, 2.9, 1.1)).distance < 0.0);
    }
}

// ── SDF JSON-LD Parser ──────────────────────────────────────────────────────

/// Parse SDF JSON-LD into SdfNode tree.
/// Supports: Sphere, Box, Cylinder, Capsule, Torus, Union, SmoothUnion, Difference, Intersection.
/// Features: named colors (#hex / "white"), pos/rot/scale shorthand, $ref + defs.
pub fn parse_sdf_jsonld(json: &str) -> Result<SdfNode, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let defs = v
        .get("defs")
        .and_then(|d| d.as_object())
        .cloned()
        .unwrap_or_default();
    parse_sdf_value(&v, &defs)
}

fn parse_sdf_value(
    v: &serde_json::Value,
    defs: &serde_json::Map<String, serde_json::Value>,
) -> Result<SdfNode, String> {
    // Handle $ref
    if let Some(ref_name) = v.get("$ref").and_then(|r| r.as_str()) {
        let def = defs
            .get(ref_name)
            .ok_or_else(|| format!("undefined $ref: {}", ref_name))?;
        // Merge ref definition with overrides (pos, rot, scale)
        let mut merged = def.clone();
        if let (Some(obj), Some(over)) = (merged.as_object_mut(), v.as_object()) {
            for (k, val) in over {
                if k != "$ref" {
                    obj.insert(k.clone(), val.clone());
                }
            }
        }
        return parse_sdf_value(&merged, defs);
    }

    let ty = v.get("@type").and_then(|t| t.as_str()).unwrap_or("");
    let color = parse_color_field(v);
    let transform = parse_transform(v);

    // Merge local defs with parent defs
    let mut all_defs = defs.clone();
    if let Some(local_defs) = v.get("defs").and_then(|d| d.as_object()) {
        for (k, val) in local_defs {
            all_defs.insert(k.clone(), val.clone());
        }
    }

    match ty {
        "Sphere" => {
            let r = v.get("r").and_then(|r| r.as_f64()).unwrap_or(0.5) as f32;
            Ok(SdfNode::Primitive {
                prim: SdfPrimitive::Sphere { radius: r },
                transform,
                color,
            })
        }
        "Box" => {
            let size = parse_vec3(v, "size").unwrap_or(Vec3::ONE);
            Ok(SdfNode::Primitive {
                prim: SdfPrimitive::Box {
                    half_extents: size * 0.5,
                },
                transform,
                color,
            })
        }
        "Cylinder" => {
            let h = v.get("h").and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
            let r = v.get("r").and_then(|x| x.as_f64()).unwrap_or(0.5) as f32;
            Ok(SdfNode::Primitive {
                prim: SdfPrimitive::Cylinder { h, r },
                transform,
                color,
            })
        }
        "Capsule" => {
            let h = v.get("h").and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
            let r = v.get("r").and_then(|x| x.as_f64()).unwrap_or(0.25) as f32;
            Ok(SdfNode::Primitive {
                prim: SdfPrimitive::Capsule { h, r },
                transform,
                color,
            })
        }
        "Torus" => {
            let major = v.get("R").and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
            let minor = v.get("r").and_then(|x| x.as_f64()).unwrap_or(0.25) as f32;
            Ok(SdfNode::Primitive {
                prim: SdfPrimitive::Torus {
                    major_r: major,
                    minor_r: minor,
                },
                transform,
                color,
            })
        }
        "Union" => {
            let children = parse_children(v, &all_defs)?;
            Ok(SdfNode::Union(children))
        }
        "SmoothUnion" => {
            let k = v.get("k").and_then(|x| x.as_f64()).unwrap_or(0.1) as f32;
            let children = parse_children(v, &all_defs)?;
            Ok(SdfNode::SmoothUnion { children, k })
        }
        "Difference" => {
            let children = parse_children(v, &all_defs)?;
            if children.len() < 2 {
                return Err("Difference needs ≥2 children".into());
            }
            let mut iter = children.into_iter();
            let base = iter.next().unwrap();
            let subtract = iter.next().unwrap();
            Ok(SdfNode::Difference {
                base: Box::new(base),
                subtract: Box::new(subtract),
            })
        }
        "Intersection" => {
            let children = parse_children(v, &all_defs)?;
            if children.len() < 2 {
                return Err("Intersection needs ≥2 children".into());
            }
            let mut iter = children.into_iter();
            Ok(SdfNode::Intersection {
                a: Box::new(iter.next().unwrap()),
                b: Box::new(iter.next().unwrap()),
            })
        }
        _ => Err(format!("unknown SDF type: {}", ty)),
    }
}

fn parse_children(
    v: &serde_json::Value,
    defs: &serde_json::Map<String, serde_json::Value>,
) -> Result<Vec<SdfNode>, String> {
    match v.get("children").and_then(|c| c.as_array()) {
        Some(arr) => {
            let nodes: Vec<SdfNode> = arr
                .iter()
                .filter_map(|child| parse_sdf_value(child, defs).ok())
                .collect();
            Ok(nodes)
        }
        None => Ok(vec![]),
    }
}

fn parse_vec3(v: &serde_json::Value, key: &str) -> Option<Vec3> {
    v.get(key).and_then(|a| a.as_array()).map(|arr| {
        Vec3::new(
            arr.first().and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
            arr.get(1).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
            arr.get(2).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32,
        )
    })
}

fn parse_transform(v: &serde_json::Value) -> Mat4 {
    let pos = parse_vec3(v, "pos").unwrap_or(Vec3::ZERO);
    let scale = parse_vec3(v, "scale").unwrap_or(Vec3::ONE);
    let rot = parse_vec3(v, "rot").unwrap_or(Vec3::ZERO);
    let q = Quat::from_euler(
        glam::EulerRot::XYZ,
        rot.x.to_radians(),
        rot.y.to_radians(),
        rot.z.to_radians(),
    );
    Mat4::from_scale_rotation_translation(scale, q, pos)
}

fn parse_color_field(v: &serde_json::Value) -> [f32; 4] {
    match v.get("color") {
        Some(serde_json::Value::String(s)) => parse_color_str(s),
        Some(serde_json::Value::Array(arr)) => {
            let r = arr.first().and_then(|x| x.as_f64()).unwrap_or(0.5) as f32;
            let g = arr.get(1).and_then(|x| x.as_f64()).unwrap_or(0.5) as f32;
            let b = arr.get(2).and_then(|x| x.as_f64()).unwrap_or(0.5) as f32;
            let a = arr.get(3).and_then(|x| x.as_f64()).unwrap_or(1.0) as f32;
            [r, g, b, a]
        }
        _ => [0.5, 0.5, 0.5, 1.0],
    }
}

fn parse_color_str(s: &str) -> [f32; 4] {
    if s.starts_with('#') && s.len() >= 7 {
        let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(128) as f32 / 255.0;
        let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(128) as f32 / 255.0;
        let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(128) as f32 / 255.0;
        return [r, g, b, 1.0];
    }
    match s.to_lowercase().as_str() {
        "white" => [1.0, 1.0, 1.0, 1.0],
        "black" => [0.0, 0.0, 0.0, 1.0],
        "red" => [1.0, 0.0, 0.0, 1.0],
        "green" => [0.0, 1.0, 0.0, 1.0],
        "blue" => [0.0, 0.0, 1.0, 1.0],
        _ => [0.5, 0.5, 0.5, 1.0],
    }
}
