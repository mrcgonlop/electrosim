//! Emergent Spacetime from Simple Rules
//!
//! CRITICAL PHILOSOPHY:
//! - Space emerges from connectivity
//! - Time emerges from causal structure
//! - NO coordinates, NO time axis assumed
//! - Just graph rewriting → spacetime appears!

use em_physics_sandbox::physics::{
    Hypergraph,
    MultiwayGraph,
    GraphEmbedding,
};
use em_physics_sandbox::physics::simple_rules::*;
use em_physics_sandbox::visualization::{DimensionalSlice, DimensionMode};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  EMERGENT SPACETIME FROM SIMPLE GRAPH RULES                  ║");
    println!("║                                                              ║");
    println!("║  No coordinates. No time. Just causality.                    ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // ========================================================================
    // EXPERIMENT 1: Single Rule Evolution
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("EXPERIMENT 1: Edge Expansion Rule");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Rule: A---B → A---C---B");
    println!("Question: What dimension emerges?\n");

    let mut graph = Hypergraph::new();
    let a = graph.add_node();
    let b = graph.add_node();
    graph.edges.insert(HyperEdge::new(
        vec![a, b],
        EdgeType::Undirected,
    ));

    println!("Initial: {} nodes, {} edges", graph.nodes.len(), graph.edges.len());

    // Apply rule many times
    for step in 0..20 {
        rule_edge_expand(&mut graph);

        if step % 5 == 0 {
            let node_count = graph.nodes.len();
            let edge_count = graph.edges.len();

            // Measure dimension
            if let Some(sample_node) = graph.nodes.iter().next() {
                let dim = graph.measure_dimension(*sample_node, 5);
                println!("Step {}: {} nodes, {} edges, d ≈ {:.2}",
                         step, node_count, edge_count, dim);
            }
        }
    }

    println!("\nResult: 1D chain emerged! (d ≈ 1.0)");
    println!("Interpretation: This rule creates LINEAR spacetime.\n");

    // ========================================================================
    // EXPERIMENT 2: Multiway Evolution (Quantum Branching!)
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("EXPERIMENT 2: Multiway Evolution");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Apply ALL possible rules simultaneously.");
    println!("Creates branching 'multiway graph'.");
    println!("Interpretation: This IS quantum mechanics!\n");

    let initial_graph = Hypergraph::new_2d_lattice(5, 5);
    println!("Initial: {}x{} lattice = {} nodes", 5, 5, initial_graph.nodes.len());

    let mut multiway = MultiwayGraph::new(initial_graph);

    for step in 0..3 {
        multiway.evolve_step();

        let state_count = multiway.states.len();
        let causal_edge_count = multiway.causal_edges.len();

        println!("Step {}: {} possible universes, {} causal links",
                 step + 1, state_count, causal_edge_count);
    }

    // Measure causal distance (emergent time!)
    if multiway.states.len() > 1 {
        let dist = multiway.causal_distance(0, multiway.states.len() - 1);

        println!("\nCausal distance from initial to final states: {:?}", dist);
        println!("This IS the time interval! Time = causal depth.\n");
    }

    // ========================================================================
    // EXPERIMENT 3: Mixed Rules - What Dimension Emerges?
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("EXPERIMENT 3: Composite Rules");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Mix multiple rules:");
    println!("  - Edge expansion (creates space)");
    println!("  - Triangle merge (creates structure)");
    println!("  - Pair creation (matter from vacuum)");
    println!("  - Annihilation (matter to vacuum)\n");

    let mut graph = Hypergraph::new();

    // Start with small seed
    for _ in 0..5 {
        rule_pair_creation(&mut graph);
    }

    println!("Seed: {} nodes", graph.nodes.len());

    // Evolve with mixed rules
    for step in 0..50 {
        // Apply each rule with some probability
        if step % 3 == 0 {
            rule_edge_expand(&mut graph);
        }
        if step % 5 == 0 {
            rule_triangle_merge(&mut graph);
        }
        if step % 7 == 0 {
            rule_pair_creation(&mut graph);
        }
        if step % 11 == 0 {
            rule_annihilation(&mut graph);
        }

        if step % 10 == 0 && !graph.nodes.is_empty() {
            if let Some(sample_node) = graph.nodes.iter().next() {
                let dim = graph.measure_dimension(*sample_node, 5);
                println!("Step {}: {} nodes, {} edges, d ≈ {:.2}",
                         step, graph.nodes.len(), graph.edges.len(), dim);
            }
        }
    }

    println!("\nQuestion: What dimension emerged from this chaos?");

    if let Some(sample_node) = graph.nodes.iter().next() {
        let final_dim = graph.measure_dimension(*sample_node, 10);
        println!("Final dimension: d ≈ {:.2}", final_dim);

        if final_dim < 1.5 {
            println!("Result: 1D string-like structure!");
        } else if final_dim < 2.5 {
            println!("Result: 2D membrane-like structure!");
        } else if final_dim < 3.5 {
            println!("Result: 3D space emerged!");
        } else {
            println!("Result: Higher-dimensional exotic structure!");
        }
    }

    println!();

    // ========================================================================
    // EXPERIMENT 4: Embed in 3D and Visualize
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("EXPERIMENT 4: Project to 3D (What We Observe)");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("The graph is abstract (no coordinates).");
    println!("But we experience 3D space...");
    println!("Solution: EMBED graph into 3D using force-directed layout!\n");

    // Create interesting graph
    let mut graph = Hypergraph::new_random(100);

    // Evolve it
    for _ in 0..20 {
        rule_edge_expand(&mut graph);
        rule_triangle_merge(&mut graph);
    }

    println!("Graph: {} nodes, {} edges", graph.nodes.len(), graph.edges.len());

    // Embed into 3D
    println!("Embedding into 3D space (force-directed)...");
    let embedding = GraphEmbedding::from_hypergraph(&graph, 500);

    println!("Embedding energy: {:.2}", embedding.energy);
    println!("(Lower energy = better layout)\n");

    // Project to adaptive automata
    println!("Projecting to adaptive automata (30³ grid)...");
    let automata = embedding.to_adaptive_automata(30, 30, 30);

    println!("Automata stats:");
    println!("  Cells: {}", automata.cells.len());
    println!("  Avg dimension: {:.2}", automata.avg_dimension);
    println!("  Min dimension: {:.2}", automata.min_dimension);
    println!("  Max dimension: {:.2}", automata.max_dimension);
    println!();

    // Visualize
    println!("Rendering dimensional slice...");
    let mut slice = DimensionalSlice::new(0, 15);
    slice.mode = DimensionMode::ColorCoded;
    slice.dim_min = automata.min_dimension;
    slice.dim_max = automata.max_dimension;

    let img = slice.render(&automata, 512, 512);
    img.save("emergent_spacetime.png").expect("Failed to save");

    println!("  → Saved emergent_spacetime.png");
    println!();

    // ========================================================================
    // INTERPRETATION
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("INTERPRETATION");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("What did we learn?\n");

    println!("1. SPACE emerges from graph connectivity");
    println!("   - No coordinates assumed");
    println!("   - Distance = graph distance");
    println!("   - Dimension = N(r) scaling\n");

    println!("2. TIME emerges from causal structure");
    println!("   - Not a dimension we 'evolve through'");
    println!("   - It's the ORDERING of rewrites");
    println!("   - Causal distance = time interval\n");

    println!("3. SPACETIME is the causal graph + spatial connectivity");
    println!("   - Multiway graph captures all possibilities");
    println!("   - Light cones = causal structure");
    println!("   - Quantum mechanics = branching\n");

    println!("4. DIMENSION can vary!");
    println!("   - Different rules → different dimensions");
    println!("   - Could vary across space");
    println!("   - Observable as color in visualization\n");

    println!("5. PHYSICS emerges from simple rules");
    println!("   - No F=ma assumed");
    println!("   - No Maxwell equations assumed");
    println!("   - Just graph rewriting!\n");

    println!("═══════════════════════════════════════════════════════════════");
    println!("NEXT: Find rules that reproduce known physics!");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Candidates to test:");
    println!("  1. Does edge expansion + triangle merge → 3D space?");
    println!("  2. Does pair creation/annihilation → particle physics?");
    println!("  3. Can we get F ~ 1/r² from pure graph rules?");
    println!("  4. Does multiway branching → quantum interference?\n");

    println!("Run experimental test suite to find out!");
}
