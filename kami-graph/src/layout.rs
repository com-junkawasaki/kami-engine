//! Force-directed graph layout (Fruchterman-Reingold style).
//!
//! Pure computation — no GPU dependency. Produces (x, y) positions for each node.

use crate::data::GraphData;

pub struct LayoutNode {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub group_index: usize,
    pub fixed: bool,
}

pub struct LayoutEdge {
    pub from_idx: usize,
    pub to_idx: usize,
    pub edge_type: String,
}

pub struct ForceLayout {
    pub nodes: Vec<LayoutNode>,
    pub edges: Vec<LayoutEdge>,
    pub alpha: f32,
    pub alpha_decay: f32,
    pub alpha_min: f32,
    charge: f32,
    link_dist: f32,
    damping: f32,
}

impl ForceLayout {
    /// Create layout with golden angle spiral initial positions.
    pub fn new(data: &GraphData) -> Self {
        Self::with_positions(data, None)
    }

    /// Create layout with optional pre-computed (x, y) positions.
    /// If positions are provided and non-zero, those nodes start fixed in place.
    pub fn with_positions(data: &GraphData, positions: Option<&[(f32, f32)]>) -> Self {
        let n = data.nodes.len();
        let nodes: Vec<LayoutNode> = data
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let (x, y) = if let Some(pos) = positions {
                    if i < pos.len() && (pos[i].0 != 0.0 || pos[i].1 != 0.0) {
                        (pos[i].0, pos[i].1)
                    } else {
                        let angle = i as f32 * 2.399;
                        let r = (i as f32).sqrt() * 8.0;
                        (r * angle.cos(), r * angle.sin())
                    }
                } else {
                    let angle = i as f32 * 2.399;
                    let r = (i as f32).sqrt() * 8.0;
                    (r * angle.cos(), r * angle.sin())
                };
                LayoutNode {
                    x,
                    y,
                    vx: 0.0,
                    vy: 0.0,
                    radius: node.radius,
                    group_index: node.group_index,
                    fixed: false,
                }
            })
            .collect();

        let edges: Vec<LayoutEdge> = data
            .edges
            .iter()
            .map(|e| LayoutEdge {
                from_idx: e.from_idx,
                to_idx: e.to_idx,
                edge_type: e.edge_type.clone(),
            })
            .collect();

        // Adaptive parameters based on graph size
        let charge = if n > 500 { -80.0 } else { -200.0 };
        let link_dist = if n > 500 { 50.0 } else { 100.0 };

        ForceLayout {
            nodes,
            edges,
            alpha: 1.0,
            alpha_decay: 0.005,
            alpha_min: 0.001,
            charge,
            link_dist,
            damping: 0.85,
        }
    }

    /// Run one tick of the simulation. Returns false when converged.
    pub fn tick(&mut self) -> bool {
        if self.alpha < self.alpha_min {
            return false;
        }

        let n = self.nodes.len();

        // Charge repulsion (O(n^2) — fine for <3000 nodes)
        // For very large graphs, skip distant pairs
        let skip_threshold = if n > 1000 { 500.0 * 500.0 } else { f32::MAX };

        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.nodes[j].x - self.nodes[i].x;
                let dy = self.nodes[j].y - self.nodes[i].y;
                let d2 = (dx * dx + dy * dy).max(1.0);
                if d2 > skip_threshold {
                    continue;
                }
                let d = d2.sqrt();
                let f = self.charge * self.alpha / d2;
                let fx = dx / d * f;
                let fy = dy / d * f;
                self.nodes[i].vx -= fx;
                self.nodes[i].vy -= fy;
                self.nodes[j].vx += fx;
                self.nodes[j].vy += fy;
            }
        }

        // Link spring
        for edge in &self.edges {
            let (s, t) = if edge.from_idx < edge.to_idx {
                let (left, right) = self.nodes.split_at_mut(edge.to_idx);
                (&mut left[edge.from_idx], &mut right[0])
            } else if edge.from_idx > edge.to_idx {
                let (left, right) = self.nodes.split_at_mut(edge.from_idx);
                (&mut right[0], &mut left[edge.to_idx])
            } else {
                continue;
            };

            let dx = t.x - s.x;
            let dy = t.y - s.y;
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            let f = (d - self.link_dist) * 0.05 * self.alpha;
            let fx = dx / d * f;
            let fy = dy / d * f;
            s.vx += fx;
            s.vy += fy;
            t.vx -= fx;
            t.vy -= fy;
        }

        // Center gravity
        for node in &mut self.nodes {
            node.vx -= node.x * 0.005 * self.alpha;
            node.vy -= node.y * 0.005 * self.alpha;
        }

        // Integrate
        for node in &mut self.nodes {
            if node.fixed {
                node.vx = 0.0;
                node.vy = 0.0;
                continue;
            }
            node.vx *= self.damping;
            node.vy *= self.damping;
            node.x += node.vx;
            node.y += node.vy;
        }

        self.alpha = (self.alpha - self.alpha_decay).max(self.alpha_min);
        true
    }

    /// Run simulation until converged (up to max_ticks).
    pub fn run(&mut self, max_ticks: u32) {
        for _ in 0..max_ticks {
            if !self.tick() {
                break;
            }
        }
    }

    pub fn reheat(&mut self) {
        self.alpha = 1.0;
    }

    pub fn is_settled(&self) -> bool {
        self.alpha <= self.alpha_min
    }
}

