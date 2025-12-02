//! Graph Optimization and Pruning
//!
//! Inspired by neural network pruning techniques, this module implements:
//! - Magnitude-based pruning (remove unimportant edges)
//! - Gradient-based pruning (physics-aware)
//! - Self-refinement via physics loss minimization
//! - Fractal structure detection
//!
//! Key idea: Hypergraphs should LEARN to represent physics optimally,
//! just like neural networks learn to represent data!

use crate::physics::{Hypergraph, HyperEdge, GraphEmbedding, AdaptiveAutomata};
use std::collections::{HashMap, HashSet};
use glam::Vec3;

// ============================================================================
// PHYSICS LOSS FUNCTION (What we're optimizing for)
// ============================================================================

/// Metrics that physics must satisfy
#[derive(Debug, Clone)]
pub struct PhysicsMetrics {
    pub avg_dimension: f32,
    pub dimensional_variance: f32,
    pub force_law_exponent: f32,  // Should be 2.0 for Coulomb
    pub causal_invariance_score: f32,
    pub num_nodes: usize,
    pub num_edges: usize,
}

impl PhysicsMetrics {
    /// Compute metrics from hypergraph
    pub fn from_hypergraph(graph: &Hypergraph, sample_size: usize) -> Self {
        // Quick embedding for measurement
        let embedding = GraphEmbedding::from_hypergraph(graph, sample_size.min(100));
        let automata = embedding.to_adaptive_automata(30, 30, 30);

        // Dimensional statistics
        let avg_dimension = automata.avg_dimension;
        let dimensional_variance = automata.cells.iter()
            .map(|c| (c.dimension - avg_dimension).powi(2))
            .sum::<f32>() / automata.cells.len() as f32;

        // Force law exponent (very rough estimate)
        let force_law_exponent = estimate_force_law_exponent(&automata);

        // Causal invariance (simplified)
        let causal_invariance_score = graph.edges.len() as f32 / graph.nodes.len() as f32;

        Self {
            avg_dimension,
            dimensional_variance,
            force_law_exponent,
            causal_invariance_score,
            num_nodes: graph.nodes.len(),
            num_edges: graph.edges.len(),
        }
    }

    /// Compute physics loss
    /// Lower is better - means closer to matching experimental physics
    pub fn loss(&self) -> f32 {
        // Target: d=3, F∝1/r²
        let dim_error = (self.avg_dimension - 3.0).powi(2);
        let force_error = (self.force_law_exponent - 2.0).powi(2);

        // Regularization: prefer simpler graphs
        let complexity = self.num_edges as f32 / self.num_nodes.max(1) as f32;
        let complexity_penalty = if complexity > 10.0 {
            (complexity - 10.0).powi(2)
        } else {
            0.0
        };

        // Weighted sum
        10.0 * dim_error +           // Dimension is critical
        100.0 * force_error +        // Force law is VERY critical
        0.1 * complexity_penalty     // Prefer sparse graphs
    }
}

fn estimate_force_law_exponent(automata: &AdaptiveAutomata) -> f32 {
    // Create test defect and measure force at two distances
    // F ∝ 1/r^n, so log(F1/F2) = n * log(r2/r1)

    // For now, return estimate (will implement properly later)
    2.0 + (automata.avg_dimension - 3.0) * 0.5
}

// ============================================================================
// EDGE IMPORTANCE METRICS
// ============================================================================

/// Measure how "important" an edge is to graph structure
pub fn edge_importance(graph: &Hypergraph, edge: &HyperEdge) -> f32 {
    // Combine multiple importance measures

    // 1. Order (higher-order edges connect more nodes)
    let order_importance = edge.nodes.len() as f32;

    // 2. Connectivity (edges connecting high-degree nodes are important)
    let connectivity_importance = edge.nodes.iter()
        .map(|&n| graph.degree(n) as f32)
        .sum::<f32>() / edge.nodes.len() as f32;

    // 3. Centrality (edges on many shortest paths are important)
    let centrality_importance = estimate_betweenness(graph, edge);

    // Weighted combination
    0.3 * order_importance +
    0.4 * connectivity_importance +
    0.3 * centrality_importance
}

fn estimate_betweenness(graph: &Hypergraph, edge: &HyperEdge) -> f32 {
    // Simplified: count how many nodes are reachable through this edge
    // (Full betweenness centrality is expensive)

    let mut reachable = 0;
    for node in &edge.nodes {
        reachable += graph.neighbors(*node).len();
    }

    reachable as f32 / edge.nodes.len() as f32
}

// ============================================================================
// PRUNING STRATEGIES
// ============================================================================

/// Remove edges with importance below threshold
pub fn magnitude_based_pruning(graph: &mut Hypergraph, threshold: f32) -> usize {
    let initial_count = graph.edges.len();

    // Collect edges to keep (avoid borrow checker issues)
    let edges_to_keep: Vec<_> = graph.edges.iter()
        .filter(|edge| edge_importance(graph, edge) >= threshold)
        .cloned()
        .collect();

    graph.edges.clear();
    graph.edges.extend(edges_to_keep);

    let removed = initial_count - graph.edges.len();
    removed
}

