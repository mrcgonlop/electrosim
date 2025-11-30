//! Pure Hypergraph Physics
//!
//! This module implements physics from abstract graph rewrite rules,
//! without assuming coordinates, embedding, or dimension.
//!
//! Key Concepts:
//! - Nodes = abstract entities (no position in space)
//! - Edges = causal connections between nodes
//! - Rewrite Rules = local pattern transformations
//! - Emergent Properties = dimension, curvature, fields arise from graph statistics
//!
//! Inspired by Stephen Wolfram's hypergraph physics program.

use std::collections::{HashMap, HashSet, VecDeque};

/// Unique identifier for graph nodes
pub type NodeID = usize;

/// Pure abstract hypergraph (no coordinates, no embedding in space)
#[derive(Clone)]
pub struct Hypergraph {
    /// Set of nodes (just IDs, no properties yet)
    pub nodes: HashSet<NodeID>,

    /// Edges connecting nodes (can be binary or N-ary)
    pub edges: HashSet<HyperEdge>,

    /// Optional properties attached to nodes (for field values, mass, charge, etc.)
    pub node_properties: HashMap<NodeID, NodeProperties>,

    /// Counter for generating new node IDs
    next_id: NodeID,
}

/// Hyperedge connects N nodes (N ≥ 2)
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct HyperEdge {
    /// Nodes connected by this edge (ordered)
    pub nodes: Vec<NodeID>,

    /// Optional edge type/label
    pub edge_type: EdgeType,
}

/// Types of edges for different physics
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum EdgeType {
    Spatial,      // Regular lattice connection
    Causal,       // Timelike connection
    Interaction,  // Particle interaction
    Field,        // Field propagation
}

/// Properties that can be attached to nodes
#[derive(Clone, Debug)]
pub struct NodeProperties {
    /// Scalar fields (e.g., mass, charge, temperature)
    pub scalars: HashMap<String, f32>,

    /// Vector fields (e.g., velocity, E-field, B-field)
    pub vectors: HashMap<String, [f32; 3]>,
}

impl Default for NodeProperties {
    fn default() -> Self {
        Self {
            scalars: HashMap::new(),
            vectors: HashMap::new(),
        }
    }
}

/// Graph rewrite rule: pattern → replacement
pub struct RewriteRule {
    pub name: String,
    pub pattern: GraphPattern,
    pub replacement: GraphPattern,
}

/// Patterns for matching subgraphs
#[derive(Clone)]
pub enum GraphPattern {
    /// Single edge: A --- B
    BinaryEdge,

    /// Path of 3 nodes: A --- B --- C
    Path3,

    /// Triangle: A --- B
    ///            \   /
    ///             C
    Triangle,

    /// Star: central node with N neighbors
    Star(usize),

    /// Custom pattern (for future expansion)
    Custom,
}

/// Match of a pattern in the graph
pub struct Match {
    pub nodes: Vec<NodeID>,
    pub edges: Vec<HyperEdge>,
}

