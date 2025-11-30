//! Simple Hypergraph Rewrite Rules
//!
//! CRITICAL PHILOSOPHY:
//! - The hypergraph IS reality (not embedded in spacetime)
//! - Rewrites ARE causality (not events "in time")
//! - Space emerges from connectivity
//! - Time emerges from causal dependencies between rewrites
//!
//! We don't evolve "through time" - we build a causal graph where:
//! - Nodes = graph states
//! - Edges = possible rewrites
//! - Paths through this = "timelines"
//!
//! SPACETIME = causal structure + spatial connectivity!

use crate::physics::hypergraph::{Hypergraph, HyperEdge, EdgeType, NodeID};
use std::collections::HashSet;

/// RULE 1: Binary Edge Expansion
/// Pattern: A---B
/// Replace: A---C---B
///
/// This is the SIMPLEST possible rule. What emerges?
/// - Creates "space" (graph grows)
/// - Creates causal chain (rewrite history)
/// - No time assumed!
pub fn rule_edge_expand(graph: &mut Hypergraph) -> bool {
    // Find a binary edge
    for edge in graph.edges.clone() {
        if edge.nodes.len() == 2 {
            let a = edge.nodes[0];
            let b = edge.nodes[1];

            // Remove old edge
            graph.edges.remove(&edge);

            // Create intermediate node
            let c = graph.add_node();

            // Create two new edges
            graph.edges.insert(HyperEdge { nodes: vec![a, c], edge_type: EdgeType::Spatial });
            graph.edges.insert(HyperEdge { nodes: vec![c, b], edge_type: EdgeType::Spatial });

            return true;  // Applied one rewrite
        }
    }

    false  // No applicable pattern found
}

/// RULE 2: Triangle Merge
/// Pattern: A---B---C with A---C already existing (triangle)
/// Replace: ABC (single 3-hyperedge)
///
/// Effect: Creates higher-order structure
/// Causal interpretation: Three pairwise interactions → single 3-way interaction
pub fn rule_triangle_merge(graph: &mut Hypergraph) -> bool {
    let nodes: Vec<NodeID> = graph.nodes.iter().copied().collect();

    for i in 0..nodes.len() {
        for j in i+1..nodes.len() {
            for k in j+1..nodes.len() {
                let (a, b, c) = (nodes[i], nodes[j], nodes[k]);

                // Check if triangle exists (3 binary edges)
                let ab_exists = graph.has_edge(&[a, b]);
                let bc_exists = graph.has_edge(&[b, c]);
                let ca_exists = graph.has_edge(&[c, a]);

                if ab_exists && bc_exists && ca_exists {
                    // Remove three binary edges
                    graph.edges.retain(|e| {
                        !(e.nodes.len() == 2 &&
                          ((e.nodes.contains(&a) && e.nodes.contains(&b)) ||
                           (e.nodes.contains(&b) && e.nodes.contains(&c)) ||
                           (e.nodes.contains(&c) && e.nodes.contains(&a))))
                    });

                    // Create single 3-hyperedge
                    graph.edges.insert(HyperEdge { nodes: vec![a, b, c], edge_type: EdgeType::Spatial });

                    return true;
                }
            }
        }
    }

    false
}

/// RULE 3: Hyperedge Split
/// Pattern: ABC (3-hyperedge)
/// Replace: A---B, B---C, C---A (triangle)
///
/// Inverse of triangle merge. Creates/destroys structure.
pub fn rule_hyperedge_split(graph: &mut Hypergraph) -> bool {
    for edge in graph.edges.clone() {
        if edge.nodes.len() == 3 {
            let a = edge.nodes[0];
            let b = edge.nodes[1];
            let c = edge.nodes[2];

            // Remove hyperedge
            graph.edges.remove(&edge);

            // Create three binary edges
            graph.edges.insert(HyperEdge { nodes: vec![a, b], edge_type: EdgeType::Spatial });
            graph.edges.insert(HyperEdge { nodes: vec![b, c], edge_type: EdgeType::Spatial });
            graph.edges.insert(HyperEdge { nodes: vec![c, a], edge_type: EdgeType::Spatial });

            return true;
        }
    }

    false
}

