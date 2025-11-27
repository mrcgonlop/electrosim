//! Turbulent Vortex Collision - Chaotic Interaction Demo
//!
//! Two strong vortices collide head-on with added perturbations
//! to trigger turbulence, eddies, and chaotic behavior.
//!
//! Run with: cargo run --example vortex_collision --release

use em_physics_sandbox::physics::Fluid2D;
use em_physics_sandbox::visualization::slice2d::colormaps;
use glam::Vec2;
use std::fs;
use std::process::Command;

fn main() {
    println!("Turbulent Vortex Collision");
    println!("==========================\n");

    // High resolution for capturing eddies
    let nx = 256;
    let ny = 256;
    let spacing = 0.0025; // 2.5mm cells for fine detail
    let viscosity = 1e-5; // Low viscosity → more turbulence
    let dt = 0.0005; // Small timestep for stability

    println!("Simulation parameters:");
    println!("  Resolution: {}×{} ({} cells)", nx, ny, nx * ny);
    println!("  Domain: {:.2}×{:.2} m", nx as f32 * spacing, ny as f32 * spacing);
    println!("  Viscosity: {:.2e} m²/s (low → turbulent)", viscosity);
    println!("  Time step: {:.2e} s (small → stable)", dt);
    println!("  Reynolds number: ~1000 (turbulent regime)\n");

    // Initialize fluid
    let mut fluid = Fluid2D::new(nx, ny, spacing, viscosity, dt);

    println!("Setting up collision scenario...");

    // Two strong vortices moving toward each other
    let domain_center = Vec2::new(
        nx as f32 * spacing * 0.5,
        ny as f32 * spacing * 0.5,
    );

    // Vortex 1: Coming from left
    let vortex1_pos = Vec2::new(
        domain_center.x - 0.15,
        domain_center.y + 0.02,
    );

    // Vortex 2: Coming from right (opposite rotation)
    let vortex2_pos = Vec2::new(
        domain_center.x + 0.15,
        domain_center.y - 0.02,
    );

    let circulation = 0.5; // Strong vortices (reduced from 8.0)
    let core_radius = 0.01; // 4cm cores (slightly larger)

    fluid.add_vortex(vortex1_pos, circulation, core_radius);
    fluid.add_vortex(vortex2_pos, -circulation, core_radius);

    // Add velocity to make them collide
    let collision_speed = 0.3; // m/s (reduced from 0.5)
    for j in 0..ny {
        for i in 0..nx {
            let idx = fluid.index(i, j);
            let pos = Vec2::new(i as f32 * spacing, j as f32 * spacing);

            // Left half moves right, right half moves left
            if pos.x < domain_center.x {
                fluid.velocity[idx].x += collision_speed;
            } else {
                fluid.velocity[idx].x -= collision_speed;
            }
        }
    }

    // Add perturbations to seed turbulence
    fluid.add_perturbations(0.05); // Reduced from 0.1

    println!("  Vortex 1: pos=({:.2}, {:.2}), Γ={:.1}, moving RIGHT",
             vortex1_pos.x, vortex1_pos.y, circulation);
    println!("  Vortex 2: pos=({:.2}, {:.2}), Γ={:.1}, moving LEFT",
             vortex2_pos.x, vortex2_pos.y, -circulation);
    println!("  Collision speed: {:.1} m/s", collision_speed);
    println!("  Perturbations: Added for turbulence onset\n");

    // Create output directories
    fs::create_dir_all("output").ok();
    fs::create_dir_all("temp_frames").ok();

    // Simulation parameters
    let total_frames = 800;
    let steps_per_frame = 10;
    let fps = 30;

    println!("Animation settings:");
    println!("  Frames: {}", total_frames);
    println!("  Steps per frame: {}", steps_per_frame);
    println!("  Total steps: {}", total_frames * steps_per_frame);
    println!("  Physical time: {:.2} s", total_frames as f32 * steps_per_frame as f32 * dt);
    println!("  Output: {} FPS\n", fps);

    println!("Running turbulent simulation...");
    println!("(Watch for: vortex collision, eddy formation, cascade to smaller scales)\n");

    let mut max_vort_ever = 0.0_f32;

    for frame in 0..total_frames {
        // Advance simulation
        for _ in 0..steps_per_frame {
            fluid.step();
        }

        // Track maximum vorticity (indicates turbulence intensity)
        let max_vort = fluid.max_vorticity();
        max_vort_ever = max_vort_ever.max(max_vort);

        // Export frame
        export_vorticity_frame(&fluid, frame);

        if frame % 40 == 0 || frame == total_frames - 1 {
            let progress = 100.0 * (frame + 1) as f32 / total_frames as f32;
            let energy = fluid.kinetic_energy();
            let enstrophy = fluid.enstrophy();

            println!("  Frame {:3}/{} ({:3.0}%) | E: {:.2e} J | Ω: {:.2e} | ω_max: {:.1e} s⁻¹",
                     frame + 1, total_frames, progress, energy, enstrophy, max_vort);
        }
    }

    println!("\n✓ Simulation complete!");
    println!("  Max vorticity reached: {:.2e} s⁻¹", max_vort_ever);
    println!("  Generated {} frames\n", total_frames);

    // Encode video
    println!("Encoding high-quality video...");

    let status = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-framerate", &fps.to_string(),
            "-i", "temp_frames/vortex_%04d.png",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-crf", "15", // Very high quality
            "-preset", "slow",
            "output/vortex_collision.mp4",
        ])
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("✓ Video encoding complete!\n");
            fs::remove_dir_all("temp_frames").ok();

            if let Ok(metadata) = fs::metadata("output/vortex_collision.mp4") {
                println!("File size: {:.1} MB", metadata.len() as f64 / 1_048_576.0);
            }

            println!("\n✓ SUCCESS!");
            println!("\nVideo: output/vortex_collision.mp4");
            println!("\nWhat to look for:");
            println!("  • Initial: Two distinct vortices approaching");
            println!("  • Collision: Vortices interact, intensify");
            println!("  • Turbulence: Small-scale eddies emerge");
            println!("  • Cascade: Energy transfers to smaller scales");
            println!("  • Chaos: Unpredictable, complex flow patterns");
            println!("  • Red/Blue: Opposite rotation directions");
            println!("\nPhysics:");
            println!("  • Kelvin-Helmholtz instability at shear layers");
            println!("  • Vortex stretching and tilting");
            println!("  • Enstrophy cascade (energy → smaller eddies)");
            println!("  • Re ~ 1000 (transitional turbulence)");
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

/// Export vorticity field with enhanced contrast for turbulence
fn export_vorticity_frame(fluid: &Fluid2D, frame: usize) {
    let nx = fluid.nx;
    let ny = fluid.ny;

    let max_vort = fluid.max_vorticity().max(1.0);

    let mut rgba = Vec::with_capacity(nx * ny * 4);

    for j in (0..ny).rev() {
        for i in 0..nx {
            let idx = fluid.index(i, j);
            let vort = fluid.vorticity[idx];

            // Enhanced contrast for viewing fine structure
            let normalized = (vort / max_vort).clamp(-1.0, 1.0);

            // Symmetric scale around zero
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
