//! Newtonian Gravity - Three Body Problem
//!
//! Simulates three masses interacting via Newton's law of gravitation:
//! F = G·m₁m₂/r²
//!
//! This uses the same particle infrastructure as Weber electrodynamics,
//! demonstrating the generality of the action-at-a-distance framework.
//!
//! Classic chaotic system - demonstrates sensitivity to initial conditions.
//!
//! Run with: cargo run --example gravity_three_body --release

use em_physics_sandbox::physics::{WeberParticles, Particle};
use glam::Vec2;
use std::fs;
use std::process::Command;

/// Gravitational particle system (reusing Weber infrastructure)
struct GravitySystem {
    particles: Vec<Particle>,
    g: f32, // Gravitational constant
}

impl GravitySystem {
    fn new() -> Self {
        Self {
            particles: Vec::new(),
            g: 6.674e-11, // N⋅m²/kg²
        }
    }

    fn add_particle(&mut self, particle: Particle) {
        self.particles.push(particle);
    }

    /// Newton's gravitational force: F = G·m₁m₂/r² (attractive)
    fn gravitational_force(&self, i: usize, j: usize) -> Vec2 {
        let pi = &self.particles[i];
        let pj = &self.particles[j];

        let r_vec = pj.position - pi.position;
        let r = r_vec.length();

        if r < 1e-10 {
            return Vec2::ZERO; // Avoid singularity
        }

        let r_hat = r_vec / r;

        // F = G·m₁m₂/r² (always attractive for gravity)
        let f_magnitude = self.g * pi.mass * pj.mass / (r * r);

        f_magnitude * r_hat
    }

    fn compute_force(&self, i: usize) -> Vec2 {
        let mut total_force = Vec2::ZERO;

        for j in 0..self.particles.len() {
            if i != j {
                total_force += self.gravitational_force(i, j);
            }
        }

        total_force
    }

    /// Velocity Verlet integration (same as Weber)
    fn step(&mut self, dt: f32) {
        let n = self.particles.len();

        let old_accelerations: Vec<Vec2> = self.particles.iter()
            .map(|p| p.acceleration)
            .collect();
        let old_velocities: Vec<Vec2> = self.particles.iter()
            .map(|p| p.velocity)
            .collect();

        // Update positions
        for i in 0..n {
            let vel = old_velocities[i];
            let acc = old_accelerations[i];
            self.particles[i].position += vel * dt + 0.5 * acc * dt * dt;
        }

        // Compute new accelerations
        for i in 0..n {
            let force = self.compute_force(i);
            self.particles[i].acceleration = force / self.particles[i].mass;
        }

        // Update velocities
        for i in 0..n {
            let new_acc = self.particles[i].acceleration;
            self.particles[i].velocity = old_velocities[i] + 0.5 * (old_accelerations[i] + new_acc) * dt;
        }
    }

    fn kinetic_energy(&self) -> f32 {
        self.particles.iter()
            .map(|p| 0.5 * p.mass * p.velocity.length_squared())
            .sum()
    }

    fn potential_energy(&self) -> f32 {
        let mut u = 0.0;

        for i in 0..self.particles.len() {
            for j in (i + 1)..self.particles.len() {
                let r = (self.particles[j].position - self.particles[i].position).length();
                if r > 1e-10 {
                    // U = -G·m₁m₂/r (negative for attractive force)
                    u -= self.g * self.particles[i].mass * self.particles[j].mass / r;
                }
            }
        }

        u
    }

    fn total_energy(&self) -> f32 {
        self.kinetic_energy() + self.potential_energy()
    }

    fn center_of_mass(&self) -> Vec2 {
        let total_mass: f32 = self.particles.iter().map(|p| p.mass).sum();
        let com: Vec2 = self.particles.iter()
            .map(|p| p.position * p.mass)
            .sum();

        if total_mass > 0.0 {
            com / total_mass
        } else {
            Vec2::ZERO
        }
    }
}