/// RULE 4: Node Pair Creation (from vacuum)
/// Pattern: (nothing)
/// Replace: A---B
///
/// Creates matter from nothing! Conservation violation?
/// Or is this how "virtual particles" emerge?
pub fn rule_pair_creation(graph: &mut Hypergraph) -> bool {
    let a = graph.add_node();
    let b = graph.add_node();

    graph.edges.insert(HyperEdge { nodes: vec![a, b], edge_type: EdgeType::Spatial });

    true
}

/// RULE 5: Isolated Node Annihilation
/// Pattern: A (with no edges)
/// Replace: (nothing)
///
/// Destroys isolated matter. Balances pair creation.
pub fn rule_annihilation(graph: &mut Hypergraph) -> bool {
    for &node in graph.nodes.clone().iter() {
        if graph.neighbors(node).is_empty() {
            graph.nodes.remove(&node);
            graph.node_properties.remove(&node);
            return true;
        }
    }

    false
}

/// RULE 6: Path Shortcut (Emergent Causality!)
/// Pattern: A---B---C (path of length 2)
/// Replace: A---C and B (B becomes isolated)
///
/// Key insight: This creates "timelike" vs "spacelike" separation!
/// - If A→B→C are causally connected, shortcut is "FTL"
/// - If graph prevents shortcut, that's "lightcone"!
pub fn rule_path_shortcut(graph: &mut Hypergraph) -> bool {
    let nodes: Vec<NodeID> = graph.nodes.iter().copied().collect();

    for &a in &nodes {
        let neighbors_a = graph.neighbors(a);

        for &b in &neighbors_a {
            let neighbors_b = graph.neighbors(b);

            for &c in &neighbors_b {
                if c != a && !neighbors_a.contains(&c) {
                    // Path A-B-C exists but not A-C

                    // Remove A-B and B-C
                    graph.edges.retain(|e| {
                        !(e.nodes.len() == 2 &&
                          ((e.nodes.contains(&a) && e.nodes.contains(&b)) ||
                           (e.nodes.contains(&b) && e.nodes.contains(&c))))
                    });

                    // Create A-C shortcut
                    graph.edges.insert(HyperEdge { nodes: vec![a, c], edge_type: EdgeType::Spatial });

                    // B becomes isolated (may be annihilated later)

                    return true;
                }
            }
        }
    }

    false
}

/// MULTIWAY EVOLUTION
/// Instead of picking ONE rule to apply, apply ALL possible rules!
/// This creates a branching "multiway graph" where:
/// - Each branch = different rewrite sequence
/// - Quantum mechanics = exploring all branches
/// - Measurement = picking a branch
///
/// This is Wolfram's key insight: quantum superposition IS multiway branching!
pub struct MultiwayGraph {
    /// All possible graph states
    pub states: Vec<Hypergraph>,

    /// Which state led to which (causal edges)
    pub causal_edges: Vec<(usize, usize, String)>,  // (from, to, rule_name)
}

impl MultiwayGraph {
    pub fn new(initial_graph: Hypergraph) -> Self {
        Self {
            states: vec![initial_graph],
            causal_edges: Vec::new(),
        }
    }

