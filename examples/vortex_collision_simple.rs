//! Simple Vortex Collision - Stable Version
//!
//! Two vortices placed close together to interact and merge.
//! Lower intensity than full turbulent collision for numerical stability.
//!
//! Run with: cargo run --example vortex_collision_simple --release

use em_physics_sandbox::physics::Fluid2D;
use em_physics_sandbox::visualization::slice2d::colormaps;
use glam::Vec2;
use std::fs;
use std::process::Command;

fn main() {
    println!("Simple Vortex Collision");
    println!("========================\n");

    // Moderate resolution
    let nx = 256;
    let ny = 256;
    let spacing = 0.0025; // 2.5mm cells
    let viscosity = 2e-5; // Moderate viscosity for stability
    let dt = 0.0005; // 0.5ms timestep

    println!("Simulation parameters:");
    println!("  Resolution: {}×{} ({} cells)", nx, ny, nx * ny);
    println!("  Domain: {:.2}×{:.2} m", nx as f32 * spacing, ny as f32 * spacing);
    println!("  Viscosity: {:.2e} m²/s", viscosity);
    println!("  Time step: {:.2e} s", dt);
    println!("  Reynolds number: ~500 (moderate)\n");

    let mut fluid = Fluid2D::new(nx, ny, spacing, viscosity, dt);

    println!("Setting up vortex pair...");

    // Domain center
    let domain_center = Vec2::new(
        nx as f32 * spacing * 0.5,
        ny as f32 * spacing * 0.5,
    );

    // Two vortices closer together for interaction
    let vortex1_pos = Vec2::new(
        domain_center.x - 0.08,  // 8cm left of center
        domain_center.y,
    );

    let vortex2_pos = Vec2::new(
        domain_center.x + 0.08,  // 8cm right of center
        domain_center.y,
    );

    let circulation = 2.0; // Moderate strength
    let core_radius = 0.04; // 4cm cores

    fluid.add_vortex(vortex1_pos, circulation, core_radius);
    fluid.add_vortex(vortex2_pos, -circulation, core_radius);

    println!("  Vortex 1: pos=({:.2}, {:.2}), Γ={:.1}",
             vortex1_pos.x, vortex1_pos.y, circulation);
    println!("  Vortex 2: pos=({:.2}, {:.2}), Γ={:.1}",
             vortex2_pos.x, vortex2_pos.y, -circulation);
    println!("  Core radius: {:.2} m", core_radius);
    println!("  Separation: {:.2} m\n", (vortex2_pos - vortex1_pos).length());

    // Create output directories
    fs::create_dir_all("output").ok();
    fs::create_dir_all("temp_frames").ok();

    // Simulation parameters
    let total_frames = 300;
    let steps_per_frame = 5;
    let fps = 30;

    println!("Animation settings:");
    println!("  Frames: {}", total_frames);
    println!("  Steps per frame: {}", steps_per_frame);
    println!("  Total steps: {}", total_frames * steps_per_frame);
    println!("  Physical time: {:.2} s\n", total_frames as f32 * steps_per_frame as f32 * dt);

    println!("Running simulation...");
    println!("(Watch for: vortex pairing, orbital motion, mutual interaction)\n");

    let mut max_vort_ever = 0.0_f32;

    for frame in 0..total_frames {
        // Advance simulation
        for _ in 0..steps_per_frame {
            fluid.step();
        }

        // Track maximum vorticity
        let max_vort = fluid.max_vorticity();
        max_vort_ever = max_vort_ever.max(max_vort);

        // Export frame
        export_vorticity_frame(&fluid, frame);

        if frame % 30 == 0 || frame == total_frames - 1 {
            let progress = 100.0 * (frame + 1) as f32 / total_frames as f32;
            let energy = fluid.kinetic_energy();

            println!("  Frame {:3}/{} ({:3.0}%) | E: {:.2e} J | ω_max: {:.1e} s⁻¹",
                     frame + 1, total_frames, progress, energy, max_vort);
        }
    }

    println!("\n✓ Simulation complete!");
    println!("  Max vorticity reached: {:.2e} s⁻¹", max_vort_ever);
    println!("  Generated {} frames\n", total_frames);

    // Encode video
    println!("Encoding video...");

    let status = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-framerate", &fps.to_string(),
            "-i", "temp_frames/vortex_%04d.png",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-crf", "18",
            "-preset", "medium",
            "output/vortex_collision_simple.mp4",
        ])
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("✓ Video encoding complete!\n");
            fs::remove_dir_all("temp_frames").ok();

            if let Ok(metadata) = fs::metadata("output/vortex_collision_simple.mp4") {
                println!("File size: {:.1} MB", metadata.len() as f64 / 1_048_576.0);
            }

            println!("\n✓ Video: output/vortex_collision_simple.mp4");
            println!("\nWhat to see:");
            println!("  • Two counter-rotating vortices");
            println!("  • Vortex pairing and orbital motion");
            println!("  • Mutual advection and interaction");
            println!("  • Gradual viscous decay");
            println!("  • Red/Blue diverging colormap");
        }
        Ok(_) => {
            eprintln!("✗ ffmpeg encoding failed");
            eprintln!("Frames saved in temp_frames/");
        }
        Err(e) => {
            eprintln!("✗ ffmpeg error: {}", e);
            eprintln!("Frames saved in temp_frames/");
        }
    }
}

/// Export vorticity field with enhanced contrast
fn export_vorticity_frame(fluid: &Fluid2D, frame: usize) {
    let nx = fluid.nx;
    let ny = fluid.ny;

    let max_vort = fluid.max_vorticity().max(1.0);

    let mut rgba = Vec::with_capacity(nx * ny * 4);

    for j in (0..ny).rev() {
        for i in 0..nx {
            let idx = fluid.index(i, j);
            let vort = fluid.vorticity[idx];

            // Symmetric scale around zero
            let normalized = (vort / max_vort).clamp(-1.0, 1.0);
            let t = (normalized * 0.5 + 0.5).clamp(0.0, 1.0);

            let rgb = colormaps::red_blue(t);

            rgba.push(rgb[0]);
            rgba.push(rgb[1]);
            rgba.push(rgb[2]);
            rgba.push(255);
        }
    }

    image::save_buffer(
        format!("temp_frames/vortex_{:04}.png", frame),
        &rgba,
        nx as u32,
        ny as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