fn main() {
    println!("Newtonian Gravity - Three Body Problem");
    println!("=======================================\n");

    println!("Newton's law of gravitation:");
    println!("  F = G·m₁m₂/r²");
    println!("\nUsing normalized units:");
    println!("  [Length] = 1 AU = 1.496×10¹¹ m");
    println!("  [Mass] = 1 M☉ = 1.989×10³⁰ kg");
    println!("  [Time] = 1 day = 86400 s");
    println!("  G_norm = 2.96×10⁻⁴ AU³/(M☉·day²)\n");

    // NORMALIZED UNITS to avoid overflow
    // Distance in AU, mass in solar masses, time in days
    let g_normalized = 2.96e-4; // G in AU³/(M☉·day²)

    let mut system = GravitySystem {
        particles: Vec::new(),
        g: g_normalized,
    };

    // Timestep: 0.1 days
    let dt = 0.1;

    println!("Simulation parameters:");
    println!("  Time step: {:.2} days ({:.1} hours)", dt, dt * 24.0);
    println!("  Integration: Velocity Verlet (symplectic)\n");

    println!("Three-body configuration:");
    println!("  Scenario: Sun-Jupiter binary with perturbing asteroid\n");

    // Configuration in normalized units
    // Mass 1: "Sun" (left of barycenter)
    let m1 = 2.0; // 1 solar mass
    let sep = 0.5; // 0.5 AU separation
    let p1_pos = Vec2::new(-sep * 0.001 / 1.001, 0.0); // Barycenter correction
    // Orbital velocity for circular orbit
    let v_orbit = (g_normalized * 0.301 / (2.0 * sep)).sqrt(); // AU/day
    let p1_vel = Vec2::new(0.0, v_orbit * 0.001 / 1.001);
    system.add_particle(Particle::new(p1_pos, p1_vel, 0.0, m1));

    // Mass 2: "Jupiter" (right of barycenter)
    let m2 = 0.5 ; // ~1 Jupiter mass
    let p2_pos = Vec2::new(sep * 1.0 / 1.001, 0.0);
    let p2_vel = Vec2::new(0.0, -v_orbit);
    system.add_particle(Particle::new(p2_pos, p2_vel, 0.0, m2));

    // Mass 3: "Asteroid" (perturber)
    let m3 = 0.5; // Negligible mass (asteroid)
    let p3_pos = Vec2::new(-sep * 1.0 / 1.005, 0.0);
    let p3_vel = Vec2::new(v_orbit, v_orbit);
    system.add_particle(Particle::new(p3_pos, p3_vel, 0.0, m3));

    println!("Mass 1 (Sun): {:.3} M☉", m1);
    println!("  Position: ({:.4}, {:.4}) AU", p1_pos.x, p1_pos.y);
    println!("  Velocity: ({:.4}, {:.4}) AU/day", p1_vel.x, p1_vel.y);

    println!("Mass 2 (Jupiter): {:.6} M☉ ({:.1} M_Jup)", m2, m2 / 0.001);
    println!("  Position: ({:.4}, {:.4}) AU", p2_pos.x, p2_pos.y);
    println!("  Velocity: ({:.4}, {:.4}) AU/day", p2_vel.x, p2_vel.y);

    println!("Mass 3 (Asteroid): {:.3e} M☉", m3);
    println!("  Position: ({:.4}, {:.4}) AU", p3_pos.x, p3_pos.y);
    println!("  Velocity: ({:.4}, {:.4}) AU/day\n", p3_vel.x, p3_vel.y);

    // Create output directories
    fs::create_dir_all("output").ok();
    fs::create_dir_all("temp_frames").ok();

    // Animation parameters
    let total_frames = 1200;
    let steps_per_frame = 5;
    let fps = 30;

    let total_time = total_frames as f32 * steps_per_frame as f32 * dt; // In days
    let total_years = total_time / 365.25;

    println!("Animation settings:");
    println!("  Frames: {}", total_frames);
    println!("  Steps per frame: {}", steps_per_frame);
    println!("  Total steps: {}", total_frames * steps_per_frame);
    println!("  Physical time: {:.1} days ({:.2} years)",
             total_time, total_years);
    println!("  Output: {} FPS\n", fps);

    println!("Running gravitational simulation...");
    println!("(Watch for chaotic three-body dynamics)\n");

    // Initialize accelerations before first step
    for i in 0..system.particles.len() {
        let force = system.compute_force(i);
        system.particles[i].acceleration = force / system.particles[i].mass;
    }

    let e_initial = system.total_energy();
    println!("Initial energy: {:.6e} J\n", e_initial);

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
        export_gravity_frame(&system, &trajectories, frame);

        if frame % 80 == 0 || frame == total_frames - 1 {
            let progress = 100.0 * (frame + 1) as f32 / total_frames as f32;
            let e = system.total_energy();
            let ke = system.kinetic_energy();
            let pe = system.potential_energy();
            let e_error = ((e - e_initial) / e_initial.abs()).abs();

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
            "-i", "temp_frames/gravity_%04d.png",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-crf", "18",
            "-preset", "medium",
            "output/gravity_three_body.mp4",
        ])
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("✓ Video encoding complete!\n");
            fs::remove_dir_all("temp_frames").ok();

            if let Ok(metadata) = fs::metadata("output/gravity_three_body.mp4") {
                println!("File size: {:.1} MB", metadata.len() as f64 / 1_048_576.0);
            }

            println!("\n✓ SUCCESS!");
            println!("\nVideo: output/gravity_three_body.mp4");
            println!("\nWhat to see:");
            println!("  • Yellow: Mass 1 (1.0 M☉)");
            println!("  • Cyan: Mass 2 (1.0 M☉)");
            println!("  • Magenta: Mass 3 (0.5 M☉)");
            println!("  • Colored trails showing orbital paths");
            println!("  • Chaotic, unpredictable motion");
            println!("\nPhysics - Newtonian Gravity:");
            println!("  • F = G·m₁m₂/r² (always attractive)");
            println!("  • No analytical solution for n≥3 bodies");
            println!("  • Chaotic dynamics (sensitive to initial conditions)");
            println!("  • Energy conservation test for integrator");
            println!("\nFamous Three-Body Problem:");
            println!("  • Poincaré showed no general closed-form solution");
            println!("  • Can exhibit chaotic behavior");
            println!("  • Relevant to star systems, planetary stability");
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