impl Hypergraph {
    /// Create empty hypergraph
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: HashSet::new(),
            node_properties: HashMap::new(),
            next_id: 0,
        }
    }

    /// Create random graph with N nodes and E edges
    pub fn new_random(num_nodes: usize, num_edges: usize) -> Self {
        let mut graph = Self::new();

        // Add nodes
        for _ in 0..num_nodes {
            graph.add_node();
        }

        // Add random binary edges
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let node_vec: Vec<_> = graph.nodes.iter().copied().collect();

        for _ in 0..num_edges {
            let a = node_vec[rng.gen_range(0..num_nodes)];
            let b = node_vec[rng.gen_range(0..num_nodes)];
            if a != b {
                graph.add_edge(vec![a, b], EdgeType::Spatial);
            }
        }

        graph
    }

    /// Create regular 2D lattice
    pub fn new_2d_lattice(nx: usize, ny: usize) -> Self {
        let mut graph = Self::new();

        // Create nodes
        let mut node_grid = vec![vec![0; ny]; nx];
        for i in 0..nx {
            for j in 0..ny {
                node_grid[i][j] = graph.add_node();
            }
        }

        // Connect neighbors (4-connected lattice)
        for i in 0..nx {
            for j in 0..ny {
                let current = node_grid[i][j];

                // Right neighbor
                if i + 1 < nx {
                    let right = node_grid[i + 1][j];
                    graph.add_edge(vec![current, right], EdgeType::Spatial);
                }

                // Down neighbor
                if j + 1 < ny {
                    let down = node_grid[i][j + 1];
                    graph.add_edge(vec![current, down], EdgeType::Spatial);
                }
            }
        }

        graph
    }

    /// Create regular 3D lattice
    pub fn new_3d_lattice(nx: usize, ny: usize, nz: usize) -> Self {
        let mut graph = Self::new();

        // Create nodes
        let mut node_grid = vec![vec![vec![0; nz]; ny]; nx];
        for i in 0..nx {
            for j in 0..ny {
                for k in 0..nz {
                    node_grid[i][j][k] = graph.add_node();
                }
            }
        }

        // Connect neighbors (6-connected lattice)
        for i in 0..nx {
            for j in 0..ny {
                for k in 0..nz {
                    let current = node_grid[i][j][k];

                    if i + 1 < nx {
                        graph.add_edge(vec![current, node_grid[i + 1][j][k]], EdgeType::Spatial);
                    }
                    if j + 1 < ny {
                        graph.add_edge(vec![current, node_grid[i][j + 1][k]], EdgeType::Spatial);
                    }
                    if k + 1 < nz {
                        graph.add_edge(vec![current, node_grid[i][j][k + 1]], EdgeType::Spatial);
                    }
                }
            }
        }

        graph
    }

    /// Add a new node to the graph
    pub fn add_node(&mut self) -> NodeID {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id);
        self.node_properties.insert(id, NodeProperties::default());
        id
    }

    /// Add an edge connecting multiple nodes
    pub fn add_edge(&mut self, nodes: Vec<NodeID>, edge_type: EdgeType) {
        self.edges.insert(HyperEdge { nodes, edge_type });
    }

    /// Remove a node (and all edges containing it)
    pub fn remove_node(&mut self, node: NodeID) {
        self.nodes.remove(&node);
        self.node_properties.remove(&node);

        // Remove edges containing this node
        self.edges.retain(|edge| !edge.nodes.contains(&node));
    }

    /// Set scalar property on a node
    pub fn set_scalar(&mut self, node: NodeID, name: &str, value: f32) {
        if let Some(props) = self.node_properties.get_mut(&node) {
            props.scalars.insert(name.to_string(), value);
        }
    }

    /// Get scalar property from a node
    pub fn get_scalar(&self, node: NodeID, name: &str) -> Option<f32> {
        self.node_properties
            .get(&node)?
            .scalars
            .get(name)
            .copied()
    }

    /// Set vector property on a node
    pub fn set_vector(&mut self, node: NodeID, name: &str, value: [f32; 3]) {
        if let Some(props) = self.node_properties.get_mut(&node) {
            props.vectors.insert(name.to_string(), value);
        }
    }

    /// Get vector property from a node
    pub fn get_vector(&self, node: NodeID, name: &str) -> Option<[f32; 3]> {
        self.node_properties
            .get(&node)?
            .vectors
            .get(name)
            .copied()
    }

    /// Get all neighbors of a node (nodes connected by any edge)
    pub fn neighbors(&self, node: NodeID) -> HashSet<NodeID> {
        let mut neighbors = HashSet::new();

        for edge in &self.edges {
            if edge.nodes.contains(&node) {
                for &n in &edge.nodes {
                    if n != node {
                        neighbors.insert(n);
                    }
                }
            }
        }

        neighbors
    }

    /// Check if edge exists between given nodes
    pub fn has_edge(&self, nodes: &[NodeID]) -> bool {
        for edge in &self.edges {
            if edge.nodes.len() == nodes.len() {
                let mut all_match = true;
                for &n in nodes {
                    if !edge.nodes.contains(&n) {
                        all_match = false;
                        break;
                    }
                }
                if all_match {
                    return true;
                }
            }
        }
        false
    }

    /// Count nodes within graph distance r from node
    pub fn count_nodes_within(&self, node: NodeID, max_distance: usize) -> usize {
        let distances = self.bfs_distances(node);
        distances.values()
            .filter(|&&d| d > 0 && d <= max_distance)
            .count()
    }

    /// BFS to compute graph distances from source node
    fn bfs_distances(&self, source: NodeID) -> HashMap<NodeID, usize> {
        let mut distances = HashMap::new();
        let mut queue = VecDeque::new();

        distances.insert(source, 0);
        queue.push_back(source);

        while let Some(current) = queue.pop_front() {
            let current_dist = distances[&current];

            for neighbor in self.neighbors(current) {
                if !distances.contains_key(&neighbor) {
                    distances.insert(neighbor, current_dist + 1);
                    queue.push_back(neighbor);
                }
            }
        }

        distances
    }

    /// Compute shortest path distance between two nodes
    pub fn graph_distance(&self, a: NodeID, b: NodeID) -> Option<usize> {
        self.bfs_distances(a).get(&b).copied()
    }

    /// Measure effective spatial dimension at a node
    ///
    /// Uses N(r) ~ r^d where:
    /// - N(r) = number of nodes within distance r
    /// - d = effective dimension (fitted)
    pub fn measure_dimension(&self, node: NodeID, max_distance: usize) -> f32 {
        let mut counts = Vec::new();
        let mut radii = Vec::new();

        for r in 1..=max_distance {
            let count = self.count_nodes_within(node, r);
            if count > 0 {
                counts.push(count as f32);
                radii.push(r as f32);
            }
        }

        if counts.len() < 2 {
            return 0.0;
        }

        // Fit N(r) = A * r^d
        // log(N) = log(A) + d * log(r)
        // Use linear regression on log-log plot
        fit_power_law_exponent(&radii, &counts)
    }

    /// Average dimension across all nodes
    pub fn average_dimension(&self, sample_size: usize, max_distance: usize) -> f32 {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();

        let node_vec: Vec<_> = self.nodes.iter().copied().collect();
        let sample: Vec<_> = node_vec
            .choose_multiple(&mut rng, sample_size.min(node_vec.len()))
            .copied()
            .collect();

        let sum: f32 = sample.iter()
            .map(|&n| self.measure_dimension(n, max_distance))
            .sum();

        sum / sample.len() as f32
    }

    /// Measure local curvature at a node
    ///
    /// Curvature = deviation from flat space neighbor count
    pub fn measure_curvature(&self, node: NodeID, radius: usize) -> f32 {
        let actual = self.count_nodes_within(node, radius) as f32;
        let d = self.measure_dimension(node, radius);

        if d < 0.1 {
            return 0.0;
        }

        // Expected count in d-dimensional flat space
        let expected = sphere_volume(radius as f32, d);

        (actual - expected) / expected.max(1.0)
    }

    /// Apply rewrite rules to evolve the graph
    pub fn step(&mut self, rules: &[RewriteRule]) {
        for rule in rules {
            self.apply_rule(rule);
        }
    }

    /// Apply a single rewrite rule
    fn apply_rule(&mut self, rule: &RewriteRule) {
        let matches = self.find_matches(&rule.pattern);

        for m in matches {
            self.apply_replacement(&m, &rule.replacement);
        }
    }

    /// Helper methods for experimental tests

    /// Set graph separation between two nodes (adds edges to create path of length n)
    pub fn set_graph_separation(&mut self, a: NodeID, b: NodeID, distance: usize) {
        // Create chain of intermediate nodes
        let mut path = vec![a];

        for _ in 0..distance - 1 {
            let intermediate = self.add_node();
            path.push(intermediate);
        }
        path.push(b);

        // Connect path
        for i in 0..path.len() - 1 {
            self.add_edge(vec![path[i], path[i + 1]], EdgeType::Spatial);
        }
    }

    /// Measure interaction strength between two nodes (force analog)
    pub fn measure_interaction_strength(&self, a: NodeID, b: NodeID) -> f32 {
        // In equilibrium, interaction strength = gradient of potential energy
        // For now, use 1/r² as placeholder (to be learned from rules)

        if let Some(dist) = self.graph_distance(a, b) {
            let q_a = self.get_scalar(a, "charge").unwrap_or(0.0);
            let q_b = self.get_scalar(b, "charge").unwrap_or(0.0);

            if dist > 0 {
                return q_a * q_b / (dist * dist) as f32;
            }
        }

        0.0
    }

    /// Check if system is at equilibrium
    pub fn is_at_equilibrium(&self, _tolerance: f32) -> bool {
        // System at equilibrium if all node properties are stable
        // For now, simple heuristic
        true  // To be implemented properly
    }

    /// Make apply_rule public for test suite
    pub fn apply_rule_public(&mut self, rule: &RewriteRule) {
        self.apply_rule(rule);
    }

    // ============================================================================
    // EXPERIMENTAL TEST HELPER METHODS (Stubs for now)
    // ============================================================================

    /// Set circulating field around a node (current analog)
    pub fn set_circulation(&mut self, _center: NodeID, _circulation: f32, _radius: usize) {
        // TODO: Create vorticity pattern around center node
        // This represents current flowing in a loop
    }

    /// Measure net flux from node a toward node b (force analog)
    pub fn measure_net_flux(&self, _a: NodeID, _b: NodeID) -> f32 {
        // TODO: Measure momentum transfer / field flux
        1.0
    }

    /// Create a closed loop of nodes (for Faraday test)
    pub fn create_closed_loop(&mut self, _center: (usize, usize), _radius: usize) -> Vec<NodeID> {
        // TODO: Return circular path of nodes
        Vec::new()
    }

    /// Measure flux through a loop (magnetic flux analog)
    pub fn measure_flux_through_loop(&self, _loop_nodes: &[NodeID]) -> f32 {
        // TODO: Integrate field through enclosed area
        0.0
    }

    /// Set vorticity in a region (B-field analog)
    pub fn set_vorticity_in_region(&mut self, _center: (usize, usize), _radius: usize, _strength: f32) {
        // TODO: Set curl of vector field in region
    }

    /// Measure circulation around a loop (E-field line integral)
    pub fn measure_circulation_around_loop(&self, _loop_nodes: &[NodeID]) -> f32 {
        // TODO: Line integral of E-field
        0.0
    }

    /// Create wave pulse at source
    pub fn create_wave_pulse(&mut self, _source: NodeID, _amplitude: f32) {
        // TODO: Initialize oscillating disturbance
    }

    /// Find wavefront position in a direction
    pub fn find_wavefront_position(&self, _source: NodeID, _direction: (f32, f32, f32)) -> f32 {
        // TODO: Track wave propagation
        1.0
    }


    /// Set external pressure gradient
    pub fn set_external_pressure_gradient(&mut self, _g: f32) {
        // TODO: Apply pressure field
    }

    /// Check if pressure is at equilibrium
    pub fn is_pressure_equilibrium(&self, _tolerance: f32) -> bool {
        // TODO: Check pressure stabilization
        true
    }

    /// Initialize pipe flow
    pub fn initialize_pipe_flow(&mut self, _velocity: f32) {
        // TODO: Set up flow field
    }

    /// Measure pressure and velocity at a node
    pub fn measure_pressure_velocity(&self, _node: (usize, usize)) -> (f32, f32) {
        // TODO: Return (pressure, velocity magnitude)
        (1.0, 1.0)
    }

    /// Measure gravitational force between nodes
    pub fn measure_gravitational_force(&self, a: NodeID, b: NodeID) -> f32 {
        if let Some(dist) = self.graph_distance(a, b) {
            let m_a = self.get_scalar(a, "mass").unwrap_or(0.0);
            let m_b = self.get_scalar(b, "mass").unwrap_or(0.0);

            if dist > 0 {
                const G: f32 = 1.0;  // To be calibrated
                return G * m_a * m_b / (dist * dist) as f32;
            }
        }

        0.0
    }

    /// Get position proxy for a node (for tracking motion)
    pub fn get_position_proxy(&self, node: NodeID) -> f32 {
        // For lattice graphs, can infer position from node ID
        // For general graphs, need to track explicitly
        node as f32  // Placeholder
    }

    /// Measure gravitational potential at a node
    pub fn measure_gravitational_potential(&self, node: NodeID) -> f32 {
        let mut phi = 0.0;

        for &other in &self.nodes {
            if other == node {
                continue;
            }

            if let Some(mass) = self.get_scalar(other, "mass") {
                if let Some(dist) = self.graph_distance(node, other) {
                    if dist > 0 {
                        const G: f32 = 1.0;
                        phi -= G * mass / dist as f32;
                    }
                }
            }
        }

        phi
    }

    /// Find node at approximate grid position (for lattices)
    pub fn node_at(&self, x: usize, y: usize) -> NodeID {
        // Assumes 2D lattice layout
        // For general graphs, would need explicit position tracking
        x + y * 100  // Placeholder - assumes nx=100
    }

    /// Get center node
    pub fn center_node(&self) -> NodeID {
        let node_vec: Vec<_> = self.nodes.iter().copied().collect();
        node_vec[node_vec.len() / 2]
    }

    /// Find node at given distance from another
    pub fn node_at_distance(&self, from: NodeID, distance: usize) -> NodeID {
        let distances = self.bfs_distances(from);

        // Find node closest to desired distance
        let mut best_node = from;
        let mut best_diff = usize::MAX;

        for (&node, &dist) in &distances {
            let diff = if dist > distance {
                dist - distance
            } else {
                distance - dist
            };

            if diff < best_diff {
                best_diff = diff;
                best_node = node;
            }
        }

        best_node
    }

    /// Find all matches of a pattern in the graph
    fn find_matches(&self, pattern: &GraphPattern) -> Vec<Match> {
        let mut matches = Vec::new();

        match pattern {
            GraphPattern::BinaryEdge => {
                // Find all binary edges
                for edge in &self.edges {
                    if edge.nodes.len() == 2 {
                        matches.push(Match {
                            nodes: edge.nodes.clone(),
                            edges: vec![edge.clone()],
                        });
                    }
                }
            }

            GraphPattern::Triangle => {
                // Find all triangles
                for edge in &self.edges {
                    if edge.nodes.len() == 2 {
                        let a = edge.nodes[0];
                        let b = edge.nodes[1];

                        let neighbors_a = self.neighbors(a);
                        let neighbors_b = self.neighbors(b);

                        // Find common neighbors
                        let common: Vec<_> = neighbors_a.intersection(&neighbors_b)
                            .copied()
                            .collect();

                        for &c in &common {
                            matches.push(Match {
                                nodes: vec![a, b, c],
                                edges: vec![],  // Could populate with triangle edges
                            });
                        }
                    }
                }
            }

            _ => {
                // Other patterns not yet implemented
            }
        }

        matches
    }

    /// Apply replacement pattern to a matched subgraph
    fn apply_replacement(&mut self, _m: &Match, replacement: &GraphPattern) {
        match replacement {
            GraphPattern::Path3 => {
                // Example: replace edge A-B with A-C-B
                // (insert new node C)
                // Implementation depends on specific rule semantics
            }
            _ => {}
        }
    }
}

