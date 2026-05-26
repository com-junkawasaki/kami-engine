//! kami-usd — OpenUSD canonical impl + Hydra-style scene delegate.
//!
//! ADR-2605261800 §D10.3: drop-in `omni.usd` API-compat facade backed by a
//! KAMI-native USDA mini-parser. Full tinyusdz binding is deferred to R1.2
//! (Crate + binary formats); this R1.1 cut covers the subset of USDA needed
//! to express robotics + voxel-sandbox scenes for `isekai.etzhayyim.com`:
//!
//!   - `def Xform "name" { double3 xformOp:translate = (x, y, z) ; ... }`
//!   - `def Cube "name"   { double size = 1.0 }`
//!   - `def Sphere "name" { double radius = 0.5 }`
//!   - `def Mesh "name"   { ... }`  — header only (geometry stub)
//!   - `def PhysicsScene "physics" { vector3f physics:gravityDirection = (0,-1,0); float physics:gravityMagnitude = 9.81 }`
//!   - `def Cartpole "cart" { custom string urdf = "@./cart.urdf@" }` — UsdPhysics-style hand-off to `kami-articulated`.
//!
//! The parser is deliberately small (~250 LoC). It exists to prove the
//! omni.usd facade boundary — once tinyusdz lands, callers swap the
//! backing impl without touching the public surface.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const ADR: &str = "ADR-2605261800";
pub const PHASE: &str = "R1.1-usda-mini";
pub const KAMI_NAME: &str = "kami-usd";
pub const NV_COMPAT_TARGET: &str = "omni.usd";
pub const UPSTREAM_REPO: &str = "lighttransport/tinyusdz";