    /// Evolve one step: apply ALL possible rules to ALL states
    pub fn evolve_step(&mut self) {
        let current_state_count = self.states.len();
        let mut new_states = Vec::new();
        let mut new_causal_edges = Vec::new();

        for state_idx in 0..current_state_count {
            let graph = self.states[state_idx].clone();  // Clone upfront

            // Try each rule
            let rules: Vec<(&str, fn(&mut Hypergraph) -> bool)> = vec![
                ("edge_expand", rule_edge_expand),
                ("triangle_merge", rule_triangle_merge),
                ("hyperedge_split", rule_hyperedge_split),
                ("pair_creation", rule_pair_creation),
                ("annihilation", rule_annihilation),
                ("path_shortcut", rule_path_shortcut),
            ];

            for (rule_name, rule_fn) in rules {
                let mut new_graph = graph.clone();

                if rule_fn(&mut new_graph) {
                    // Rule was applicable! Create new branch
                    let new_state_idx = self.states.len() + new_states.len();
                    new_states.push(new_graph);
                    new_causal_edges.push((state_idx, new_state_idx, rule_name.to_string()));
                }
            }
        }

        // Add all new states at once
        self.states.extend(new_states);
        self.causal_edges.extend(new_causal_edges);
    }

    /// Measure causal distance between states
    /// This is EMERGENT TIME!
    pub fn causal_distance(&self, from: usize, to: usize) -> Option<usize> {
        // BFS through causal graph
        use std::collections::VecDeque;

        let mut queue = VecDeque::new();
        let mut visited = vec![false; self.states.len()];
        let mut distances = vec![usize::MAX; self.states.len()];

        queue.push_back(from);
        visited[from] = true;
        distances[from] = 0;

        while let Some(state) = queue.pop_front() {
            if state == to {
                return Some(distances[to]);
            }

            // Find all states this can evolve to
            for &(from_state, to_state, _) in &self.causal_edges {
                if from_state == state && !visited[to_state] {
                    visited[to_state] = true;
                    distances[to_state] = distances[state] + 1;
                    queue.push_back(to_state);
                }
            }
        }

        None  // Not causally connected!
    }
}

/// Causal Invariance Check
/// Do different rewrite orders reach the same state?
/// If YES: physics is consistent!
/// If NO: we have "time travel paradoxes"
pub fn check_causal_invariance(graph: &Hypergraph, steps: usize) -> bool {
    let mut multiway = MultiwayGraph::new(graph.clone());

    for _ in 0..steps {
        multiway.evolve_step();
    }

    // Check if different paths converge
    // (Simple version: check if final states are similar)

    if multiway.states.len() < 2 {
        return true;  // Trivially invariant
    }

    // Compare graph structures
    let final_states: Vec<_> = multiway.states.iter()
        .skip(multiway.states.len() - 10)
        .collect();

    // If most final states have similar node/edge counts, probably invariant
    let avg_nodes = final_states.iter()
        .map(|g| g.nodes.len())
        .sum::<usize>() as f32 / final_states.len() as f32;

    let variance = final_states.iter()
        .map(|g| {
            let diff = g.nodes.len() as f32 - avg_nodes;
            diff * diff
        })
        .sum::<f32>() / final_states.len() as f32;

    variance < avg_nodes * 0.5  // Low variance = causal invariance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_expand() {
        let mut graph = Hypergraph::new();
        let a = graph.add_node();
        let b = graph.add_node();
        graph.edges.insert(HyperEdge { nodes: vec![a, b], edge_type: EdgeType::Spatial });

        rule_edge_expand(&mut graph);

        // Should have 3 nodes and 2 edges now
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn test_multiway_evolution() {
        let mut graph = Hypergraph::new();
        let a = graph.add_node();
        let b = graph.add_node();
        graph.edges.insert(HyperEdge { nodes: vec![a, b], edge_type: EdgeType::Spatial });

        let mut multiway = MultiwayGraph::new(graph);
        multiway.evolve_step();

        // Should have created multiple branches
        assert!(multiway.states.len() > 1);
        assert!(!multiway.causal_edges.is_empty());
    }

    #[test]
    fn test_causal_distance() {
        let graph = Hypergraph::new();
        let mut multiway = MultiwayGraph::new(graph);

        multiway.evolve_step();
        multiway.evolve_step();

        // Check causal distance from initial state
        if multiway.states.len() > 1 {
            let dist = multiway.causal_distance(0, multiway.states.len() - 1);
            assert!(dist.is_some());
            assert!(dist.unwrap() <= 2);
        }
    }
}
