use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use glam::Vec3;
use crate::parser::bvh::Bvh;

const STEP_SIZE: f32 = 32.0;
const PLAYER_CLEARANCE_RADIUS: f32 = 18.0;
const MAX_ITERATIONS: usize = 400;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridNode {
    pub x: i32,
    pub y: i32,
}

impl GridNode {
    pub fn from_vec3(pos: Vec3) -> Self {
        Self {
            x: (pos.x / STEP_SIZE).round() as i32,
            y: (pos.y / STEP_SIZE).round() as i32,
        }
    }

    pub fn to_vec3(&self, z: f32) -> Vec3 {
        Vec3::new(self.x as f32 * STEP_SIZE, self.y as f32 * STEP_SIZE, z)
    }
}

#[derive(Clone, Copy, Debug)]
struct SearchNode {
    grid: GridNode,
    f_score: u32,
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_score == other.f_score
    }
}

impl Eq for SearchNode {}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap behavior for BinaryHeap
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Checks if a movement step from `start` to `end` has full 3D player body clearance against BVH map geometry.
/// Tests left, center, and right body bounds at feet, waist, and head height.
pub fn is_step_valid(bvh: &Bvh, start: Vec3, end: Vec3) -> bool {
    let dir = end - start;
    let len = dir.length();
    if len < 0.001 {
        return true;
    }

    let dir_2d = Vec3::new(dir.x, dir.y, 0.0);
    let perp = if dir_2d.length_squared() > 0.0001 {
        let norm = dir_2d.normalize();
        Vec3::new(-norm.y, norm.x, 0.0) * PLAYER_CLEARANCE_RADIUS
    } else {
        Vec3::ZERO
    };

    let heights = [12.0, 36.0, 60.0];
    for &h in &heights {
        let z = Vec3::new(0.0, 0.0, h);
        let s_center = start + z;
        let e_center = end + z;

        if !bvh.has_line_of_sight(s_center, e_center) {
            return false;
        }

        if perp != Vec3::ZERO {
            if !bvh.has_line_of_sight(s_center + perp, e_center + perp) {
                return false;
            }
            if !bvh.has_line_of_sight(s_center - perp, e_center - perp) {
                return false;
            }
        }
    }

    true
}

/// Finds a path from `start` to `target` using A* pathfinding.
/// If direct line of sight with body clearance exists, returns `[target]`.
/// Otherwise, uses BVH line-of-sight clearance checks to navigate around map obstacles.
pub fn find_path(start: Vec3, target: Vec3, bvh: Option<&Bvh>) -> Vec3 {
    let Some(bvh) = bvh else {
        return target;
    };

    // 1. Check direct line of sight with full body clearance first
    if is_step_valid(bvh, start, target) {
        return target;
    }

    let start_node = GridNode::from_vec3(start);
    let goal_node = GridNode::from_vec3(target);

    if start_node == goal_node {
        return target;
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<GridNode, GridNode> = HashMap::new();
    let mut g_score: HashMap<GridNode, f32> = HashMap::new();
    let mut closed_set: HashSet<GridNode> = HashSet::new();

    g_score.insert(start_node, 0.0);
    let initial_h = (start - target).length();
    open_set.push(SearchNode {
        grid: start_node,
        f_score: (initial_h * 100.0) as u32,
    });

    let neighbors_offset: [(i32, i32, f32); 8] = [
        (1, 0, 1.0),
        (-1, 0, 1.0),
        (0, 1, 1.0),
        (0, -1, 1.0),
        (1, 1, 1.414),
        (1, -1, 1.414),
        (-1, 1, 1.414),
        (-1, -1, 1.414),
    ];

    let mut best_node = start_node;
    let mut best_dist = initial_h;
    let mut iterations = 0;

    while let Some(current) = open_set.pop() {
        iterations += 1;
        if iterations >= MAX_ITERATIONS {
            break;
        }

        if current.grid == goal_node {
            best_node = goal_node;
            break;
        }

        if !closed_set.insert(current.grid) {
            continue;
        }

        let current_pos = current.grid.to_vec3(start.z);
        let dist_to_goal = (current_pos - target).length();
        if dist_to_goal < best_dist {
            best_dist = dist_to_goal;
            best_node = current.grid;
        }

        let current_g = *g_score.get(&current.grid).unwrap_or(&f32::INFINITY);

        for &(dx, dy, cost_multiplier) in &neighbors_offset {
            let neighbor_grid = GridNode {
                x: current.grid.x + dx,
                y: current.grid.y + dy,
            };

            if closed_set.contains(&neighbor_grid) {
                continue;
            }

            let neighbor_pos = neighbor_grid.to_vec3(start.z);

            // Check if movement step from current to neighbor has full body clearance
            if !is_step_valid(bvh, current_pos, neighbor_pos) {
                continue;
            }

            let step_cost = STEP_SIZE * cost_multiplier;
            let tentative_g = current_g + step_cost;

            if tentative_g < *g_score.get(&neighbor_grid).unwrap_or(&f32::INFINITY) {
                came_from.insert(neighbor_grid, current.grid);
                g_score.insert(neighbor_grid, tentative_g);

                let h = (neighbor_pos - target).length();
                let f = tentative_g + h;
                open_set.push(SearchNode {
                    grid: neighbor_grid,
                    f_score: (f * 100.0) as u32,
                });
            }
        }
    }

    // Reconstruct path from best_node back to start_node
    let mut raw_path = Vec::new();
    let mut curr = best_node;
    raw_path.push(target); // Append final target position
    while curr != start_node {
        raw_path.push(curr.to_vec3(start.z));
        if let Some(&prev) = came_from.get(&curr) {
            curr = prev;
        } else {
            break;
        }
    }
    raw_path.reverse();

    if raw_path.is_empty() {
        return target;
    }

    // Path Smoothing / String Pulling: find furthest waypoint with full body clearance
    let mut target_idx = 0;
    for (idx, waypoint) in raw_path.iter().enumerate().skip(1) {
        if is_step_valid(bvh, start, *waypoint) {
            target_idx = idx;
        } else {
            break;
        }
    }

    let mut target_waypoint = raw_path[target_idx];

    // If the smoothed waypoint is already close to the player (< 30 units), e.g. at a corner tip,
    // advance to the next node in the path so the player smoothly rounds the 90-degree turn instead of stalling.
    if (target_waypoint - start).length() < 30.0 && target_idx + 1 < raw_path.len() {
        target_waypoint = raw_path[target_idx + 1];
    }

    target_waypoint
}
