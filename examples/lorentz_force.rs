//! Lorentz Force - Charged Particle in Vorticity Field
//!
//! Demonstrates the fluid analog of the Lorentz force F = q(E + v×B).
//! A charged particle (localized charge blob) moves through a background
//! vorticity field (B-field analog) and experiences a deflecting force.
//!
//! Physics:
//! - Background vorticity ω ≈ Magnetic field B
//! - Charged fluid blob with velocity v
//! - Fluid advection creates v×ω force (Lorentz force analog)
//! - Particle should curve in circular/helical path
//!
//! Run with: cargo run --example lorentz_force --release

use em_physics_sandbox::physics::Fluid2D;
use em_physics_sandbox::visualization::slice2d::colormaps;
use glam::Vec2;
use std::fs;
use std::process::Command;

fn main() {
    println!("Lorentz Force - Fluid EM Analog");
    println!("================================\n");

    // Simulation parameters
    let nx = 256;
    let ny = 256;
    let spacing = 0.0025; // 2.5mm cells
    let viscosity = 0.0; // Zero viscosity - inviscid flow to preserve motion
    let dt = 0.0005; // 0.5ms timestep

    println!("Simulation parameters:");
    println!("  Resolution: {}×{}", nx, ny);
    println!("  Domain: {:.2}×{:.2} m", nx as f32 * spacing, ny as f32 * spacing);
    println!("  Viscosity: {:.2e} m²/s", viscosity);
    println!("  Time step: {:.2e} s\n", dt);

    let mut fluid = Fluid2D::new(nx, ny, spacing, viscosity, dt);

    // Domain center
    let domain_center = Vec2::new(
        nx as f32 * spacing * 0.5,
        ny as f32 * spacing * 0.5,
    );

    // Background vorticity field (uniform "magnetic field")
    let vortex_circulation = 8.0; // Strong vorticity
    let vortex_radius = 0.20; // Large radius for uniform region

    println!("Background vorticity field (B-field analog):");
    println!("  Center: ({:.2}, {:.2}) m", domain_center.x, domain_center.y);
    println!("  Circulation: {:.1} m²/s", vortex_circulation);
    println!("  Radius: {:.2} m\n", vortex_radius);

    // Create background vorticity
    fluid.add_vortex(domain_center, vortex_circulation, vortex_radius);

    // Charged particle parameters
    let particle_charge = 1e-6; // 1 μC
    let particle_radius = 0.01; // 1cm blob
    let particle_initial_pos = Vec2::new(
        domain_center.x - 0.15, // Start left of center
        domain_center.y,
    );
    let particle_initial_velocity = Vec2::new(0.8, 0.0); // Launch rightward at 0.8 m/s

    println!("Charged particle:");
    println!("  Initial position: ({:.2}, {:.2}) m", particle_initial_pos.x, particle_initial_pos.y);
    println!("  Initial velocity: ({:.2}, {:.2}) m/s", particle_initial_velocity.x, particle_initial_velocity.y);
    println!("  Charge: {:.2e} C", particle_charge);
    println!("  Radius: {:.2} m\n", particle_radius);

    // Add charged particle as a localized charge blob
    // The charge will be advected by the fluid
    for j in 0..ny {
        for i in 0..nx {
            let pos = Vec2::new(i as f32 * spacing, j as f32 * spacing);
            let r = (pos - particle_initial_pos).length();

            if r < particle_radius {
                let idx = fluid.index(i, j);

                // Gaussian charge distribution
                let charge_strength = particle_charge * (-r*r / (particle_radius*particle_radius)).exp();
                fluid.charge_density[idx] = charge_strength;

                // Give the particle initial velocity
                fluid.velocity[idx] += particle_initial_velocity;
            }
        }
    }

    // Create output directories
    fs::create_dir_all("output").ok();
    fs::create_dir_all("temp_frames").ok();

    // Animation parameters
    let total_frames = 400;
    let steps_per_frame = 3;
    let fps = 30;

    println!("Animation settings:");
    println!("  Frames: {}", total_frames);
    println!("  Steps per frame: {}", steps_per_frame);
    println!("  Output: {} FPS", fps);
    println!("  Total time: {:.2} s\n", total_frames as f32 * steps_per_frame as f32 * dt);

    println!("Running simulation...");
    println!("(Watch particle trajectory - should curve due to Lorentz force)\n");

    // Track particle position
    let mut trajectory = Vec::new();

    for frame in 0..total_frames {
        // Advance simulation
        for _ in 0..steps_per_frame {
            fluid.step();
        }

        // Find particle center (peak of charge density)
        let particle_pos = find_charge_peak(&fluid);
        trajectory.push(particle_pos);

        // Export visualization frame
        export_lorentz_frame(&fluid, &trajectory, frame);

        if frame % 40 == 0 || frame == total_frames - 1 {
            let progress = 100.0 * (frame + 1) as f32 / total_frames as f32;
            let max_vort = fluid.max_vorticity();
            let energy = fluid.kinetic_energy();

            println!("  Frame {:3}/{} ({:3.0}%) | Particle: ({:.3}, {:.3}) m | ω_max: {:.1e} s⁻¹ | E: {:.2e} J",
                     frame + 1, total_frames, progress, particle_pos.x, particle_pos.y, max_vort, energy);
        }
    }

    // Analyze trajectory
    println!("\n✓ Simulation complete!");
    println!("  Particle traveled {} steps", trajectory.len());

    let initial_pos = trajectory[0];
    let final_pos = trajectory[trajectory.len() - 1];
    let displacement = final_pos - initial_pos;

    println!("  Initial position: ({:.3}, {:.3}) m", initial_pos.x, initial_pos.y);
    println!("  Final position:   ({:.3}, {:.3}) m", final_pos.x, final_pos.y);
    println!("  Net displacement: ({:.3}, {:.3}) m", displacement.x, displacement.y);
    println!("  Generated {} frames\n", total_frames);

    // Encode video
    println!("Encoding video...");

    let status = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-framerate", &fps.to_string(),
            "-i", "temp_frames/lorentz_%04d.png",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-crf", "18",
            "-preset", "medium",
            "output/lorentz_force.mp4",
        ])
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("✓ Video encoding complete!\n");
            fs::remove_dir_all("temp_frames").ok();

            if let Ok(metadata) = fs::metadata("output/lorentz_force.mp4") {
                println!("File size: {:.1} MB", metadata.len() as f64 / 1_048_576.0);
            }

            println!("\n✓ SUCCESS!");
            println!("\nVideo: output/lorentz_force.mp4");
            println!("\nWhat to see:");
            println!("  • Background vorticity field (blue/red)");
            println!("  • Green charged particle moving through field");
            println!("  • White trail showing particle trajectory");
            println!("  • Particle curves due to v×ω Lorentz-like force");
            println!("\nPhysics - Martins' Fluid EM Theory:");
            println!("  • Vorticity ω ≈ Magnetic field B");
            println!("  • Moving charge in fluid experiences v×ω force");
            println!("  • Particle trajectory curves (cyclotron motion)");
            println!("  • F = q(v × B) → fluid advection in vorticity field");
            println!("\nReal EM Analog:");
            println!("  • Charged particle in magnetic field");
            println!("  • Cathode ray tube deflection");
            println!("  • Cyclotron/synchrotron motion");
            println!("  • Radius of curvature r = mv/(qB)");
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

