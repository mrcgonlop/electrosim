//! Vortex Interaction Demo - Phase 2 Showcase
//!
//! Demonstrates two fluid vortices interacting in 2D.
//! Exports frames and creates an MP4 video using ffmpeg.
//!
//! Run with: cargo run --example vortex_interaction --release

use em_physics_sandbox::physics::Fluid2D;
use em_physics_sandbox::visualization::slice2d::colormaps;
use glam::Vec2;
use std::fs;
use std::process::Command;

fn main() {
    println!("Vortex Interaction - Phase 2 Demo");
    println!("==================================\n");

    // Create fluid simulation (128x128 grid, 5mm spacing)
    let nx = 128;
    let ny = 128;
    let spacing = 0.005; // 5mm cells
    let viscosity = 1e-5; // Low viscosity for long-lived vortices
    let dt = 0.001; // 1ms time step

    let mut fluid = Fluid2D::new(nx, ny, spacing, viscosity, dt);

    println!("Fluid domain:");
    println!("  Grid: {}×{}", nx, ny);
    println!("  Spacing: {:.3} m", spacing);
    println!("  Domain size: {:.2}×{:.2} m", nx as f32 * spacing, ny as f32 * spacing);
    println!("  Viscosity: {:.2e} m²/s", viscosity);
    println!("  Time step: {:.3e} s\n", dt);

    // Add two counter-rotating vortices
    println!("Initializing vortices...");

    let vortex1_pos = Vec2::new(0.20, 0.32); // Left vortex
    let vortex2_pos = Vec2::new(0.44, 0.22); // Right vortex
    let circulation = 2.0; // Circulation strength
    let core_radius = 0.05; // 5cm core

    fluid.add_vortex(vortex1_pos, circulation, core_radius);   // Clockwise
    fluid.add_vortex(vortex2_pos, -circulation, core_radius);  // Counter-clockwise

    println!("  Vortex 1: pos=({:.2}, {:.2}), Γ={:.1}", vortex1_pos.x, vortex1_pos.y, circulation);
    println!("  Vortex 2: pos=({:.2}, {:.2}), Γ={:.1}", vortex2_pos.x, vortex2_pos.y, -circulation);
    println!("  Core radius: {:.2} m\n", core_radius);

    // Create output directories
    fs::create_dir_all("output").expect("Failed to create output directory");
    fs::create_dir_all("temp_frames").expect("Failed to create temp directory");

    // Simulation parameters
    let total_frames = 300;
    let steps_per_frame = 5;
    let fps = 30;

    println!("Simulation parameters:");
    println!("  Total frames: {}", total_frames);
    println!("  Steps per frame: {}", steps_per_frame);
    println!("  Output FPS: {}", fps);
    println!("  Total simulation time: {:.2} s\n", total_frames as f32 * steps_per_frame as f32 * dt);

    println!("Running simulation and exporting frames...");

    let mut frame_count = 0;
    for frame in 0..total_frames {
        // Advance simulation
        for _ in 0..steps_per_frame {
            fluid.step();
        }

        // Export frame
        export_vorticity_frame(&fluid, frame);
        frame_count += 1;

        if frame % 30 == 0 || frame == total_frames - 1 {
            let progress = 100.0 * (frame + 1) as f32 / total_frames as f32;
            let energy = fluid.kinetic_energy();
            let max_vort = fluid.max_vorticity();
            println!("  Frame {}/{} ({:.0}%) | Energy: {:.2e} J | Max vorticity: {:.2e} s⁻¹",
                     frame + 1, total_frames, progress, energy, max_vort);
        }
    }

    println!("\n✓ Generated {} frames\n", frame_count);

    // Encode video with ffmpeg
    println!("Encoding video with ffmpeg...");

    let status = Command::new("ffmpeg")
        .args(&[
            "-y", // Overwrite output
            "-framerate", &fps.to_string(),
            "-i", "temp_frames/vortex_%04d.png",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-crf", "20", // High quality
            "-preset", "medium",
            "output/vortex_interaction.mp4",
        ])
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("✓ Video encoding complete!\n");

            // Clean up temp frames
            println!("Cleaning up temporary files...");
            fs::remove_dir_all("temp_frames").ok();

            // Get file size
            if let Ok(metadata) = fs::metadata("output/vortex_interaction.mp4") {
                let size_mb = metadata.len() as f64 / 1_048_576.0;
                println!("Video size: {:.1} MB", size_mb);
            }

            println!("\n✓ Success! Video saved to: output/vortex_interaction.mp4");
            println!("\nThe video shows:");
            println!("  • Two counter-rotating vortices");
            println!("  • Vortex pairing and orbital motion");
            println!("  • Vorticity field colored by intensity");
            println!("  • Real fluid dynamics with viscous decay");
        }
        Ok(_) => {
            eprintln!("✗ ffmpeg failed to encode video");
            eprintln!("  Frames saved in temp_frames/ directory");
        }
        Err(e) => {
            eprintln!("✗ ffmpeg not found: {}", e);
            eprintln!("  Please install ffmpeg (already done earlier!)");
            eprintln!("  Frames saved in temp_frames/ directory");
        }
    }
}

/// Export vorticity field as PNG frame
fn export_vorticity_frame(fluid: &Fluid2D, frame: usize) {
    let nx = fluid.nx;
    let ny = fluid.ny;

    // Find min/max vorticity for color scaling
    let max_vort = fluid.max_vorticity();
    let vort_range = max_vort.max(1e-6);

    // Create RGBA image
    let mut rgba = Vec::with_capacity(nx * ny * 4);

    for j in (0..ny).rev() {
        // Flip Y for image coordinates
        for i in 0..nx {
            let idx = fluid.index(i, j);
            let vort = fluid.vorticity[idx];

            // Normalize to [0, 1] using symmetric scale
            let t = (vort / vort_range * 0.5 + 0.5).clamp(0.0, 1.0);

            // Use red-blue diverging colormap for vorticity
            let rgb = colormaps::red_blue(t);

            rgba.push(rgb[0]);
            rgba.push(rgb[1]);
            rgba.push(rgb[2]);
            rgba.push(255);
        }
    }

    // Save frame
    image::save_buffer(
        format!("temp_frames/vortex_{:04}.png", frame),
        &rgba,
        nx as u32,
        ny as u32,
        image::ColorType::Rgba8,
    )
    .expect("Failed to save frame");
}