// --- Merkle DAG PCB Layout ---
//
// Layered layout following data flow direction (Merkle DAG):
//
//   Y=0    ┌─────────────────── Layer 0: Writers (apps that write) ──────────┐
//          │  [app] [app] [app] ...   sorted by project, placed in columns  │
//          └────────────────────────────────────────────────────────────────────┘
//                    │ write                    │ write
//   Y=BUS  ═══════ collection bus lines (shared data, horizontal) ═══════════
//                    │ subscribe/read           │ subscribe/read
//   Y=READ ┌─────────────────── Layer 2: Readers (apps that read) ───────────┐
//          │  [app] [app] [app] ...   sorted by project, placed in columns  │
//          └────────────────────────────────────────────────────────────────────┘
//
// Deterministic, O(n). Data flows top → bottom (writer → collection → reader).

const DAG_CELL_W: f32 = 6.0; // horizontal spacing
const DAG_CELL_H: f32 = 4.0; // vertical spacing within layer
const DAG_LAYER_GAP: f32 = 60.0; // gap between writer/bus/reader layers
const DAG_BUS_GAP: f32 = 2.0; // gap between bus lines

/// Bus line: a shared collection with its Y coordinate.
pub struct BusLine {
    pub collection: String,
    pub y: f32,
    pub subscriber_count: usize,
}

pub struct PcbLayout {
    pub nodes: Vec<LayoutNode>,
    pub edges: Vec<LayoutEdge>,
    pub buses: Vec<BusLine>,
}

