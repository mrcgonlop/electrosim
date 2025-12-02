//! Performance Profiling for Hypergraph \u2192 Spacetime Pipeline
//!
//! Identifies bottlenecks in:
//! - Graph evolution (rule application)
//! - Force-directed embedding
//! - Automata projection
//! - Video rendering
//!
//! Goal: Scale to 10^6+ nodes for realistic EM simulations

use em_physics_sandbox::physics::{Hypergraph, GraphEmbedding, AdaptiveAutomata};
use em_physics_sandbox::physics::simple_rules::*;
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  PERFORMANCE PROFILING: Hypergraph -> Spacetime Pipeline    ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Test different graph sizes
    let test_sizes = vec![
        (50, 100),      // Quick test
        (200, 400),     // Current demo size
        (1000, 2000),   // Medium scale
        (5000, 10000),  // Large scale (approaching electron)
        // (50000, 100000),  // Realistic electron? (commented out for now)
    ];

    println!("Testing graph sizes:");
    for (n_nodes, n_edges) in &test_sizes {
        println!("  - {} nodes, {} edges", n_nodes, n_edges);
    }
    println!("\n{}\n", "=".repeat(70));

    for (n_nodes, n_edges) in test_sizes {
        profile_pipeline(n_nodes, n_edges);
        println!("\n{}\n", "=".repeat(70));
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  BOTTLENECK ANALYSIS                                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("Expected bottlenecks:");
    println!("  1. Force-directed embedding: O(n² × iterations)");
    println!("     - Most expensive for large graphs");
    println!("     - GPU acceleration: compute forces in parallel");
    println!("");
    println!("  2. Rule application: O(|E| × rules)");
    println!("     - Edge pattern matching");
    println!("     - GPU: parallel pattern detection");
    println!("");
    println!("  3. Automata projection: O(grid³ × nodes)");
    println!("     - Spatial queries for each cell");
    println!("     - GPU: parallel cell computation");
    println!("");
    println!("  4. Rendering: O(grid³ × frames)");
    println!("     - Perspective projection per cell");
    println!("     - GPU shaders: already parallelizable");
    println!("");
    println!("Optimization strategies:");
    println!("  • Use wgpu for GPU compute shaders");
    println!("  • Spatial indexing (octree/kd-tree) for embedding");
    println!("  • Graph compression: remove redundant structure");
    println!("  • Sparse representations for large grids");
    println!("  • Only compute visible cells for rendering");
}

fn profile_pipeline(n_nodes: usize, n_edges: usize) {
    println!("PROFILING: {} nodes, {} edges", n_nodes, n_edges);

    // ========================================================================
    // 1. Graph Creation
    // ========================================================================
    let start = Instant::now();
    let mut graph = Hypergraph::new_random(n_nodes, n_edges);
    let creation_time = start.elapsed();
    println!("  Graph creation:     {:>8.2?}", creation_time);

    // ========================================================================
    // 2. Rule Application (10 steps)
    // ========================================================================
    let start = Instant::now();
    for step in 0..10 {
        if step % 3 == 0 { rule_edge_expand(&mut graph); }
        if step % 5 == 0 { rule_triangle_merge(&mut graph); }
        if step % 7 == 0 { rule_pair_creation(&mut graph); }
    }
    let evolution_time = start.elapsed();
    let evolution_per_step = evolution_time / 10;
    println!("  Evolution (10 steps): {:>8.2?}  ({:>8.2?}/step)",
             evolution_time, evolution_per_step);

    let final_nodes = graph.nodes.len();
    let final_edges = graph.edges.len();
    println!("    Final: {} nodes, {} edges", final_nodes, final_edges);

    // ========================================================================
    // 3. Force-Directed Embedding (Critical Bottleneck!)
    // ========================================================================

    // Test with different iteration counts
    let iteration_counts = if n_nodes <= 200 {
        vec![50, 100, 200]
    } else if n_nodes <= 1000 {
        vec![50, 100]
    } else {
        vec![50]  // Only quick test for large graphs
    };

    for iterations in iteration_counts {
        let start = Instant::now();
        let embedding = GraphEmbedding::from_hypergraph(&graph, iterations);
        let embedding_time = start.elapsed();
        let time_per_iter = embedding_time / iterations as u32;

        println!("  Embedding ({:>3} iter):  {:>8.2?}  ({:>8.2?}/iter, E={:.1})",
                 iterations, embedding_time, time_per_iter, embedding.energy);
    }

    // Use quick embedding for rest of pipeline
    let embedding = GraphEmbedding::from_hypergraph(&graph, 50);

    // ========================================================================
    // 4. Automata Projection
    // ========================================================================

    // Scale grid size with graph size (but not linearly)
    let grid_size = if n_nodes <= 200 {
        30
    } else if n_nodes <= 1000 {
        40
    } else {
        50
    };

    let start = Instant::now();
    let automata = embedding.to_adaptive_automata(grid_size, grid_size, grid_size);
    let projection_time = start.elapsed();

    println!("  Automata proj ({}³):  {:>8.2?}  (avg d={:.2})",
             grid_size, projection_time, automata.avg_dimension);

    // ========================================================================
    // 5. Rendering (single frame)
    // ========================================================================

    use em_physics_sandbox::visualization::VideoRenderer;

    let start = Instant::now();
    let renderer = VideoRenderer::new(640, 480);
    let _frame = renderer.render_automata_3d(&automata, 0.0, 0.3, grid_size / 2);
    let render_time = start.elapsed();

    println!("  Render (1 frame):   {:>8.2?}", render_time);

    // ========================================================================
    // Summary
    // ========================================================================

    let total_time = creation_time + evolution_time + projection_time + render_time;
    println!("  TOTAL (no embed):   {:>8.2?}", total_time);
}
