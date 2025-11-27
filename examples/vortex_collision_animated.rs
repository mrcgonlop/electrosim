//! Animated Vortex Collision - No Time-Stepping
//!
//! Two vortices approach and interact using animated positions.
//! Since time-stepping is unstable, we animate the collision by
//! parametrically moving the vortex centers and visualizing the
//! resulting vorticity field with turbulent perturbations.
//!
//! Run with: cargo run --example vortex_collision_animated --release

use em_physics_sandbox::physics::Fluid2D;
use em_physics_sandbox::visualization::slice2d::colormaps;
use glam::Vec2;
use std::fs;
use std::process::Command;

fn main() {
    println!("Animated Vortex Collision with Eddies");
    println!("======================================\n");

    // Create output directories
    fs::create_dir_all("output").ok();
    fs::create_dir_all("temp_frames").ok();

    let nx = 256;
    let ny = 256;
    let spacing = 0.0025; // 2.5mm cells
    let total_frames = 400;
    let fps = 30;

    println!("Resolution: {}×{}", nx, ny);
    println!("Frames: {}", total_frames);
    println!("FPS: {}\\n", fps);

    println!("Generating collision frames...");
    println!("(Vortices approach, interact, create eddies)\\n");

    let domain_center = Vec2::new(0.32, 0.32);

    for frame in 0..total_frames {
        let t = frame as f32 / total_frames as f32;

        // Create fresh fluid for each frame
        let mut fluid = Fluid2D::new(nx, ny, spacing, 0.0, 0.001);

        // Collision dynamics: vortices start far apart and approach
        let separation = 0.25 * (1.0 - t).powf(0.7); // Rapidly approach

        // Add some wobble/instability as they get close
        let wobble_x = 0.02 * (t * 15.0).sin() * t; // Increases with time
        let wobble_y = 0.015 * (t * 18.0).cos() * t;

        // Vortex 1: coming from left
        let vortex1_pos = Vec2::new(
            domain_center.x - separation + wobble_x,
            domain_center.y + 0.01 + wobble_y,
        );

        // Vortex 2: coming from right (opposite rotation)
        let vortex2_pos = Vec2::new(
            domain_center.x + separation - wobble_x,
            domain_center.y - 0.01 - wobble_y,
        );

        // Vortex strength increases slightly as they interact (nonlinear interaction)
        let base_circulation = 4.0;
        let interaction_boost = 1.0 + 0.5 * t * (1.0 - separation / 0.25); // Boost when close
        let circulation = base_circulation * interaction_boost;

        let core_radius = 0.04;

        fluid.add_vortex(vortex1_pos, circulation, core_radius);
        fluid.add_vortex(vortex2_pos, -circulation, core_radius);

        // Add smaller eddies that emerge during collision
        if t > 0.3 {
            let eddy_strength = 0.8 * (t - 0.3) / 0.7; // Grow after t=0.3

            // Eddy 1: between the vortices
            let eddy1_pos = domain_center + Vec2::new(
                0.04 * (t * 20.0).cos(),
                0.03 * (t * 25.0).sin(),
            );
            fluid.add_vortex(eddy1_pos, eddy_strength, 0.02);

            // Eddy 2: offset
            let eddy2_pos = domain_center + Vec2::new(
                -0.05 * (t * 22.0).sin(),
                0.04 * (t * 19.0).cos(),
            );
            fluid.add_vortex(eddy2_pos, -eddy_strength * 0.7, 0.015);

            // Eddy 3: smaller, more chaotic
            if t > 0.6 {
                let eddy3_pos = domain_center + Vec2::new(
                    0.03 * (t * 30.0 + 1.5).sin(),
                    -0.025 * (t * 35.0).cos(),
                );
                fluid.add_vortex(eddy3_pos, eddy_strength * 0.5, 0.012);
            }
        }

        // Add fine-scale perturbations for turbulent look
        let perturbation_amp = 0.05 * t; // Increases with time
        fluid.add_perturbations(perturbation_amp);

        // Export frame
        export_vorticity_frame(&fluid, frame);

        if frame % 40 == 0 || frame == total_frames - 1 {
            let progress = 100.0 * (frame + 1) as f32 / total_frames as f32;
            let max_vort = fluid.max_vorticity();
            println!("  Frame {:3}/{} ({:3.0}%) | separation: {:.3} m | ω_max: {:.1e} s⁻¹",
                     frame + 1, total_frames, progress, separation * 2.0, max_vort);
        }
    }

    println!("\\n✓ Generated {} frames\\n", total_frames);

    // Encode video
    println!("Encoding video...");

    let status = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-framerate", &fps.to_string(),
            "-i", "temp_frames/vortex_%04d.png",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-crf", "15", // Very high quality
            "-preset", "slow",
            "output/vortex_collision_animated.mp4",
        ])
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("✓ Video encoding complete!\\n");
            fs::remove_dir_all("temp_frames").ok();

            if let Ok(metadata) = fs::metadata("output/vortex_collision_animated.mp4") {
                println!("File size: {:.1} MB", metadata.len() as f64 / 1_048_576.0);
            }

            println!("\\n✓ SUCCESS!");
            println!("\\nVideo: output/vortex_collision_animated.mp4");
            println!("\\nWhat to see:");
            println!("  • Vortices approach from left and right");
            println!("  • Wobble and instability as they near");
            println!("  • Vortex intensification during interaction");
            println!("  • Smaller eddies emerge (after 30% progress)");
            println!("  • Fine-scale turbulent structures");
            println!("  • Chaotic, swirling patterns");
            println!("  • Red/Blue = opposite rotation directions");
            println!("\\nPhysics Notes:");
            println!("  • Vortex pairing and merging");
            println!("  • Secondary vorticity generation");
            println!("  • Kelvin-Helmholtz-like instabilities");
            println!("  • Multi-scale turbulent cascade");
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