/// Remove fraction of least important edges
pub fn prune_fraction(graph: &mut Hypergraph, fraction: f32) -> usize {
    let num_to_remove = (graph.edges.len() as f32 * fraction) as usize;

    // Compute importance for all edges
    let mut edge_importances: Vec<_> = graph.edges.iter()
        .map(|e| (e.clone(), edge_importance(graph, e)))
        .collect();

    // Sort by importance (ascending)
    edge_importances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // Remove least important edges
    for (edge, _) in edge_importances.iter().take(num_to_remove) {
        graph.edges.remove(edge);
    }

    num_to_remove
}

/// Physics-aware pruning: only remove edges that don't hurt physics
pub fn physics_aware_pruning(
    graph: &mut Hypergraph,
    max_loss_increase: f32,
) -> usize {
    // Measure current physics
    let metrics_before = PhysicsMetrics::from_hypergraph(graph, 100);
    let loss_before = metrics_before.loss();

    println!("Initial physics loss: {:.4}", loss_before);

    let mut removed_count = 0;

    // Try removing edges one by one
    let edges_to_test: Vec<_> = graph.edges.iter().cloned().collect();

    for edge in edges_to_test {
        // Compute importance
        let importance = edge_importance(graph, &edge);

        // Skip very important edges
        if importance > 5.0 {
            continue;
        }

        // Try removing
        graph.edges.remove(&edge);

        // Measure physics impact
        let metrics_after = PhysicsMetrics::from_hypergraph(graph, 50);
        let loss_after = metrics_after.loss();

        if loss_after <= loss_before + max_loss_increase {
            // Removal is acceptable!
            removed_count += 1;

            if removed_count % 10 == 0 {
                println!("  Removed {} edges, current loss: {:.4}",
                         removed_count, loss_after);
            }
        } else {
            // Restore edge
            graph.edges.insert(edge);
        }
    }

    println!("Physics-aware pruning removed {} edges", removed_count);
    removed_count
}

// ============================================================================
// STRUCTURED PRUNING: Compress Uniform Regions
// ============================================================================

#[derive(Debug, Clone)]
pub struct CompressedRegion {
    pub bounding_box: (Vec3, Vec3),  // (min, max)
    pub dimension: f32,
    pub node_density: f32,
    pub boundary_nodes: Vec<usize>,
}

/// Find regions with uniform dimension that can be compressed
pub fn find_uniform_regions(
    graph: &Hypergraph,
    min_size: usize,
) -> Vec<CompressedRegion> {
    let embedding = GraphEmbedding::from_hypergraph(graph, 100);
    let automata = embedding.to_adaptive_automata(40, 40, 40);

    let mut regions = Vec::new();

    // Simple approach: find connected regions with similar dimension
    let dim_tolerance = 0.1;

    // Cluster cells by dimension
    let mut dimension_clusters: HashMap<i32, Vec<usize>> = HashMap::new();

    for (idx, cell) in automata.cells.iter().enumerate() {
        let dim_bucket = (cell.dimension * 10.0) as i32;
        dimension_clusters.entry(dim_bucket).or_default().push(idx);
    }

    // Create regions from large clusters
    for (dim_bucket, cell_indices) in dimension_clusters {
        if cell_indices.len() < min_size {
            continue;
        }

        let dimension = dim_bucket as f32 / 10.0;

        // Compute bounding box
        let positions: Vec<_> = cell_indices.iter()
            .map(|&idx| automata.cells[idx].position)
            .collect();

        let min_pos = Vec3::new(
            positions.iter().map(|p| p.x).fold(f32::INFINITY, f32::min),
            positions.iter().map(|p| p.y).fold(f32::INFINITY, f32::min),
            positions.iter().map(|p| p.z).fold(f32::INFINITY, f32::min),
        );

        let max_pos = Vec3::new(
            positions.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max),
            positions.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max),
            positions.iter().map(|p| p.z).fold(f32::NEG_INFINITY, f32::max),
        );

        let volume = (max_pos.x - min_pos.x) *
                     (max_pos.y - min_pos.y) *
                     (max_pos.z - min_pos.z);
        let node_density = cell_indices.len() as f32 / volume.max(1.0);

        regions.push(CompressedRegion {
            bounding_box: (min_pos, max_pos),
            dimension,
            node_density,
            boundary_nodes: Vec::new(),  // TODO: identify boundary
        });
    }

    regions
}

// ============================================================================
// FRACTAL STRUCTURE ANALYSIS
// ============================================================================

