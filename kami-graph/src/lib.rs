//! kami-graph: Force-directed graph layout + mesh generation for wgpu rendering.
//!
//! Ingests haisen/systemofsystem JSON from `gftd` CLI and produces:
//! - Node positions via force-directed simulation
//! - Instanced sphere meshes (nodes) with per-instance color + transform
//! - Line meshes (edges) updated each frame
//!
//! Rendering is done by kami-render's existing PBR pipeline (orthographic camera).

pub mod data;
pub mod layout;

pub use data::{GraphData, GraphEdge, GraphNode, HaisenData, SoSData};
pub use layout::{BusLine, ForceLayout, PcbLayout};

use glam::{Mat4, Vec3};

/// Edge type color palette (RGBA f32).
/// Edge type color — Nintendo-inspired vivid pastels.
pub fn edge_color(edge_type: &str) -> [f32; 4] {
    match edge_type {
        "invoke" => [0.98, 0.34, 0.40, 1.0],   // Splatoon red
        "writes" => [0.20, 0.75, 0.95, 1.0],   // Switch blue
        "reads" => [0.40, 0.90, 0.45, 1.0],    // Nintendo green
        "subscribe" => [1.0, 0.75, 0.20, 1.0], // Mario gold
        "follow" => [0.70, 0.40, 0.95, 1.0],   // Waluigi purple
        "workers_rpc" => [0.20, 0.75, 0.95, 1.0],
        "xrpc" => [0.40, 0.90, 0.45, 1.0],
        "subscribe_repos" => [1.0, 0.75, 0.20, 1.0],
        "http" => [0.65, 0.70, 0.75, 1.0], // Light gray
        _ => [0.60, 0.65, 0.70, 1.0],
    }
}

/// Group color palette — Nintendo-inspired bright pastels (Splatoon/Animal Crossing).
const PALETTE: [[f32; 4]; 20] = [
    [0.98, 0.34, 0.40, 1.0], // Splatoon pink-red
    [0.20, 0.75, 0.95, 1.0], // Switch blue
    [0.40, 0.90, 0.45, 1.0], // Nintendo green
    [1.0, 0.75, 0.20, 1.0],  // Mario gold
    [0.70, 0.40, 0.95, 1.0], // Waluigi purple
    [0.15, 0.85, 0.70, 1.0], // AC mint
    [1.0, 0.55, 0.30, 1.0],  // Inkling orange
    [0.95, 0.45, 0.70, 1.0], // Kirby pink
    [0.55, 0.55, 1.0, 1.0],  // Zelda blue
    [0.95, 0.85, 0.25, 1.0], // Star yellow
    [0.30, 0.90, 0.80, 1.0], // Teal
    [0.85, 0.55, 0.95, 1.0], // Lavender
    [0.50, 0.95, 0.40, 1.0], // Yoshi green
    [1.0, 0.65, 0.50, 1.0],  // Peach
    [0.45, 0.80, 1.0, 1.0],  // Sky
    [0.90, 0.40, 0.40, 1.0], // Red
    [0.40, 0.70, 0.95, 1.0], // Light blue
    [0.80, 0.95, 0.40, 1.0], // Lime
    [0.95, 0.60, 0.80, 1.0], // Rose
    [0.70, 0.90, 0.60, 1.0], // Sage
];

pub fn group_color(group_index: usize) -> [f32; 4] {
    PALETTE[group_index % PALETTE.len()]
}

/// Build instance transforms for graph nodes (translate to position, scale by radius).
pub fn build_node_instances(layout: &ForceLayout) -> Vec<[f32; 16]> {
    layout
        .nodes
        .iter()
        .map(|n| {
            let s = n.radius * 0.1; // Scale factor for sphere (base sphere r=1 → visible ~0.5 units)
            let t = Mat4::from_scale_rotation_translation(
                Vec3::new(s, s, s),
                glam::Quat::IDENTITY,
                Vec3::new(n.x, n.y, 0.0),
            );
            t.to_cols_array()
        })
        .collect()
}

/// Build instance material colors for graph nodes (albedo per instance).
pub fn build_node_colors(layout: &ForceLayout) -> Vec<[f32; 4]> {
    layout
        .nodes
        .iter()
        .map(|n| group_color(n.group_index))
        .collect()
}

/// Build line vertex data for edges: pairs of (pos, color) per endpoint.
/// Returns interleaved [x, y, z, r, g, b, a] × 2 per edge.
pub fn build_edge_lines(layout: &ForceLayout) -> Vec<f32> {
    let mut verts = Vec::with_capacity(layout.edges.len() * 14);
    for edge in &layout.edges {
        let from = &layout.nodes[edge.from_idx];
        let to = &layout.nodes[edge.to_idx];
        let color = edge_color(&edge.edge_type);
        // From vertex
        verts.extend_from_slice(&[from.x, from.y, 0.0, color[0], color[1], color[2], color[3]]);
        // To vertex
        verts.extend_from_slice(&[to.x, to.y, 0.0, color[0], color[1], color[2], color[3]]);
    }
    verts
}

/// Build instance transforms for PCB layout nodes.
pub fn build_node_instances_pcb(layout: &PcbLayout) -> Vec<[f32; 16]> {
    layout
        .nodes
        .iter()
        .map(|n| {
            let s = n.radius * 0.1;
            let t = Mat4::from_scale_rotation_translation(
                Vec3::new(s, s, s),
                glam::Quat::IDENTITY,
                Vec3::new(n.x, 0.0, n.y),
            );
            t.to_cols_array()
        })
        .collect()
}

/// Build node colors for PCB layout.
pub fn build_node_colors_pcb(layout: &PcbLayout) -> Vec<[f32; 4]> {
    layout
        .nodes
        .iter()
        .map(|n| group_color(n.group_index))
        .collect()
}

/// Camera extent for orthographic projection covering a force-directed graph.
pub fn graph_camera_extent(layout: &ForceLayout, padding: f32) -> (f32, f32, Vec3) {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for n in &layout.nodes {
        if n.x < min_x {
            min_x = n.x;
        }
        if n.y < min_y {
            min_y = n.y;
        }
        if n.x > max_x {
            max_x = n.x;
        }
        if n.y > max_y {
            max_y = n.y;
        }
    }
    let cx = (min_x + max_x) * 0.5;
    let cy = (min_y + max_y) * 0.5;
    let w = (max_x - min_x) + padding;
    let h = (max_y - min_y) + padding;
    (w, h, Vec3::new(cx, cy, 0.0))
}

/// Camera extent for PCB layout (app nodes + bus lines).
pub fn graph_camera_extent_pcb(layout: &PcbLayout, padding: f32) -> (f32, f32, Vec3) {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    // App nodes
    for n in &layout.nodes {
        if n.radius <= 0.0 {
            continue;
        }
        if n.x < min_x {
            min_x = n.x;
        }
        if n.y < min_y {
            min_y = n.y;
        }
        if n.x > max_x {
            max_x = n.x;
        }
        if n.y > max_y {
            max_y = n.y;
        }
    }
    // Bus lines (include in Y range)
    for bus in &layout.buses {
        if bus.y < min_y {
            min_y = bus.y;
        }
    }
    let cx = (min_x + max_x) * 0.5;
    let cy = (min_y + max_y) * 0.5;
    let w = (max_x - min_x) + padding;
    let h = (max_y - min_y) + padding;
    (w, h, Vec3::new(cx, cy, 0.0))
}
