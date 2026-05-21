//! Data model: deserialize haisen/systemofsystem JSON from `gftd` CLI.

use serde::Deserialize;
use std::collections::HashMap;

/// Unified graph data after transform.
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub groups: Vec<String>,
}

pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub group: String,
    pub group_index: usize,
    pub node_type: String,
    pub radius: f32,
    /// Pre-computed layout position (0,0 = not set).
    pub x: f32,
    pub y: f32,
}

pub struct GraphEdge {
    pub from_idx: usize,
    pub to_idx: usize,
    pub edge_type: String,
    pub label: String,
}

// --- Haisen JSON ---

#[derive(Deserialize)]
pub struct HaisenData {
    pub apps: Vec<HaisenApp>,
    #[serde(default)]
    pub edges: Vec<HaisenEdge>,
    #[serde(default)]
    pub infra: Vec<HaisenApp>,
    #[serde(default)]
    pub stats: HaisenStats,
}

#[derive(Deserialize)]
pub struct HaisenApp {
    #[serde(default)]
    pub nanoid: String,
    #[serde(default)]
    pub did: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub performer_type: String,
    #[serde(default)]
    pub runtime_type: String,
    #[serde(default)]
    pub project: String,
    /// Pre-computed layout X coordinate (from `gftd haisen scan --layout`)
    #[serde(default)]
    pub x: f32,
    /// Pre-computed layout Y coordinate
    #[serde(default)]
    pub y: f32,
}

#[derive(Deserialize)]
pub struct HaisenEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
    #[serde(default)]
    pub label: String,
}

#[derive(Deserialize, Default)]
pub struct HaisenStats {
    #[serde(default)]
    pub total_apps: u32,
    #[serde(default)]
    pub total_edges: u32,
    #[serde(default)]
    pub orphans: u32,
}

// --- SoS JSON ---

#[derive(Deserialize)]
pub struct SoSData {
    pub systems: Vec<SoSSystem>,
    #[serde(default)]
    pub interfaces: Vec<SoSInterface>,
    #[serde(default)]
    pub layers: Vec<SoSLayer>,
    #[serde(default)]
    pub stats: SoSStats,
}

#[derive(Deserialize)]
pub struct SoSSystem {
    pub id: String,
    #[serde(default)]
    pub system_type: String,
    #[serde(default)]
    pub app_count: u32,
    #[serde(default)]
    pub deployed: u32,
}

#[derive(Deserialize)]
pub struct SoSInterface {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub edge_count: u32,
}

#[derive(Deserialize)]
pub struct SoSLayer {
    pub name: String,
    pub systems: Vec<String>,
}

#[derive(Deserialize, Default)]
pub struct SoSStats {
    #[serde(default)]
    pub total_systems: u32,
    #[serde(default)]
    pub total_interfaces: u32,
    #[serde(default)]
    pub total_apps: u32,
    #[serde(default)]
    pub coupling_score: f64,
    #[serde(default)]
    pub cohesion_score: f64,
}

// --- Transform functions ---

impl HaisenData {
    /// Check if apps have pre-computed layout coordinates.
    pub fn has_layout(&self) -> bool {
        self.apps.iter().any(|a| a.x != 0.0 || a.y != 0.0)
    }

    pub fn to_graph(&self) -> GraphData {
        let mut id_to_idx: HashMap<String, usize> = HashMap::new();
        let mut groups: Vec<String> = Vec::new();
        let mut group_map: HashMap<String, usize> = HashMap::new();
        let mut nodes: Vec<GraphNode> = Vec::new();

        let ensure_group =
            |g: &str, groups: &mut Vec<String>, map: &mut HashMap<String, usize>| -> usize {
                if let Some(&idx) = map.get(g) {
                    idx
                } else {
                    let idx = groups.len();
                    groups.push(g.to_string());
                    map.insert(g.to_string(), idx);
                    idx
                }
            };

        // Apps
        for app in &self.apps {
            let id = if !app.nanoid.is_empty() {
                app.nanoid.clone()
            } else if !app.did.is_empty() {
                app.did.clone()
            } else {
                continue;
            };
            if id_to_idx.contains_key(&id) {
                continue;
            }
            let group = if app.project.is_empty() {
                "unknown"
            } else {
                &app.project
            };
            let gi = ensure_group(group, &mut groups, &mut group_map);
            let idx = nodes.len();
            id_to_idx.insert(id.clone(), idx);
            nodes.push(GraphNode {
                id,
                label: if app.name.is_empty() {
                    app.nanoid.clone()
                } else {
                    app.name.clone()
                },
                group: group.to_string(),
                group_index: gi,
                node_type: app.performer_type.clone(),
                radius: 6.0,
                x: app.x,
                y: app.y,
            });
        }

        // Infra
        for infra in &self.infra {
            let id = infra.name.clone();
            if id.is_empty() || id_to_idx.contains_key(&id) {
                continue;
            }
            let gi = ensure_group("infra", &mut groups, &mut group_map);
            let idx = nodes.len();
            id_to_idx.insert(id.clone(), idx);
            nodes.push(GraphNode {
                id,
                label: infra.name.clone(),
                group: "infra".to_string(),
                group_index: gi,
                node_type: "system".to_string(),
                radius: 10.0,
                x: infra.x,
                y: infra.y,
            });
        }

        // Edges (ensure target nodes exist)
        let mut edges = Vec::new();
        for e in &self.edges {
            if !id_to_idx.contains_key(&e.to) {
                let gi = ensure_group("collection", &mut groups, &mut group_map);
                let idx = nodes.len();
                id_to_idx.insert(e.to.clone(), idx);
                let label = shorten_label(&e.to);
                nodes.push(GraphNode {
                    id: e.to.clone(),
                    label,
                    group: "collection".to_string(),
                    group_index: gi,
                    node_type: "collection".to_string(),
                    radius: 3.0,
                    x: 0.0,
                    y: 0.0,
                });
            }
            if let (Some(&fi), Some(&ti)) = (id_to_idx.get(&e.from), id_to_idx.get(&e.to)) {
                edges.push(GraphEdge {
                    from_idx: fi,
                    to_idx: ti,
                    edge_type: e.edge_type.clone(),
                    label: if e.label.is_empty() {
                        e.edge_type.clone()
                    } else {
                        e.label.clone()
                    },
                });
            }
        }

        GraphData {
            nodes,
            edges,
            groups,
        }
    }
}