/// Find the center of the charged particle (peak charge density)
fn find_charge_peak(fluid: &Fluid2D) -> Vec2 {
    let mut max_charge = 0.0_f32;
    let mut peak_pos = Vec2::ZERO;

    for j in 0..fluid.ny {
        for i in 0..fluid.nx {
            let idx = fluid.index(i, j);
            let charge = fluid.charge_density[idx];

            if charge > max_charge {
                max_charge = charge;
                peak_pos = Vec2::new(i as f32 * fluid.spacing, j as f32 * fluid.spacing);
            }
        }
    }

    peak_pos
}

/// Export visualization showing vorticity field, particle, and trajectory
fn export_lorentz_frame(fluid: &Fluid2D, trajectory: &[Vec2], frame: usize) {
    let nx = fluid.nx;
    let ny = fluid.ny;

    let max_vort = fluid.max_vorticity().max(1.0);
    let max_charge = fluid.charge_density.iter().cloned().fold(0.0_f32, f32::max);

    let mut rgba = Vec::with_capacity(nx * ny * 4);

    // Render vorticity background and particle
    for j in (0..ny).rev() {
        for i in 0..nx {
            let idx = fluid.index(i, j);
            let vort = fluid.vorticity[idx];
            let charge = fluid.charge_density[idx];

            // Vorticity colormap (blue/red for background field)
            let normalized = (vort / max_vort).clamp(-1.0, 1.0);
            let t = (normalized * 0.5 + 0.5).clamp(0.0, 1.0);

            let mut rgb = colormaps::red_blue(t);

            // Overlay charged particle in bright green
            if charge > max_charge * 0.1 {
                let charge_intensity = (charge / max_charge).clamp(0.0, 1.0);
                rgb = [
                    (50.0 * (1.0 - charge_intensity) + 0.0 * charge_intensity) as u8,
                    (50.0 * (1.0 - charge_intensity) + 255.0 * charge_intensity) as u8,
                    (50.0 * (1.0 - charge_intensity) + 0.0 * charge_intensity) as u8,
                ];
            }

            // Draw trajectory as white dots
            let pos = Vec2::new(i as f32 * fluid.spacing, j as f32 * fluid.spacing);
            for &trail_pos in trajectory.iter() {
                if (pos - trail_pos).length() < fluid.spacing * 1.5 {
                    rgb = [255, 255, 255];
                    break;
                }
            }

            rgba.push(rgb[0]);
            rgba.push(rgb[1]);
            rgba.push(rgb[2]);
            rgba.push(255);
        }
    }

    image::save_buffer(
        format!("temp_frames/lorentz_{:04}.png", frame),
        &rgba,
        nx as u32,
        ny as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