/// Canonical scene description after parsing a `.usda` document.
///
/// Mirrors `pxr::UsdStage` shape: a flat list of prims plus a layer header.
/// Callers (kami-app-isekai, e7m-sim) walk `prims` and dispatch by `kind`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stage {
    pub up_axis: UpAxis,
    pub meters_per_unit: f32,
    pub prims: Vec<Prim>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UpAxis {
    X,
    #[default]
    Y,
    Z,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prim {
    pub path: String,
    pub kind: PrimKind,
    pub xform: Xform,
    pub attrs: Vec<Attr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimKind {
    Xform,
    Cube { size: f32 },
    Sphere { radius: f32 },
    Plane { width: f32, length: f32 },
    Mesh,
    PhysicsScene { gravity: [f32; 3] },
    Cartpole { urdf_ref: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Xform {
    pub translate: [f32; 3],
    pub rotate_xyz_deg: [f32; 3],
    pub scale: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attr {
    pub name: String,
    pub value: AttrValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttrValue {
    Float(f32),
    Float3([f32; 3]),
    String(String),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("expected token `{0}` at line {1}")]
    Expected(&'static str, usize),
    #[error("malformed prim header at line {0}: `{1}`")]
    BadPrimHeader(usize, String),
    #[error("malformed attribute at line {0}: `{1}`")]
    BadAttribute(usize, String),
}

/// Parse a USDA document into a `Stage`.
///
/// Whitespace-tolerant; recognizes a fixed set of `def` types; unknown
/// prim types are accepted as `PrimKind::Xform` so a partial parse
/// degrades gracefully rather than rejecting an entire scene.
pub fn parse_usda(src: &str) -> Result<Stage, ParseError> {
    let mut stage = Stage {
        up_axis: UpAxis::Y,
        meters_per_unit: 1.0,
        prims: Vec::new(),
    };

    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0usize;
    while i < lines.len() {
        let line = lines[i].trim();
        i += 1;

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Layer metadata: `upAxis = "Y"` / `metersPerUnit = 1`
        if let Some(rest) = line.strip_prefix("upAxis") {
            let v = rest.trim().trim_start_matches('=').trim().trim_matches('"');
            stage.up_axis = match v {
                "X" => UpAxis::X,
                "Z" => UpAxis::Z,
                _ => UpAxis::Y,
            };
            continue;
        }
        if let Some(rest) = line.strip_prefix("metersPerUnit") {
            let v = rest.trim().trim_start_matches('=').trim();
            stage.meters_per_unit = v.parse().unwrap_or(1.0);
            continue;
        }

        // `def Xform "name"` / `def Cube "name"` / `def PhysicsScene "name"` / ...
        if let Some(after_def) = line.strip_prefix("def ") {
            let (kind_tok, name_tok) = match after_def.split_once(' ') {
                Some(p) => p,
                None => return Err(ParseError::BadPrimHeader(i, line.to_string())),
            };
            let name = name_tok.trim().trim_matches('"').trim_matches('{').trim();

            // Body lives between `{` ... `}`; collect raw body lines.
            let mut body: Vec<&str> = Vec::new();
            while i < lines.len() {
                let bl = lines[i].trim();
                i += 1;
                if bl == "{" {
                    continue;
                }
                if bl == "}" {
                    break;
                }
                body.push(bl);
            }

            let (xform, attrs, kind) = parse_body(kind_tok.trim(), &body, i)?;
            stage.prims.push(Prim {
                path: format!("/{}", name),
                kind,
                xform,
                attrs,
            });
        }
    }

    Ok(stage)
}

fn parse_body(
    kind_tok: &str,
    body: &[&str],
    line_no: usize,
) -> Result<(Xform, Vec<Attr>, PrimKind), ParseError> {
    let mut xform = Xform {
        scale: [1.0, 1.0, 1.0],
        ..Default::default()
    };
    let mut attrs: Vec<Attr> = Vec::new();
    let mut kind = match kind_tok {
        "Xform" => PrimKind::Xform,
        "Cube" => PrimKind::Cube { size: 1.0 },
        "Sphere" => PrimKind::Sphere { radius: 0.5 },
        "Plane" => PrimKind::Plane {
            width: 10.0,
            length: 10.0,
        },
        "Mesh" => PrimKind::Mesh,
        "PhysicsScene" => PrimKind::PhysicsScene {
            gravity: [0.0, -9.81, 0.0],
        },
        "Cartpole" => PrimKind::Cartpole {
            urdf_ref: String::new(),
        },
        _ => PrimKind::Xform,
    };

    for bl in body {
        if bl.is_empty() || bl.starts_with('#') {
            continue;
        }
        // Pattern: `<typetok> name = value`
        let (lhs, rhs) = match bl.split_once('=') {
            Some(p) => (p.0.trim(), p.1.trim()),
            None => continue,
        };
        // USDA attribute prefix grammar:
        //   [custom|uniform|varying] <typetok> <key> = <value>
        // Skip any number of leading modifiers; consume the type token;
        // the next token is the key.
        let mut lhs_iter = lhs.split_whitespace().peekable();
        while matches!(lhs_iter.peek().copied(), Some("custom") | Some("uniform") | Some("varying")) {
            lhs_iter.next();
        }
        let _typetok = lhs_iter.next();
        let key = match lhs_iter.next() {
            Some(k) => k,
            None => return Err(ParseError::BadAttribute(line_no, bl.to_string())),
        };

        match key {
            "xformOp:translate" => xform.translate = parse_vec3(rhs).unwrap_or(xform.translate),
            "xformOp:rotateXYZ" => {
                xform.rotate_xyz_deg = parse_vec3(rhs).unwrap_or(xform.rotate_xyz_deg)
            }
            "xformOp:scale" => xform.scale = parse_vec3(rhs).unwrap_or(xform.scale),
            "size" => {
                let v = rhs.parse::<f32>().unwrap_or(1.0);
                if matches!(kind, PrimKind::Cube { .. }) {
                    kind = PrimKind::Cube { size: v };
                }
            }
            "radius" => {
                let v = rhs.parse::<f32>().unwrap_or(0.5);
                if matches!(kind, PrimKind::Sphere { .. }) {
                    kind = PrimKind::Sphere { radius: v };
                }
            }
            "width" => {
                let v = rhs.parse::<f32>().unwrap_or(10.0);
                if let PrimKind::Plane { length, .. } = kind {
                    kind = PrimKind::Plane { width: v, length };
                }
            }
            "length" => {
                let v = rhs.parse::<f32>().unwrap_or(10.0);
                if let PrimKind::Plane { width, .. } = kind {
                    kind = PrimKind::Plane { width, length: v };
                }
            }
            "physics:gravityDirection" => {
                if let Some(dir) = parse_vec3(rhs) {
                    if let PrimKind::PhysicsScene { gravity } = kind {
                        let mag = (gravity[0].powi(2) + gravity[1].powi(2) + gravity[2].powi(2))
                            .sqrt()
                            .max(9.81);
                        kind = PrimKind::PhysicsScene {
                            gravity: [dir[0] * mag, dir[1] * mag, dir[2] * mag],
                        };
                    }
                }
            }
            "physics:gravityMagnitude" => {
                let mag = rhs.parse::<f32>().unwrap_or(9.81);
                if let PrimKind::PhysicsScene { gravity } = kind {
                    let norm = (gravity[0].powi(2) + gravity[1].powi(2) + gravity[2].powi(2)).sqrt();
                    let dir = if norm > 1e-6 {
                        [gravity[0] / norm, gravity[1] / norm, gravity[2] / norm]
                    } else {
                        [0.0, -1.0, 0.0]
                    };
                    kind = PrimKind::PhysicsScene {
                        gravity: [dir[0] * mag, dir[1] * mag, dir[2] * mag],
                    };
                }
            }
            "urdf" => {
                let s = rhs.trim_matches('"').trim_matches('@').to_string();
                if matches!(kind, PrimKind::Cartpole { .. }) {
                    kind = PrimKind::Cartpole { urdf_ref: s };
                }
            }
            _ => {
                attrs.push(Attr {
                    name: key.to_string(),
                    value: AttrValue::String(rhs.to_string()),
                });
            }
        }
    }

    Ok((xform, attrs, kind))
}

fn parse_vec3(s: &str) -> Option<[f32; 3]> {
    // `(x, y, z)` — tolerant of whitespace.
    let inner = s.trim().trim_start_matches('(').trim_end_matches(')');
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 3 {
        return None;
    }
    let x = parts[0].trim().parse().ok()?;
    let y = parts[1].trim().parse().ok()?;
    let z = parts[2].trim().parse().ok()?;
    Some([x, y, z])
}

#[cfg(test)]
mod tests {
    use super::*;

    const ISEKAI_OMNIVERSE: &str = r#"#usda 1.0
(
    upAxis = "Y"
    metersPerUnit = 1.0
)

def PhysicsScene "physics"
{
    vector3f physics:gravityDirection = (0, -1, 0)
    float physics:gravityMagnitude = 9.81
}

def Plane "ground"
{
    double3 xformOp:translate = (0, 0, 0)
    double width = 32.0
    double length = 32.0
}

def Cube "block"
{
    double3 xformOp:translate = (-11, 34, 18)
    double size = 1.0
}

def Cartpole "cart"
{
    double3 xformOp:translate = (0, 1.0, 0)
    custom string urdf = "@./cartpole.urdf@"
}
"#;

    #[test]
    fn parses_isekai_omniverse_stage() {
        let st = parse_usda(ISEKAI_OMNIVERSE).expect("parse");
        assert_eq!(st.up_axis, UpAxis::Y);
        assert_eq!(st.meters_per_unit, 1.0);
        assert_eq!(st.prims.len(), 4);

        assert!(matches!(st.prims[0].kind, PrimKind::PhysicsScene { .. }));
        if let PrimKind::PhysicsScene { gravity } = st.prims[0].kind {
            assert!((gravity[1] - (-9.81)).abs() < 1e-3);
        }
        assert!(matches!(st.prims[1].kind, PrimKind::Plane { .. }));
        assert!(matches!(st.prims[2].kind, PrimKind::Cube { .. }));
        if let PrimKind::Cube { size } = st.prims[2].kind {
            assert_eq!(size, 1.0);
        }
        assert_eq!(st.prims[2].xform.translate, [-11.0, 34.0, 18.0]);

        assert!(matches!(st.prims[3].kind, PrimKind::Cartpole { .. }));
        if let PrimKind::Cartpole { ref urdf_ref } = st.prims[3].kind {
            assert!(urdf_ref.contains("cartpole.urdf"));
        }
    }

    #[test]
    fn empty_stage_round_trips() {
        let st = parse_usda("#usda 1.0\n").expect("parse");
        assert!(st.prims.is_empty());
    }

    #[test]
    fn unknown_prim_degrades_to_xform() {
        let st = parse_usda(
            r#"#usda 1.0
def SomeFutureType "x"
{
    double3 xformOp:translate = (1, 2, 3)
}
"#,
        )
        .expect("parse");
        assert_eq!(st.prims.len(), 1);
        assert!(matches!(st.prims[0].kind, PrimKind::Xform));
        assert_eq!(st.prims[0].xform.translate, [1.0, 2.0, 3.0]);
    }
}
