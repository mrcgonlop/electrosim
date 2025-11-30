//! Generate 3D Rotating Video of Emergent Dimensional Structure
//!
//! Creates an MP4 video showing:
//! - 3D rotating view of dimensional automata
//! - Color-coded dimensions (blue=1D, green=2D, yellow=3D, red=4D+)
//! - Multiple camera angles
//! - Evolving slice position

use em_physics_sandbox::physics::{Hypergraph, GraphEmbedding, AdaptiveAutomata};
use em_physics_sandbox::physics::simple_rules::*;
use em_physics_sandbox::visualization::create_rotation_video;
use glam::Vec3;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  3D ROTATING VIDEO: EMERGENT DIMENSIONAL STRUCTURE           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // ========================================================================
    // SCENARIO 1: Random Graph Evolution
    // ========================================================================

    println!("Creating interesting graph structure...");
    let mut graph = Hypergraph::new_random(200, 400);

    println!("  Initial: {} nodes, {} edges", graph.nodes.len(), graph.edges.len());

    // Evolve with mixed rules
    println!("Evolving graph with mixed rules...");
    for step in 0..30 {
        if step % 3 == 0 {
            rule_edge_expand(&mut graph);
        }
        if step % 5 == 0 {
            rule_triangle_merge(&mut graph);
        }
        if step % 7 == 0 {
            rule_pair_creation(&mut graph);
        }

        if step % 10 == 0 {
            println!("  Step {}: {} nodes, {} edges", step, graph.nodes.len(), graph.edges.len());
        }
    }

    println!("  Final: {} nodes, {} edges\n", graph.nodes.len(), graph.edges.len());

    // ========================================================================
    // Embed into 3D
    // ========================================================================

    println!("Embedding graph into 3D space...");
    let embedding = GraphEmbedding::from_hypergraph(&graph, 1000);
    println!("  Embedding energy: {:.2}\n", embedding.energy);

    // ========================================================================
    // Project to Adaptive Automata
    // ========================================================================

    println!("Projecting to adaptive automata...");
    let automata = embedding.to_adaptive_automata(40, 40, 40);

    println!("  Grid: {}x{}x{} = {} cells", automata.nx, automata.ny, automata.nz, automata.cells.len());
    println!("  Avg dimension: {:.2}", automata.avg_dimension);
    println!("  Min dimension: {:.2}", automata.min_dimension);
    println!("  Max dimension: {:.2}\n", automata.max_dimension);

    // ========================================================================
    // SCENARIO 2: Create Interesting Dimensional Structures
    // ========================================================================

    println!("Creating dimensional defects for visualization...");
    let mut automata = AdaptiveAutomata::new_uniform(50, 50, 50, 3.0);

    // Add various dimensional structures
    automata.create_particle_defect(Vec3::new(15.0, 25.0, 25.0), 3.0);
    automata.create_string_defect(
        Vec3::new(25.0, 10.0, 25.0),
        Vec3::new(25.0, 40.0, 25.0),
        2.0
    );
    automata.create_membrane_defect(
        Vec3::new(35.0, 25.0, 25.0),
        Vec3::new(1.0, 0.0, 0.0),
        8.0,
        1.5
    );
    automata.create_4d_bubble(Vec3::new(40.0, 40.0, 25.0), 5.0);

    println!("  Created:");
    println!("    - Point defect (d=0) at (15, 25, 25)");
    println!("    - String defect (d=1) from (25, 10, 25) to (25, 40, 25)");
    println!("    - Membrane defect (d=2) at (35, 25, 25)");
    println!("    - 4D bubble at (40, 40, 25)\n");

    // ========================================================================
    // Generate Video Frames
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("Generating video frames...");
    println!("═══════════════════════════════════════════════════════════════\n");

    let duration = 10.0; // seconds
    let fps = 30;
    let width = 1280;
    let height = 720;

    println!("Settings:");
    println!("  Duration: {} seconds", duration);
    println!("  FPS: {}", fps);
    println!("  Resolution: {}x{}", width, height);
    println!("  Total frames: {}\n", (duration * fps as f32) as usize);

    println!("Rendering frames (this may take a minute)...");
    let video_sequence = create_rotation_video(&automata, duration, fps, width, height);

    println!("  Generated {} frames\n", video_sequence.frames.len());

    // ========================================================================
    // Save Frames
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("Saving frames...");
    println!("═══════════════════════════════════════════════════════════════\n");

    let output_dir = "video_frames";
    match video_sequence.save_frames(output_dir) {
        Ok(()) => {
            println!("\n✓ Success! Frames saved to {}/ directory\n", output_dir);
        }
        Err(e) => {
            eprintln!("Error saving frames: {}", e);
            return;
        }
    }

    // ========================================================================
    // Instructions
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("CREATE VIDEO");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("To create the MP4 video, run:\n");
    println!("  ffmpeg -r {} -i {}frame_%05d.png -c:v libx264 -pix_fmt yuv420p -crf 20 dimensional_rotation.mp4\n",
             fps, output_dir);

    println!("Video features:");
    println!("  • 3D rotating camera (2 complete rotations)");
    println!("  • Oscillating tilt angle");
    println!("  • Moving slice position");
    println!("  • Color-coded dimensions:");
    println!("      - BLUE: 1D string-like structures");
    println!("      - GREEN: 2D membrane-like structures");
    println!("      - YELLOW: 3D normal space");
    println!("      - RED: 4D+ exotic regions\n");

    println!("═══════════════════════════════════════════════════════════════");
    println!("ADVANCED: Create Evolution Video");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("To show graph evolution over time:");
    println!("  1. Create multiway graph");
    println!("  2. Use create_evolution_video()");
    println!("  3. See causality emerge as branching structure!\n");

    println!("This shows:");
    println!("  • How spacetime builds from simple rules");
    println!("  • Quantum branching (multiway paths)");
    println!("  • Dimensional emergence\n");

    println!("Done! 🎬");
}
