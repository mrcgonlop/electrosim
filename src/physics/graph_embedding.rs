//! Graph Embedding: Project abstract hypergraph into 3D space
//!
//! Key insight: DON'T assume 3D coordinates. Instead:
//! 1. Let graph connectivity define distance
//! 2. Use force-directed layout to embed in 3D
//! 3. Dimension emerges from connectivity, not imposed
//!
//! This is the bridge: Hypergraph (abstract) → 3D space (observable)

use crate::physics::hypergraph::{Hypergraph, NodeID};
use crate::physics::adaptive_automata::{AdaptiveAutomata, Cell};
use glam::Vec3;
use std::collections::HashMap;

/// Force-directed graph embedding into 3D
pub struct GraphEmbedding {
    /// Node positions in 3D (emergent from connectivity!)
    pub positions: HashMap<NodeID, Vec3>,

    /// Local dimension at each node (measured from graph)
    pub dimensions: HashMap<NodeID, f32>,

    /// Energy of current layout (lower = better)
    pub energy: f32,
}

impl GraphEmbedding {
    /// Embed hypergraph into 3D using force-directed layout
    pub fn from_hypergraph(graph: &Hypergraph, iterations: usize) -> Self {
        let mut positions = HashMap::new();
        let mut dimensions = HashMap::new();

        // Initialize with random positions
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for &node in &graph.nodes {
            let pos = Vec3::new(
                rng.gen_range(-10.0..10.0),
                rng.gen_range(-10.0..10.0),
                rng.gen_range(-10.0..10.0),
            );
            positions.insert(node, pos);

            // Measure dimension from graph connectivity
            let dim = graph.measure_dimension(node, 5);
            dimensions.insert(node, dim);
        }

        let mut embedding = Self {
            positions,
            dimensions,
            energy: f32::INFINITY,
        };

        // Optimize layout with force-directed algorithm
        embedding.optimize(graph, iterations);

        embedding
    }

    /// Force-directed layout optimization
    fn optimize(&mut self, graph: &Hypergraph, iterations: usize) {
        let dt = 0.01;
        let k_spring = 1.0;   // Spring constant for connected nodes
        let k_repel = 0.1;    // Repulsion between all nodes
        let damping = 0.9;

        let mut velocities: HashMap<NodeID, Vec3> = HashMap::new();
        for &node in &graph.nodes {
            velocities.insert(node, Vec3::ZERO);
        }

        for iter in 0..iterations {
            let mut forces: HashMap<NodeID, Vec3> = HashMap::new();

            // Initialize forces
            for &node in &graph.nodes {
                forces.insert(node, Vec3::ZERO);
            }

            // Spring forces: connected nodes attract
            for edge in &graph.edges {
                for i in 0..edge.nodes.len() {
                    for j in i+1..edge.nodes.len() {
                        let n1 = edge.nodes[i];
                        let n2 = edge.nodes[j];

                        if let (Some(&pos1), Some(&pos2)) = (self.positions.get(&n1), self.positions.get(&n2)) {
                            let delta = pos2 - pos1;
                            let dist = delta.length().max(0.01);

                            // Ideal distance based on graph distance
                            let ideal_dist = if let Some(d) = graph.graph_distance(n1, n2) {
                                d as f32
                            } else {
                                5.0
                            };

                            let force_mag = k_spring * (dist - ideal_dist);
                            let force = delta.normalize() * force_mag;

                            *forces.get_mut(&n1).unwrap() += force;
                            *forces.get_mut(&n2).unwrap() -= force;
                        }
                    }
                }
            }

            // Repulsive forces: all nodes repel
            let nodes_vec: Vec<NodeID> = graph.nodes.iter().copied().collect();
            for i in 0..nodes_vec.len() {
                for j in i+1..nodes_vec.len() {
                    let n1 = nodes_vec[i];
                    let n2 = nodes_vec[j];

                    if let (Some(&pos1), Some(&pos2)) = (self.positions.get(&n1), self.positions.get(&n2)) {
                        let delta = pos2 - pos1;
                        let dist = delta.length().max(0.1);

                        let force_mag = k_repel / (dist * dist);
                        let force = delta.normalize() * force_mag;

                        *forces.get_mut(&n1).unwrap() -= force;
                        *forces.get_mut(&n2).unwrap() += force;
                    }
                }
            }

            // Update velocities and positions
            for &node in &graph.nodes {
                if let Some(force) = forces.get(&node) {
                    let vel = velocities.get_mut(&node).unwrap();
                    *vel = (*vel + *force * dt) * damping;

                    let pos = self.positions.get_mut(&node).unwrap();
                    *pos += *vel * dt;
                }
            }

            // Compute energy every 10 iterations
            if iter % 10 == 0 {
                self.energy = self.compute_energy(graph);
            }
        }
    }