impl SoSData {
    pub fn to_graph(&self) -> GraphData {
        let mut id_to_idx: HashMap<String, usize> = HashMap::new();
        let mut groups: Vec<String> = Vec::new();
        let mut group_map: HashMap<String, usize> = HashMap::new();
        let mut nodes: Vec<GraphNode> = Vec::new();

        let ensure_group =
            |g: &str, groups: &mut Vec<String>, map: &mut HashMap<String, usize>| -> usize {
                if let Some(&idx) = map.get(g) {
                    idx
                } else {
                    let idx = groups.len();
                    groups.push(g.to_string());
                    map.insert(g.to_string(), idx);
                    idx
                }
            };

        // Find system → layer mapping
        let mut sys_layer: HashMap<String, String> = HashMap::new();
        for layer in &self.layers {
            for sys in &layer.systems {
                sys_layer.insert(sys.clone(), layer.name.clone());
            }
        }

        for sys in &self.systems {
            if id_to_idx.contains_key(&sys.id) {
                continue;
            }
            let layer = sys_layer
                .get(&sys.id)
                .map(|s| s.as_str())
                .unwrap_or(&sys.system_type);
            let gi = ensure_group(layer, &mut groups, &mut group_map);
            let idx = nodes.len();
            id_to_idx.insert(sys.id.clone(), idx);
            let r = (sys.app_count as f32).sqrt() * 3.0;
            nodes.push(GraphNode {
                id: sys.id.clone(),
                label: sys.id.clone(),
                group: layer.to_string(),
                group_index: gi,
                node_type: sys.system_type.clone(),
                radius: r.max(6.0).min(20.0),
                x: 0.0,
                y: 0.0,
            });
        }

        let mut edges = Vec::new();
        for iface in &self.interfaces {
            // Ensure both endpoints exist
            for ep in [&iface.from, &iface.to] {
                if !id_to_idx.contains_key(ep) {
                    let gi = ensure_group("unknown", &mut groups, &mut group_map);
                    let idx = nodes.len();
                    id_to_idx.insert(ep.clone(), idx);
                    nodes.push(GraphNode {
                        id: ep.clone(),
                        label: ep.clone(),
                        group: "unknown".to_string(),
                        group_index: gi,
                        node_type: "project".to_string(),
                        radius: 6.0,
                        x: 0.0,
                        y: 0.0,
                    });
                }
            }
            if let (Some(&fi), Some(&ti)) = (id_to_idx.get(&iface.from), id_to_idx.get(&iface.to)) {
                edges.push(GraphEdge {
                    from_idx: fi,
                    to_idx: ti,
                    edge_type: iface.protocol.clone(),
                    label: iface.protocol.clone(),
                });
            }
        }

        GraphData {
            nodes,
            edges,
            groups,
        }
    }
}

fn shorten_label(s: &str) -> String {
    if s.len() <= 16 {
        return s.to_string();
    }
    if let Some(rest) = s.strip_prefix("ai.gftd.apps.") {
        let parts: Vec<&str> = rest.split('.').collect();
        if parts.len() > 2 {
            return format!("{}.{}..", parts[0], parts[1]);
        }
        return rest.to_string();
    }
    if let Some(rest) = s.strip_prefix("did:web:") {
        return rest.replace(".gftd.ai", "");
    }
    if let Some(rest) = s.strip_prefix("sql:") {
        return rest.to_string();
    }
    format!("{}..", &s[..14])
}