/// Measure fractal (box-counting) dimension
pub fn measure_fractal_dimension(graph: &Hypergraph) -> f32 {
    let embedding = GraphEmbedding::from_hypergraph(graph, 100);

    let mut scales = Vec::new();
    let mut counts = Vec::new();

    // Box-counting at different scales
    for box_size in [1.0, 2.0, 4.0, 8.0, 16.0, 32.0] {
        let num_boxes = count_boxes_needed(graph, &embedding, box_size);
        scales.push(box_size.ln());
        counts.push((num_boxes as f32).ln());
    }

    // Fractal dimension from slope: log(N) = -D * log(ε) + const
    linear_regression_slope(&scales, &counts).abs()
}

fn count_boxes_needed(
    graph: &Hypergraph,
    embedding: &GraphEmbedding,
    box_size: f32,
) -> usize {
    let mut boxes = HashSet::new();

    for (_, pos) in &embedding.positions {
        let box_coords = (
            (pos.x / box_size).floor() as i32,
            (pos.y / box_size).floor() as i32,
            (pos.z / box_size).floor() as i32,
        );
        boxes.insert(box_coords);
    }

    boxes.len()
}

fn linear_regression_slope(x: &[f32], y: &[f32]) -> f32 {
    let n = x.len() as f32;
    let sum_x: f32 = x.iter().sum();
    let sum_y: f32 = y.iter().sum();
    let sum_xy: f32 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
    let sum_xx: f32 = x.iter().map(|xi| xi * xi).sum();

    (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x)
}

// ============================================================================
// SELF-REFINEMENT VIA OPTIMIZATION
// ============================================================================

/// Optimize graph to minimize physics loss
pub fn optimize_graph(
    mut graph: Hypergraph,
    iterations: usize,
    perturbations_per_iter: usize,
) -> Hypergraph {
    use crate::physics::simple_rules::*;

    println!("Optimizing hypergraph to match physics...");
    let mut best_loss = PhysicsMetrics::from_hypergraph(&graph, 100).loss();
    println!("Initial loss: {:.4}", best_loss);

    for iter in 0..iterations {
        let mut improved = false;

        // Try random perturbations
        for _ in 0..perturbations_per_iter {
            let mut candidate = graph.clone();

            // Random perturbation
            match rand::random::<u8>() % 6 {
                0 => { rule_edge_expand(&mut candidate); }
                1 => { rule_triangle_merge(&mut candidate); }
                2 => { rule_pair_creation(&mut candidate); }
                3 => { rule_annihilation(&mut candidate); }
                4 => {
                    // Remove random low-importance edge
                    prune_fraction(&mut candidate, 0.01);
                }
                5 => {
                    // Add random edge
                    if candidate.nodes.len() > 1 {
                        let n1 = candidate.nodes.iter().nth(rand::random::<usize>() % candidate.nodes.len()).unwrap();
                        let n2 = candidate.nodes.iter().nth(rand::random::<usize>() % candidate.nodes.len()).unwrap();
                        if n1 != n2 {
                            candidate.edges.insert(HyperEdge {
                                nodes: vec![*n1, *n2],
                                edge_type: crate::physics::EdgeType::Spatial,
                            });
                        }
                    }
                }
                _ => {}
            }

            // Measure loss
            let candidate_loss = PhysicsMetrics::from_hypergraph(&candidate, 50).loss();

            if candidate_loss < best_loss {
                graph = candidate;
                best_loss = candidate_loss;
                improved = true;
            }
        }

        if iter % 10 == 0 || improved {
            let metrics = PhysicsMetrics::from_hypergraph(&graph, 100);
            println!("Iter {}: loss={:.4}, d={:.2}, F_exp={:.2}, N={}, E={}",
                     iter, best_loss,
                     metrics.avg_dimension,
                     metrics.force_law_exponent,
                     metrics.num_nodes,
                     metrics.num_edges);
        }
    }

    println!("Optimization complete. Final loss: {:.4}", best_loss);
    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_metrics() {
        let graph = Hypergraph::new_random(100, 200);
        let metrics = PhysicsMetrics::from_hypergraph(&graph, 50);

        assert!(metrics.avg_dimension > 0.0);
        assert!(metrics.num_nodes == 100);
        assert!(metrics.num_edges == 200);
    }

    #[test]
    fn test_edge_importance() {
        let mut graph = Hypergraph::new();
        let a = graph.add_node();
        let b = graph.add_node();
        let c = graph.add_node();

        let edge_ab = HyperEdge {
            nodes: vec![a, b],
            edge_type: crate::physics::EdgeType::Spatial,
        };

        graph.edges.insert(edge_ab.clone());

        let importance = edge_importance(&graph, &edge_ab);
        assert!(importance > 0.0);
    }

    #[test]
    fn test_pruning() {
        let mut graph = Hypergraph::new_random(100, 200);
        let initial_edges = graph.edges.len();

        let removed = prune_fraction(&mut graph, 0.2);

        assert_eq!(removed, (initial_edges as f32 * 0.2) as usize);
        assert!(graph.edges.len() < initial_edges);
    }
}
