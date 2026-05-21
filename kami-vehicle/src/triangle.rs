//! Triangle — aero / collision surface formed by three nodes.
//!
//! BeamNG uses triangles for two purposes:
//!   1. **Aerodynamics**: each triangle generates drag and lift forces
//!      proportional to the signed area projected against the air-relative
//!      velocity. Body panels => drag, wings => lift.
//!   2. **Collision hull**: the convex / non-convex collection of triangles
//!      is the surface tested against ground and other vehicles.
//!
//! For now we only resolve aero forces here; ground contact is handled
//! per-node in `ground.rs` (cheaper and matches BeamNG's hybrid approach).

use crate::node::NodeId;
use serde::{Deserialize, Serialize};

pub type TriangleId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriangleGroup {
    /// Body panel (high drag, low lift).
    Body,
    /// Wing / spoiler (drag + meaningful lift).
    Wing,
    /// Underbody (drag only).
    Underbody,
    /// Window (very low drag — assumed flush).
    Window,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Triangle {
    pub id: TriangleId,
    pub n1: NodeId,
    pub n2: NodeId,
    pub n3: NodeId,
    pub drag_coef: f32,
    pub lift_coef: f32,
    pub group: TriangleGroup,
}

impl Triangle {
    pub fn new(id: TriangleId, n1: NodeId, n2: NodeId, n3: NodeId) -> Self {
        Self {
            id,
            n1,
            n2,
            n3,
            drag_coef: 0.30,
            lift_coef: 0.0,
            group: TriangleGroup::Body,
        }
    }

    pub fn with_aero(mut self, drag: f32, lift: f32) -> Self {
        self.drag_coef = drag;
        self.lift_coef = lift;
        self
    }

    pub fn with_group(mut self, g: TriangleGroup) -> Self {
        self.group = g;
        self
    }
}
