//! Simple interactive 2D field viewer
//!
//! A minimal working example of real-time field visualization.
//! Uses a simplified approach that's easier to understand and debug.
//!
//! Run with: cargo run --example simple_viewer --release

use em_physics_sandbox::{
    physics::{EMTheory, MaxwellTheory},
    simulation::VoxelGrid,
    visualization::{FieldComponent, FieldSlice, SlicePlane, slice2d::colormaps},
};
use std::time::Instant;

fn main() {
    env_logger::init();

    println!("EM Physics Sandbox - Simple Viewer");
    println!("===================================\n");
    println!("Running 3D Maxwell FDTD simulation with real-time text visualization");
    println!("This is a text-based viewer - GUI viewer requires additional setup\n");

    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(32, 32, 32, 0.01);

    // Initialize with Gaussian pulse
    maxwell.initialize(&mut grid);

    let dt = 1e-12;
    let start = Instant::now();

    println!("Simulating... (Press Ctrl+C to stop)\n");

    for frame in 0.. {
        // Update simulation
        for _ in 0..10 {
            maxwell.update_fields(&mut grid, dt);
        }

        // Display stats every 10 frames
        if frame % 10 == 0 {
            let slice = FieldSlice::from_grid(
                &grid,
                SlicePlane::XY,
                0.5,
                FieldComponent::EMagnitude,
            );

            let energy = maxwell.total_energy(&grid);
            let elapsed = start.elapsed().as_secs_f32();
            let fps = frame as f32 / elapsed;

            println!("Frame {:4} | Time: {:.2e} s | Energy: {:.2e} J | E_max: {:.2e} V/m | FPS: {:.1}",
                     frame,
                     frame as f32 * dt * 10.0,
                     energy,
                     slice.max_value,
                     fps);
        }

        // Stop after 100 frames for demo
        if frame >= 100 {
            break;
        }
    }

    println!("\nSimulation complete!");
    println!("\nFor visual output, run:");
    println!("  cargo run --example export_frames --release");
    println!("  cargo run --example export_video --release  (requires ffmpeg)");
}
