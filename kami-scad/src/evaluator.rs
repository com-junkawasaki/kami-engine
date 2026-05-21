//! CSG tree evaluator: flatten ScadNode AST into Vec<ScadEntity>.
//! Each ScadEntity = one renderable primitive with accumulated transform + color.

use crate::parser::ScadNode;
use glam::{Mat4, Quat, Vec3};
use std::collections::HashMap;

/// A single renderable entity produced by evaluating an OpenSCAD AST.
#[derive(Debug, Clone)]
pub struct ScadEntity {
    pub id: String,
    pub position: [f32; 3],
    pub rotation: [f32; 4], // quaternion xyzw
    pub scale: [f32; 3],
    pub primitive: ScadPrimitive,
    pub color: [f32; 4],
}

#[derive(Debug, Clone)]
pub enum ScadPrimitive {
    Sphere {
        radius: f32,
    },
    Cube {
        size: [f32; 3],
        center: bool,
    },
    Cylinder {
        h: f32,
        r1: f32,
        r2: f32,
        center: bool,
    },
}

struct EvalCtx {
    transform: Mat4,
    color: [f32; 4],
    entities: Vec<ScadEntity>,
    counter: u32,
    modules: HashMap<String, Vec<ScadNode>>,
}

impl EvalCtx {
    fn new() -> Self {
        Self {
            transform: Mat4::IDENTITY,
            color: [0.5, 0.5, 0.5, 1.0],
            entities: Vec::new(),
            counter: 0,
            modules: HashMap::new(),
        }
    }

    fn next_id(&mut self) -> String {
        self.counter += 1;
        format!("scad-{}", self.counter)
    }

    fn decompose(&self) -> ([f32; 3], [f32; 4], [f32; 3]) {
        let (scale, rotation, translation) = self.transform.to_scale_rotation_translation();
        (
            translation.into(),
            [rotation.x, rotation.y, rotation.z, rotation.w],
            scale.into(),
        )
    }

    fn emit(&mut self, primitive: ScadPrimitive) {
        let (pos, rot, scl) = {
            let (p, r, s) = self.decompose();
            (p, r, s)
        };
        let id = self.next_id();
        let color = self.color;
        self.entities.push(ScadEntity {
            id,
            position: pos,
            rotation: rot,
            scale: scl,
            primitive,
            color,
        });
    }

    fn eval_nodes(&mut self, nodes: &[ScadNode]) {
        for node in nodes {
            self.eval_node(node);
        }
    }

