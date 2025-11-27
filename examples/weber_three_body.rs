//! Weber Electrodynamics - Three Body Problem
//!
//! Simulates three charged particles interacting via Weber's force law:
//! F = k·q₁q₂/r² · [1 - (ṙ)²/(2c²) + r·r̈/c²]
//!
//! This demonstrates action-at-a-distance electrodynamics (no fields!).
//! The velocity and acceleration-dependent terms make Weber's theory
//! fundamentally different from Maxwell's electromagnetism.
//!
//! Run with: cargo run --example weber_three_body --release

use em_physics_sandbox::physics::{WeberParticles, Particle};
use glam::Vec2;
use std::fs;
use std::process::Command;

fn main() {
    println!("Weber Electrodynamics - Three Body Problem");
    println!("==========================================\n");

    // Physical constants
    let c = 3e8; // Speed of light (m/s)
    let dt = 1e-11; // 10 picosecond timestep (much smaller for stability)

    println!("Weber's force law:");
    println!("  F = k·q₁q₂/r² · [1 - (ṙ)²/(2c²) + r·r̈/c²]");
    println!("  • Coulomb term: k·q₁q₂/r²");
    println!("  • Velocity correction: -(ṙ)²/(2c²)");
    println!("  • Acceleration correction: r·r̈/c²\n");

    println!("Simulation parameters:");
    println!("  Speed of light: {:.2e} m/s", c);
    println!("  Time step: {:.2e} s ({:.1} ps)", dt, dt * 1e12);
    println!("  Integration: Velocity Verlet (symplectic)\n");

    // Create Weber particle system
    let mut system = WeberParticles::new(c);

    // Particle parameters
    let charge = 1e-10; // 0.1 nC (reduced for stability)
    let mass = 1e-15; // 1 femtogram (heavier for slower dynamics)

    // Initial configuration: equilateral triangle
    let radius = 0.01; // 1 cm
    let center = Vec2::new(0.0, 0.0);

    println!("Three-body configuration:");
    println!("  Arrangement: Equilateral triangle");
    println!("  Radius: {:.2} cm", radius * 100.0);
    println!("  Particle charge: {:.2e} C ({:.1} pC)", charge, charge * 1e12);
    println!("  Particle mass: {:.2e} kg ({:.1} fg)\n", mass, mass * 1e15);

    // Particle 1: Top (positive charge)
    let p1_pos = center + Vec2::new(0.0, radius);
    let p1_vel = Vec2::new(1.0, -10.0); // Smaller initial velocity
    system.add_particle(Particle::new(p1_pos, p1_vel, charge, mass));

    // Particle 2: Bottom-left (negative charge)
    let angle2 = 7.0 * std::f32::consts::PI / 6.0; // 210°
    let p2_pos = center + Vec2::new(radius * angle2.cos(), radius * angle2.sin());
    let p2_vel = Vec2::new(
        -1.0 * angle2.sin(),
        1.0 * angle2.cos(),
    );
    system.add_particle(Particle::new(p2_pos, p2_vel, -charge, mass));

    // Particle 3: Bottom-right (positive charge)
    let angle3 = -std::f32::consts::PI / 6.0; // -30°
    let p3_pos = center + Vec2::new(radius * angle3.cos(), radius * angle3.sin());
    let p3_vel = Vec2::new(
        -1.0 * angle3.sin(),
        1.0 * angle3.cos(),
    );
    system.add_particle(Particle::new(p3_pos, p3_vel, charge, mass));

    println!("Particle 1: q={:+.1e} C, pos=({:.3}, {:.3}) m",
             system.particles[0].charge, p1_pos.x, p1_pos.y);
    println!("Particle 2: q={:+.1e} C, pos=({:.3}, {:.3}) m",
             system.particles[1].charge, p2_pos.x, p2_pos.y);
    println!("Particle 3: q={:+.1e} C, pos=({:.3}, {:.3}) m\n",
             system.particles[2].charge, p3_pos.x, p3_pos.y);

    // Create output directories
    fs::create_dir_all("output").ok();
    fs::create_dir_all("temp_frames").ok();

    // Animation parameters
    let total_frames = 2000;
    let steps_per_frame = 1000; // 1000 steps per frame for finer time resolution
    let fps = 30;

    println!("Animation settings:");
    println!("  Frames: {}", total_frames);
    println!("  Steps per frame: {}", steps_per_frame);
    println!("  Total steps: {}", total_frames * steps_per_frame);
    println!("  Physical time: {:.2e} s ({:.1} ns)",
             total_frames as f32 * steps_per_frame as f32 * dt,
             total_frames as f32 * steps_per_frame as f32 * dt * 1e9);
    println!("  Output: {} FPS\n", fps);

    println!("Running Weber electrodynamics simulation...");
    println!("(Watch for complex orbital dynamics due to Weber corrections)\n");

    let e_initial = system.total_energy();

    // Track trajectories
    let mut trajectories: Vec<Vec<Vec2>> = vec![Vec::new(); 3];

    for frame in 0..total_frames {
        // Advance simulation
        for _ in 0..steps_per_frame {
            system.step(dt);
        }

        // Record positions
        for (i, particle) in system.particles.iter().enumerate() {
            trajectories[i].push(particle.position);
        }

        // Export visualization frame
        export_weber_frame(&system, &trajectories, frame);

        if frame % 60 == 0 || frame == total_frames - 1 {
            let progress = 100.0 * (frame + 1) as f32 / total_frames as f32;
            let e = system.total_energy();
            let ke = system.kinetic_energy();
            let pe = system.potential_energy();
            let e_error = (e - e_initial) / e_initial.abs();

            println!("  Frame {:3}/{} ({:3.0}%) | E: {:.2e} J | KE: {:.2e} | PE: {:.2e} | ΔE/E: {:.2e}",
                     frame + 1, total_frames, progress, e, ke, pe, e_error);
        }
    }

    let e_final = system.total_energy();
    let energy_conservation_error = ((e_final - e_initial) / e_initial.abs()).abs();

    println!("\n✓ Simulation complete!");
    println!("  Initial energy: {:.6e} J", e_initial);
    println!("  Final energy:   {:.6e} J", e_final);
    println!("  Conservation error: {:.2e} ({:.4}%)",
             energy_conservation_error,
             energy_conservation_error * 100.0);
    println!("  Generated {} frames\n", total_frames);

    // Encode video
    println!("Encoding video...");

    let status = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-framerate", &fps.to_string(),
            "-i", "temp_frames/weber_%04d.png",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-crf", "18",
            "-preset", "medium",
            "output/weber_three_body.mp4",
        ])
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("✓ Video encoding complete!\n");
            fs::remove_dir_all("temp_frames").ok();

            if let Ok(metadata) = fs::metadata("output/weber_three_body.mp4") {
                println!("File size: {:.1} MB", metadata.len() as f64 / 1_048_576.0);
            }

            println!("\n✓ SUCCESS!");
            println!("\nVideo: output/weber_three_body.mp4");
            println!("\nWhat to see:");
            println!("  • Red particle: Positive charge (+1 nC)");
            println!("  • Blue particle: Negative charge (-1 nC)");
            println!("  • Green particle: Positive charge (+1 nC)");
            println!("  • Colored trails showing particle paths");
            println!("  • Complex orbital dynamics");
            println!("\nPhysics - Weber Electrodynamics:");
            println!("  • Action-at-a-distance (no fields!)");
            println!("  • Forces depend on velocity: 1 - (ṙ)²/(2c²)");
            println!("  • Forces depend on acceleration: r·r̈/c²");
            println!("  • Energy conservation (symplectic integrator)");
            println!("  • Different from Maxwell - velocity/acceleration corrections");
            println!("\nComparison to Maxwell:");
            println!("  • Maxwell: Field-mediated, retarded interactions");
            println!("  • Weber: Instantaneous action-at-a-distance");
            println!("  • Weber includes velocity-dependent 'magnetic' forces");
            println!("  • Both reduce to Coulomb's law at low velocities");
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