    /// Compute layout energy (for monitoring convergence)
    fn compute_energy(&self, graph: &Hypergraph) -> f32 {
        let mut energy = 0.0;

        // Energy from edge stretching
        for edge in &graph.edges {
            for i in 0..edge.nodes.len() {
                for j in i+1..edge.nodes.len() {
                    let n1 = edge.nodes[i];
                    let n2 = edge.nodes[j];

                    if let (Some(&pos1), Some(&pos2)) = (self.positions.get(&n1), self.positions.get(&n2)) {
                        let dist = (pos2 - pos1).length();
                        let ideal_dist = graph.graph_distance(n1, n2).unwrap_or(5) as f32;
                        let stretch = (dist - ideal_dist).abs();
                        energy += stretch * stretch;
                    }
                }
            }
        }

        energy
    }

    /// Project embedding onto regular 3D grid (for adaptive automata)
    pub fn to_adaptive_automata(&self, nx: usize, ny: usize, nz: usize) -> AdaptiveAutomata {
        // Find bounding box
        let mut min_pos = Vec3::splat(f32::INFINITY);
        let mut max_pos = Vec3::splat(f32::NEG_INFINITY);

        for pos in self.positions.values() {
            min_pos = min_pos.min(*pos);
            max_pos = max_pos.max(*pos);
        }

        // Create grid
        let spacing = ((max_pos - min_pos) / Vec3::new(nx as f32, ny as f32, nz as f32))
            .max_element()
            .max(0.1);

        let mut cells = Vec::new();
        let mut cell_map: HashMap<(usize, usize, usize), Vec<NodeID>> = HashMap::new();

        // Map nodes to grid cells
        for (&node, &pos) in &self.positions {
            let grid_pos = ((pos - min_pos) / spacing).floor();
            let i = (grid_pos.x as usize).min(nx - 1);
            let j = (grid_pos.y as usize).min(ny - 1);
            let k = (grid_pos.z as usize).min(nz - 1);

            cell_map.entry((i, j, k)).or_insert_with(Vec::new).push(node);
        }

        // Create cells with measured dimension
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let position = min_pos + Vec3::new(i as f32, j as f32, k as f32) * spacing;

                    // Average dimension of nodes in this cell
                    let dimension = if let Some(nodes) = cell_map.get(&(i, j, k)) {
                        let avg_dim: f32 = nodes.iter()
                            .filter_map(|&n| self.dimensions.get(&n))
                            .sum::<f32>() / nodes.len().max(1) as f32;
                        avg_dim
                    } else {
                        // Empty cells default to average of neighbors
                        3.0  // or interpolate from nearby cells
                    };

                    cells.push(Cell::new(position, dimension));
                }
            }
        }

        let mut automata = AdaptiveAutomata {
            cells,
            nx,
            ny,
            nz,
            spacing,
            background_dimension: 3.0,
            avg_dimension: 0.0,
            min_dimension: 0.0,
            max_dimension: 0.0,
        };

        automata.update_neighbors();
        automata.update_statistics();

        automata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_graph_embedding() {
        let graph = Hypergraph::new_random(50);
        let embedding = GraphEmbedding::from_hypergraph(&graph, 100);

        assert_eq!(embedding.positions.len(), 50);
        assert!(embedding.energy < f32::INFINITY);
    }

    #[test]
    fn test_lattice_embedding() {
        let graph = Hypergraph::new_2d_lattice(10, 10);
        let embedding = GraphEmbedding::from_hypergraph(&graph, 200);

        // Should form roughly 2D structure
        let avg_dim = embedding.dimensions.values().sum::<f32>() / embedding.dimensions.len() as f32;
        assert!(avg_dim > 1.5 && avg_dim < 2.5, "2D lattice should have d≈2, got {}", avg_dim);
    }
}