    fn eval_node(&mut self, node: &ScadNode) {
        match node {
            ScadNode::Sphere { r } => {
                self.emit(ScadPrimitive::Sphere { radius: *r });
            }
            ScadNode::Cube { size, center } => {
                if !center {
                    // OpenSCAD default: corner at origin. Shift by half size.
                    let saved = self.transform;
                    self.transform = self.transform
                        * Mat4::from_translation(Vec3::new(
                            size[0] / 2.0,
                            size[1] / 2.0,
                            size[2] / 2.0,
                        ));
                    self.emit(ScadPrimitive::Cube {
                        size: *size,
                        center: true,
                    });
                    self.transform = saved;
                } else {
                    self.emit(ScadPrimitive::Cube {
                        size: *size,
                        center: true,
                    });
                }
            }
            ScadNode::Cylinder { h, r1, r2, center } => {
                if !center {
                    let saved = self.transform;
                    self.transform =
                        self.transform * Mat4::from_translation(Vec3::new(0.0, *h / 2.0, 0.0));
                    self.emit(ScadPrimitive::Cylinder {
                        h: *h,
                        r1: *r1,
                        r2: *r2,
                        center: true,
                    });
                    self.transform = saved;
                } else {
                    self.emit(ScadPrimitive::Cylinder {
                        h: *h,
                        r1: *r1,
                        r2: *r2,
                        center: true,
                    });
                }
            }
            ScadNode::Translate { v, children } => {
                let saved = self.transform;
                self.transform = self.transform * Mat4::from_translation(Vec3::from(*v));
                self.eval_nodes(children);
                self.transform = saved;
            }
            ScadNode::Rotate { v, children } => {
                let saved = self.transform;
                let rx = Quat::from_rotation_x(v[0].to_radians());
                let ry = Quat::from_rotation_y(v[1].to_radians());
                let rz = Quat::from_rotation_z(v[2].to_radians());
                self.transform = self.transform * Mat4::from_quat(rz * ry * rx);
                self.eval_nodes(children);
                self.transform = saved;
            }
            ScadNode::Scale { v, children } => {
                let saved = self.transform;
                self.transform = self.transform * Mat4::from_scale(Vec3::from(*v));
                self.eval_nodes(children);
                self.transform = saved;
            }
            ScadNode::Color { rgba, children } => {
                let saved_color = self.color;
                self.color = *rgba;
                self.eval_nodes(children);
                self.color = saved_color;
            }
            ScadNode::Union { children } | ScadNode::Block { children } => {
                self.eval_nodes(children);
            }
            ScadNode::Difference { children } | ScadNode::Intersection { children } => {
                // Phase 1: treat as union (visual approximation).
                // Phase 2: implement BSP-tree CSG boolean mesh operations.
                self.eval_nodes(children);
            }
            ScadNode::ModuleDef { name, body } => {
                self.modules.insert(name.clone(), body.clone());
            }
            ScadNode::ModuleCall { name } => {
                if let Some(body) = self.modules.get(name).cloned() {
                    self.eval_nodes(&body);
                }
            }
        }
    }
}

/// Evaluate OpenSCAD source code into a list of renderable entities.
pub fn evaluate(src: &str) -> Vec<ScadEntity> {
    let nodes = crate::parser::parse(src);
    let mut ctx = EvalCtx::new();
    ctx.eval_nodes(&nodes);
    ctx.entities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_simple_sphere() {
        let entities = evaluate("sphere(r=2.0);");
        assert_eq!(entities.len(), 1);
        match &entities[0].primitive {
            ScadPrimitive::Sphere { radius } => assert!((*radius - 2.0).abs() < 0.001),
            _ => panic!("expected Sphere"),
        }
    }

    #[test]
    fn eval_translated_cube() {
        let entities = evaluate("translate([5, 0, 0]) cube([2, 2, 2], center=true);");
        assert_eq!(entities.len(), 1);
        assert!((entities[0].position[0] - 5.0).abs() < 0.001);
    }

    #[test]
    fn eval_colored_union() {
        let entities = evaluate(
            r#"
            union() {
                color([1, 0, 0]) sphere(r=1);
                color([0, 1, 0]) translate([3, 0, 0]) sphere(r=0.5);
            }
        "#,
        );
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(entities[1].color, [0.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn eval_module() {
        let entities = evaluate(
            r#"
            module arm() { sphere(r=0.5); }
            translate([-2, 0, 0]) arm();
            translate([2, 0, 0]) arm();
        "#,
        );
        assert_eq!(entities.len(), 2);
        assert!((entities[0].position[0] - (-2.0)).abs() < 0.001);
        assert!((entities[1].position[0] - 2.0).abs() < 0.001);
    }

    #[test]
    fn eval_yoro_model() {
        let entities = evaluate(
            r#"
            union() {
                color([0.34, 0.80, 0.01]) sphere(r=1.5);
                translate([0, 2.8, 0]) color([0.34, 0.80, 0.01]) sphere(r=1.4);
                translate([-0.55, 2.85, 1.1]) color([1,1,1]) scale([1,1,0.5]) sphere(r=0.45);
                translate([0.55, 2.85, 1.1]) color([1,1,1]) scale([1,1,0.5]) sphere(r=0.45);
                translate([0, 3.9, 0]) color([0.93,0.93,0.95]) cube([1.3, 0.12, 1.3], center=true);
            }
        "#,
        );
        assert_eq!(entities.len(), 5);
    }
}