/// Export visualization showing masses and their trajectories
fn export_gravity_frame(system: &GravitySystem, trajectories: &[Vec<Vec2>], frame: usize) {
    let width = 512;
    let height = 512;

    // Visualization bounds in normalized units (AU)
    let bounds = 1.0; // ±1 AU
    let center = Vec2::new(0.0, 0.0);

    let mut rgba = vec![0u8; width * height * 4];

    // Helper: world to pixel coordinates (positions already in AU)
    let world_to_pixel = |pos: Vec2| -> (i32, i32) {
        let x_norm = (pos.x - center.x + bounds) / (2.0 * bounds);
        let y_norm = (pos.y - center.y + bounds) / (2.0 * bounds);

        let px = (x_norm * width as f32) as i32;
        let py = (y_norm * height as f32) as i32;

        (px, height as i32 - 1 - py)
    };

    // Background: black (space)
    for i in 0..width * height {
        rgba[i * 4] = 10;
        rgba[i * 4 + 1] = 10;
        rgba[i * 4 + 2] = 15;
        rgba[i * 4 + 3] = 255;
    }

    // Draw trajectories
    let colors = [
        [255, 220, 100],  // Yellow (mass 1)
        [100, 220, 255],  // Cyan (mass 2)
        [255, 100, 200],  // Magenta (mass 3)
    ];

    for (i, trajectory) in trajectories.iter().enumerate() {
        let trail_length = trajectory.len().min(300);
        let start_idx = trajectory.len().saturating_sub(trail_length);

        for (t_idx, &pos) in trajectory[start_idx..].iter().enumerate() {
            let (px, py) = world_to_pixel(pos);

            if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                let fade = (t_idx as f32 / trail_length as f32).powf(1.5);
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

    // Draw masses as circles (size proportional to mass)
    let base_radius = 6;

    for (i, particle) in system.particles.iter().enumerate() {
        let mass_scale = (particle.mass / 1.989e30).sqrt(); // Relative to solar mass
        let radius = (base_radius as f32 * mass_scale) as i32;

        let (cx, cy) = world_to_pixel(particle.position);

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
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

    // Draw center cross (in AU units)
    for i in -15..=15 {
        let (px, py) = world_to_pixel(Vec2::new(i as f32 * 0.05, 0.0)); // 0.05 AU increments
        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
            let idx = (py as usize * width + px as usize) * 4;
            rgba[idx] = 80;
            rgba[idx + 1] = 80;
            rgba[idx + 2] = 80;
        }

        let (px, py) = world_to_pixel(Vec2::new(0.0, i as f32 * 0.05));
        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
            let idx = (py as usize * width + px as usize) * 4;
            rgba[idx] = 80;
            rgba[idx + 1] = 80;
            rgba[idx + 2] = 80;
        }
    }

    image::save_buffer(
        format!("temp_frames/gravity_{:04}.png", frame),
        &rgba,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
