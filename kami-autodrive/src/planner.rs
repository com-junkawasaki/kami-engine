//! Global path planning: A* over the inflated occupancy grid, returned as a
//! world-coordinate polyline.

use glam::Vec2;
use kami_pathfind::astar_grid;

use crate::perception::OccupancyGrid;

/// Plan a collision-free path from `start` to `goal` over `grid`.
///
/// `grid` should already be configuration-space inflated (see
/// [`OccupancyGrid::inflated`]). Start/goal are snapped to the nearest free
/// cell. Returns world-frame waypoints (cell centres), line-of-sight
/// simplified, or `None` if no path exists.
pub fn plan(grid: &OccupancyGrid, start: Vec2, goal: Vec2) -> Option<Vec<Vec2>> {
    let s = grid.nearest_free(start)?;
    let g = grid.nearest_free(goal)?;
    let cost = grid.to_cost_grid();
    let cells = astar_grid(&cost, s, g)?;

    let pts: Vec<Vec2> = cells
        .iter()
        .map(|c| grid.cell_to_world(c.x as usize, c.y as usize))
        .collect();

    Some(simplify(&pts, grid))
}

/// Line-of-sight shortcutting: greedily drop intermediate waypoints whose
/// removal keeps the segment collision-free. Produces a sparse, drivable path.
fn simplify(pts: &[Vec2], grid: &OccupancyGrid) -> Vec<Vec2> {
    if pts.len() <= 2 {
        return pts.to_vec();
    }
    let mut out = vec![pts[0]];
    let mut anchor = 0;
    let mut i = 1;
    while i < pts.len() {
        if i == pts.len() - 1 || !segment_clear(pts[anchor], pts[i + 1], grid) {
            out.push(pts[i]);
            anchor = i;
        }
        i += 1;
    }
    out
}

/// Sample the segment `a..b` at sub-cell spacing; clear iff no sample lands on
/// an occupied cell. Thin alias over [`OccupancyGrid::line_clear`].
fn segment_clear(a: Vec2, b: Vec2, grid: &OccupancyGrid) -> bool {
    grid.line_clear(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every consecutive segment of `path` is collision-free on `grid`.
    fn path_is_clear(path: &[Vec2], grid: &OccupancyGrid) -> bool {
        path.windows(2).all(|w| segment_clear(w[0], w[1], grid))
    }

    #[test]
    fn straight_path_on_open_ground() {
        let grid = OccupancyGrid::centered(Vec2::new(5.0, 0.0), 15.0, 0.5);
        let path = plan(&grid, Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)).expect("path");
        assert!(path.len() >= 2);
        assert!(path.first().unwrap().distance(Vec2::ZERO) < 1.0);
        assert!(path.last().unwrap().distance(Vec2::new(10.0, 0.0)) < 1.0);
        // After LOS simplification an open straight is just endpoints.
        assert_eq!(path.len(), 2);
    }

    #[test]
    fn routes_around_a_wall_without_crossing_it() {
        let mut grid = OccupancyGrid::centered(Vec2::new(5.0, 0.0), 15.0, 0.5);
        // Vertical wall x≈5, y∈[-3, 3], leaving gaps above/below.
        let mut y = -3.0;
        while y <= 3.0 {
            grid.mark_world(Vec2::new(5.0, y));
            y += 0.25;
        }
        let inflated = grid.inflated(0.5);
        let path = plan(&inflated, Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)).expect("path");
        assert!(path_is_clear(&path, &inflated), "planned path crosses the wall");
        // A detour is longer than the 10 m straight line.
        let len: f32 = path.windows(2).map(|w| w[0].distance(w[1])).sum();
        assert!(len > 10.0, "expected a detour, got {len:.1} m");
    }
}