impl PcbLayout {
    pub fn new(data: &GraphData) -> Self {
        let mut nodes: Vec<LayoutNode> = data
            .nodes
            .iter()
            .map(|n| {
                let radius = if n.node_type == "collection" {
                    0.0
                } else {
                    n.radius
                };
                LayoutNode {
                    x: 0.0,
                    y: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    radius,
                    group_index: n.group_index,
                    fixed: true,
                }
            })
            .collect();

        let edges: Vec<LayoutEdge> = data
            .edges
            .iter()
            .map(|e| LayoutEdge {
                from_idx: e.from_idx,
                to_idx: e.to_idx,
                edge_type: e.edge_type.clone(),
            })
            .collect();

        // Classify app nodes into DAG layers by edge type
        let mut is_writer = vec![false; nodes.len()];
        let mut is_reader = vec![false; nodes.len()];
        for edge in &edges {
            match edge.edge_type.as_str() {
                "writes" => {
                    is_writer[edge.from_idx] = true;
                }
                "reads" | "subscribe" => {
                    is_reader[edge.from_idx] = true;
                }
                "invoke" => {
                    is_writer[edge.from_idx] = true;
                } // invoke = active
                _ => {}
            }
        }

        // Layer assignment: writer (top), reader-only (bottom), both = writer layer
        // Collection nodes stay at radius=0, placed on bus
        let mut writer_nodes: Vec<usize> = Vec::new();
        let mut reader_nodes: Vec<usize> = Vec::new();

        for (i, node) in data.nodes.iter().enumerate() {
            if node.node_type == "collection" {
                continue;
            }
            if is_writer[i] {
                writer_nodes.push(i);
            } else if is_reader[i] {
                reader_nodes.push(i);
            } else {
                reader_nodes.push(i); // orphans go to reader layer
            }
        }

        // Sort each layer by group_index (project) for consistent column placement
        writer_nodes.sort_by_key(|&i| (data.nodes[i].group_index, i));
        reader_nodes.sort_by_key(|&i| (data.nodes[i].group_index, i));

        // Place writers at Y=0 (top), spread horizontally
        for (col, &ni) in writer_nodes.iter().enumerate() {
            nodes[ni].x = col as f32 * DAG_CELL_W;
            nodes[ni].y = 0.0;
        }

        // Identify bus collections
        let mut target_counts: std::collections::HashMap<usize, usize> =
            std::collections::HashMap::new();
        for edge in &edges {
            *target_counts.entry(edge.to_idx).or_default() += 1;
        }
        let mut bus_targets: Vec<(usize, usize)> = target_counts
            .into_iter()
            .filter(|&(_, count)| count >= 3)
            .collect();
        bus_targets.sort_by(|a, b| b.1.cmp(&a.1));

        // Place bus lines in middle layer
        let bus_y_start = DAG_LAYER_GAP;
        let mut buses = Vec::new();
        for (i, &(node_idx, count)) in bus_targets.iter().enumerate() {
            let y = bus_y_start + i as f32 * DAG_BUS_GAP;
            nodes[node_idx].x = 0.0;
            nodes[node_idx].y = y;
            nodes[node_idx].radius = 0.0;
            buses.push(BusLine {
                collection: data.nodes[node_idx].id.clone(),
                y,
                subscriber_count: count,
            });
        }

        // Place readers below bus lines
        let reader_y_start = bus_y_start + (buses.len() as f32 + 2.0) * DAG_BUS_GAP + DAG_LAYER_GAP;
        for (col, &ni) in reader_nodes.iter().enumerate() {
            nodes[ni].x = col as f32 * DAG_CELL_W;
            nodes[ni].y = reader_y_start;
        }

        PcbLayout {
            nodes,
            edges,
            buses,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{GraphData, GraphEdge, GraphNode};

    #[test]
    fn test_basic_layout() {
        let data = GraphData {
            nodes: vec![
                GraphNode {
                    id: "a".into(),
                    label: "A".into(),
                    group: "g".into(),
                    group_index: 0,
                    node_type: "service".into(),
                    radius: 6.0,
                    x: 0.0,
                    y: 0.0,
                },
                GraphNode {
                    id: "b".into(),
                    label: "B".into(),
                    group: "g".into(),
                    group_index: 0,
                    node_type: "service".into(),
                    radius: 6.0,
                    x: 0.0,
                    y: 0.0,
                },
                GraphNode {
                    id: "c".into(),
                    label: "C".into(),
                    group: "g".into(),
                    group_index: 0,
                    node_type: "service".into(),
                    radius: 6.0,
                    x: 0.0,
                    y: 0.0,
                },
            ],
            edges: vec![
                GraphEdge {
                    from_idx: 0,
                    to_idx: 1,
                    edge_type: "invoke".into(),
                    label: "call".into(),
                },
                GraphEdge {
                    from_idx: 1,
                    to_idx: 2,
                    edge_type: "writes".into(),
                    label: "data".into(),
                },
            ],
            groups: vec!["g".into()],
        };

        let mut layout = ForceLayout::new(&data);
        layout.run(500);

        // Nodes should have spread out
        let d01 = ((layout.nodes[0].x - layout.nodes[1].x).powi(2)
            + (layout.nodes[0].y - layout.nodes[1].y).powi(2))
        .sqrt();
        assert!(d01 > 10.0, "nodes should be separated: d={d01}");
        assert!(layout.is_settled());
    }
}