/// Export visualization showing particles and their trajectories
fn export_weber_frame(system: &WeberParticles, trajectories: &[Vec<Vec2>], frame: usize) {
    let width = 512;
    let height = 512;

    // Visualization bounds (world coordinates in meters)
    let bounds = 0.03; // ±3 cm
    let center = Vec2::new(0.0, 0.0);

    let mut rgba = vec![0u8; width * height * 4];

    // Helper: world to pixel coordinates
    let world_to_pixel = |pos: Vec2| -> (i32, i32) {
        let x_norm = (pos.x - center.x + bounds) / (2.0 * bounds);
        let y_norm = (pos.y - center.y + bounds) / (2.0 * bounds);

        let px = (x_norm * width as f32) as i32;
        let py = (y_norm * height as f32) as i32;

        (px, height as i32 - 1 - py) // Flip y-axis
    };

    // Background: dark gray
    for i in 0..width * height {
        rgba[i * 4] = 20;
        rgba[i * 4 + 1] = 20;
        rgba[i * 4 + 2] = 20;
        rgba[i * 4 + 3] = 255;
    }

    // Draw trajectories as fading trails
    let colors = [
        [255, 80, 80],   // Red (particle 1)
        [80, 120, 255],  // Blue (particle 2)
        [80, 255, 120],  // Green (particle 3)
    ];

    for (i, trajectory) in trajectories.iter().enumerate() {
        let trail_length = trajectory.len().min(200);
        let start_idx = trajectory.len().saturating_sub(trail_length);

        for (t_idx, &pos) in trajectory[start_idx..].iter().enumerate() {
            let (px, py) = world_to_pixel(pos);

            if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                let fade = (t_idx as f32 / trail_length as f32).powf(2.0);
                let idx = (py as usize * width + px as usize) * 4;

                if idx + 3 < rgba.len() {
                    rgba[idx] = (colors[i][0] as f32 * fade) as u8;
                    rgba[idx + 1] = (colors[i][1] as f32 * fade) as u8;
                    rgba[idx + 2] = (colors[i][2] as f32 * fade) as u8;
                    rgba[idx + 3] = 255;
                }
            }
        }
    }

    // Draw particles as circles
    let particle_radius = 8; // pixels

    for (i, particle) in system.particles.iter().enumerate() {
        let (cx, cy) = world_to_pixel(particle.position);

        for dy in -particle_radius..=particle_radius {
            for dx in -particle_radius..=particle_radius {
                if dx * dx + dy * dy <= particle_radius * particle_radius {
                    let px = cx + dx;
                    let py = cy + dy;

                    if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                        let idx = (py as usize * width + px as usize) * 4;

                        if idx + 3 < rgba.len() {
                            rgba[idx] = colors[i][0];
                            rgba[idx + 1] = colors[i][1];
                            rgba[idx + 2] = colors[i][2];
                            rgba[idx + 3] = 255;
                        }
                    }
                }
            }
        }
    }

    // Draw center cross
    for i in -10..=10 {
        let (px, py) = world_to_pixel(Vec2::new(i as f32 * 0.001, 0.0));
        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
            let idx = (py as usize * width + px as usize) * 4;
            rgba[idx] = 100;
            rgba[idx + 1] = 100;
            rgba[idx + 2] = 100;
        }

        let (px, py) = world_to_pixel(Vec2::new(0.0, i as f32 * 0.001));
        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
            let idx = (py as usize * width + px as usize) * 4;
            rgba[idx] = 100;
            rgba[idx + 1] = 100;
            rgba[idx + 2] = 100;
        }
    }

    image::save_buffer(
        format!("temp_frames/weber_{:04}.png", frame),
        &rgba,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
