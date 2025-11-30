//! Quick 3D Rotating Video Demo (Faster Settings)
//!
//! Optimized version for quick visualization:
//! - Smaller graph (50 nodes vs 200)
//! - Fewer embedding iterations (200 vs 1000)
//! - Smaller grid (30x30x30 vs 50x50x50)
//! - Shorter video (5 seconds vs 10)

use em_physics_sandbox::physics::{Hypergraph, GraphEmbedding, AdaptiveAutomata};
use em_physics_sandbox::physics::simple_rules::*;
use em_physics_sandbox::visualization::create_rotation_video;
use glam::Vec3;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  QUICK 3D VIDEO: EMERGENT DIMENSIONAL STRUCTURE              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Smaller graph for speed
    println!("Creating graph structure...");
    let mut graph = Hypergraph::new_random(50, 100);

    println!("  Initial: {} nodes, {} edges", graph.nodes.len(), graph.edges.len());

    // Evolve with mixed rules (fewer steps)
    println!("Evolving graph...");
    for step in 0..15 {
        if step % 3 == 0 {
            rule_edge_expand(&mut graph);
        }
        if step % 5 == 0 {
            rule_triangle_merge(&mut graph);
        }
        if step % 7 == 0 {
            rule_pair_creation(&mut graph);
        }

        if step % 5 == 0 {
            println!("  Step {}: {} nodes, {} edges", step, graph.nodes.len(), graph.edges.len());
        }
    }

    println!("  Final: {} nodes, {} edges\n", graph.nodes.len(), graph.edges.len());

    // Faster embedding
    println!("Embedding graph into 3D (faster settings)...");
    let embedding = GraphEmbedding::from_hypergraph(&graph, 200);  // 200 iterations instead of 1000
    println!("  Embedding energy: {:.2}\n", embedding.energy);

    // Smaller grid
    println!("Projecting to adaptive automata...");
    let mut automata = embedding.to_adaptive_automata(30, 30, 30);  // 30^3 instead of 50^3

    println!("  Grid: {}x{}x{} = {} cells", automata.nx, automata.ny, automata.nz, automata.cells.len());
    println!("  Avg dimension: {:.2}", automata.avg_dimension);
    println!("  Min dimension: {:.2}", automata.min_dimension);
    println!("  Max dimension: {:.2}\n", automata.max_dimension);

    // Add dimensional defects for visualization
    println!("Creating dimensional defects...");
    automata.create_particle_defect(Vec3::new(10.0, 15.0, 15.0), 2.0);
    automata.create_string_defect(
        Vec3::new(15.0, 5.0, 15.0),
        Vec3::new(15.0, 25.0, 15.0),
        1.5
    );
    automata.create_membrane_defect(
        Vec3::new(22.0, 15.0, 15.0),
        Vec3::new(1.0, 0.0, 0.0),
        6.0,
        1.2
    );
    automata.create_4d_bubble(Vec3::new(25.0, 25.0, 15.0), 4.0);

    println!("  Created: point, string, membrane, and 4D bubble defects\n");

    // Shorter video
    println!("═══════════════════════════════════════════════════════════════");
    println!("Generating video frames...");
    println!("═══════════════════════════════════════════════════════════════\n");

    let duration = 5.0;  // 5 seconds instead of 10
    let fps = 24;  // 24fps instead of 30 (cinematic)
    let width = 960;  // 960x540 instead of 1280x720
    let height = 540;

    println!("Settings:");
    println!("  Duration: {} seconds", duration);
    println!("  FPS: {}", fps);
    println!("  Resolution: {}x{}", width, height);
    println!("  Total frames: {}\\n", (duration * fps as f32) as usize);

    println!("Rendering frames...");
    let video_sequence = create_rotation_video(&automata, duration, fps, width, height);

    println!("  Generated {} frames\n", video_sequence.frames.len());

    // Save frames
    println!("═══════════════════════════════════════════════════════════════");
    println!("Saving frames...");
    println!("═══════════════════════════════════════════════════════════════\n");

    let output_dir = "video_frames_quick";
    match video_sequence.save_frames(output_dir) {
        Ok(()) => {
            println!("\n✓ Success! Frames saved to {}/ directory\n", output_dir);
        }
        Err(e) => {
            eprintln!("Error saving frames: {}", e);
            return;
        }
    }

    // Instructions
    println!("═══════════════════════════════════════════════════════════════");
    println!("CREATE VIDEO");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("To create the MP4 video, run:\n");
    println!("  ffmpeg -r {} -i {}/frame_%05d.png -c:v libx264 -pix_fmt yuv420p -crf 20 dimensional_rotation_quick.mp4\n",
             fps, output_dir);

    println!("Video features:");
    println!("  • 3D rotating camera");
    println!("  • Color-coded dimensions");
    println!("  • Dimensional defects visible\n");

    println!("Done! 🎬");
}