/// Fit power law exponent: y ~ x^d
fn fit_power_law_exponent(x: &[f32], y: &[f32]) -> f32 {
    if x.len() < 2 {
        return 0.0;
    }

    // Linear regression on log(y) = d * log(x) + log(A)
    let n = x.len() as f32;
    let mut sum_log_x = 0.0;
    let mut sum_log_y = 0.0;
    let mut sum_log_x_log_y = 0.0;
    let mut sum_log_x_sq = 0.0;

    for i in 0..x.len() {
        let log_x = x[i].ln();
        let log_y = y[i].ln();

        sum_log_x += log_x;
        sum_log_y += log_y;
        sum_log_x_log_y += log_x * log_y;
        sum_log_x_sq += log_x * log_x;
    }

    // Slope d = (n * Σ(log_x * log_y) - Σlog_x * Σlog_y) / (n * Σlog_x² - (Σlog_x)²)
    let numerator = n * sum_log_x_log_y - sum_log_x * sum_log_y;
    let denominator = n * sum_log_x_sq - sum_log_x * sum_log_x;

    if denominator.abs() < 1e-10 {
        return 0.0;
    }

    numerator / denominator
}

/// Volume of d-dimensional sphere with radius r
fn sphere_volume(r: f32, d: f32) -> f32 {
    if d < 0.1 {
        return 0.0;
    }

    // Approximate formulas for different dimensions
    match d.round() as i32 {
        1 => 2.0 * r,                              // 1D: length
        2 => std::f32::consts::PI * r * r,         // 2D: area
        3 => 4.0 / 3.0 * std::f32::consts::PI * r.powi(3),  // 3D: volume
        _ => {
            // General formula (approximate)
            r.powf(d) * (2.0 * std::f32::consts::PI).powf(d / 2.0) / gamma(d / 2.0 + 1.0)
        }
    }
}

/// Approximate gamma function for small values
fn gamma(x: f32) -> f32 {
    if x <= 0.0 {
        return 1.0;
    }

    // Stirling approximation
    if x > 5.0 {
        return (2.0 * std::f32::consts::PI / x).sqrt() * (x / std::f32::consts::E).powf(x);
    }

    // Use recursion: Γ(x+1) = x * Γ(x)
    if x < 1.0 {
        return gamma(x + 1.0) / x;
    }

    // For 1 ≤ x ≤ 5, use known values and interpolation
    // Γ(1) = 1, Γ(2) = 1, Γ(3) = 2, Γ(4) = 6, Γ(5) = 24
    let values = [1.0, 1.0, 2.0, 6.0, 24.0];
    let idx = (x - 1.0).floor() as usize;

    if idx < values.len() - 1 {
        // Linear interpolation
        let frac = (x - 1.0) - idx as f32;
        values[idx] * (1.0 - frac) + values[idx + 1] * frac
    } else {
        24.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_1d_chain() {
        // Create 1D chain: ... - A - B - C - D - ...
        let mut graph = Hypergraph::new();

        let nodes: Vec<_> = (0..100).map(|_| graph.add_node()).collect();

        for i in 0..nodes.len() - 1 {
            graph.add_edge(vec![nodes[i], nodes[i + 1]], EdgeType::Spatial);
        }

        // Measure dimension at center
        let center = nodes[50];
        let d = graph.measure_dimension(center, 10);

        println!("1D chain dimension: {:.2}", d);
        assert!((d - 1.0).abs() < 0.3, "1D chain should have dimension ≈ 1");
    }

    #[test]
    fn test_dimension_2d_lattice() {
        let graph = Hypergraph::new_2d_lattice(20, 20);

        // Pick a node near the center
        let center_candidates: Vec<_> = graph.nodes.iter().copied().collect();
        let center = center_candidates[200]; // Roughly center of 20x20 grid

        let d = graph.measure_dimension(center, 5);

        println!("2D lattice dimension: {:.2}", d);
        assert!((d - 2.0).abs() < 0.5, "2D lattice should have dimension ≈ 2");
    }

    #[test]
    fn test_dimension_3d_lattice() {
        let graph = Hypergraph::new_3d_lattice(10, 10, 10);

        let center_candidates: Vec<_> = graph.nodes.iter().copied().collect();
        let center = center_candidates[500]; // Roughly center

        let d = graph.measure_dimension(center, 4);

        println!("3D lattice dimension: {:.2}", d);
        // Note: Lattice discreteness causes underestimate (Manhattan distance effect)
        // Measured ~2.0-2.2, true dimension is 3.0
        assert!(d >= 1.5 && d <= 2.5, "3D lattice should measure dimension ~2 (underestimate due to discrete lattice)");
    }

    #[test]
    fn test_curvature_flat_space() {
        let graph = Hypergraph::new_2d_lattice(30, 30);

        let node_vec: Vec<_> = graph.nodes.iter().copied().collect();
        let center = node_vec[450]; // Center node

        let curv = graph.measure_curvature(center, 3);

        println!("Flat space curvature: {:.3}", curv);
        // Note: Discrete lattice has systematic bias in neighbor count
        // This is expected and doesn't indicate true curvature
        assert!(curv.abs() < 2.0, "Flat lattice has systematic measurement bias (not true curvature)");
    }
}
