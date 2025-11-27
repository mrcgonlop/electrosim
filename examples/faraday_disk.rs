//! Faraday Disk (Homopolar Generator) - Fluid EM Analog
//!
//! Demonstrates electromagnetic induction using fluid dynamics.
//! A rotating charged disk creates a radial electric field through
//! the coupling between fluid vorticity (B-field) and pressure gradient (E-field).
//!
//! Physics:
//! - Rotating fluid disk → vorticity ω (analog of magnetic field B)
//! - Charge in rotating fluid → moving charges
//! - Pressure gradient ∇p → electric field E
//! - Prediction: E_radial ∝ ω × v (Lorentz-like force in fluid)
//!
//! Run with: cargo run --example faraday_disk --release

use em_physics_sandbox::physics::Fluid2D;
use em_physics_sandbox::visualization::slice2d::colormaps;
use glam::Vec2;
use std::fs;
use std::process::Command;

fn main() {
    println!("Faraday Disk - Fluid EM Analog");
    println!("===============================\n");

    // Simulation parameters
    let nx = 256;
    let ny = 256;
    let spacing = 0.0025; // 2.5mm cells
    let viscosity = 1e-5; // Low viscosity
    let dt = 0.001; // 1ms timestep

    println!("Simulation parameters:");
    println!("  Resolution: {}×{}", nx, ny);
    println!("  Domain: {:.2}×{:.2} m", nx as f32 * spacing, ny as f32 * spacing);
    println!("  Viscosity: {:.2e} m²/s", viscosity);
    println!("  Time step: {:.2e} s\n", dt);

    let mut fluid = Fluid2D::new(nx, ny, spacing, viscosity, dt);

    // Disk parameters
    let domain_center = Vec2::new(
        nx as f32 * spacing * 0.5,
        ny as f32 * spacing * 0.5,
    );
    let disk_radius = 0.15; // 15cm radius disk
    let circulation = 5.0; // Rotation strength
    let charge_density = 1e-4; // C/m² (uniform charge on disk)

    println!("Faraday disk setup:");
    println!("  Center: ({:.2}, {:.2}) m", domain_center.x, domain_center.y);
    println!("  Radius: {:.2} m", disk_radius);
    println!("  Circulation: {:.1} m²/s", circulation);
    println!("  Charge density: {:.2e} C/m²\n", charge_density);

    // Initialize rotating disk (vortex)
    fluid.add_vortex(domain_center, circulation, disk_radius);

    // Add charge to the disk
    fluid.add_charge_in_disk(domain_center, disk_radius, charge_density);

    // Coupling constant for E-field (dimensional analysis)
    // E ~ ∇p, dimensionally [V/m] ~ [Pa/m] * constant
    // Choosing constant to give reasonable E-field values
    let coupling_constant = 1.0; // Adjust to match real Faraday disk EMF

    // Create output directories
    fs::create_dir_all("output").ok();
    fs::create_dir_all("temp_frames").ok();

    // Animation parameters
    let total_frames = 300;
    let steps_per_frame = 5;
    let fps = 30;

    println!("Animation settings:");
    println!("  Frames: {}", total_frames);
    println!("  Steps per frame: {}", steps_per_frame);
    println!("  Output: {} FPS", fps);
    println!("  Total time: {:.2} s\n", total_frames as f32 * steps_per_frame as f32 * dt);

    println!("Running simulation...");
    println!("(Measuring radial E-field at disk edge)\n");

    // Measurements
    let mut max_e_radial_ever = 0.0_f32;

    for frame in 0..total_frames {
        // Advance simulation
        for _ in 0..steps_per_frame {
            fluid.step();
        }

        // Compute E-field from pressure gradient
        fluid.compute_e_field_from_pressure(coupling_constant);

        // Measure radial E-field at disk edge
        let e_radial = fluid.radial_e_field(domain_center, disk_radius);
        max_e_radial_ever = max_e_radial_ever.max(e_radial.abs());

        // Export visualization frame
        export_faraday_frame(&fluid, domain_center, disk_radius, frame);

        if frame % 30 == 0 || frame == total_frames - 1 {
            let progress = 100.0 * (frame + 1) as f32 / total_frames as f32;
            let max_vort = fluid.max_vorticity();
            let max_e = fluid.max_e_field();

            println!("  Frame {:3}/{} ({:3.0}%) | ω_max: {:.1e} s⁻¹ | E_max: {:.2e} V/m | E_radial: {:.2e} V/m",
                     frame + 1, total_frames, progress, max_vort, max_e, e_radial);
        }
    }

    println!("\n✓ Simulation complete!");
    println!("  Max radial E-field: {:.2e} V/m", max_e_radial_ever);
    println!("  Generated {} frames\n", total_frames);

    // Encode video
    println!("Encoding video...");

    let status = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-framerate", &fps.to_string(),
            "-i", "temp_frames/faraday_%04d.png",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-crf", "18",
            "-preset", "medium",
            "output/faraday_disk.mp4",
        ])
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("✓ Video encoding complete!\n");
            fs::remove_dir_all("temp_frames").ok();

            if let Ok(metadata) = fs::metadata("output/faraday_disk.mp4") {
                println!("File size: {:.1} MB", metadata.len() as f64 / 1_048_576.0);
            }

            println!("\n✓ SUCCESS!");
            println!("\nVideo: output/faraday_disk.mp4");
            println!("\nWhat to see:");
            println!("  • Rotating fluid disk (vortex)");
            println!("  • Vorticity field (B-field analog) in blue/red");
            println!("  • E-field vectors (arrows) showing radial pattern");
            println!("  • Disk boundary marked in white");
            println!("\nPhysics - Martins' Fluid EM Theory:");
            println!("  • Vorticity ω ≈ Magnetic field B");
            println!("  • Pressure gradient ∇p ≈ Electric field E");
            println!("  • Rotating charged fluid generates radial E-field");
            println!("  • E_radial ∝ v × B (Lorentz force law)");
            println!("\nReal EM Analog:");
            println!("  • Faraday's homopolar generator (1831)");
            println!("  • Rotating conductor in magnetic field");
            println!("  • EMF = ∫(v × B)·dl from center to rim");
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

/// Export visualization showing vorticity, E-field vectors, and disk boundary
fn export_faraday_frame(fluid: &Fluid2D, center: Vec2, disk_radius: f32, frame: usize) {
    let nx = fluid.nx;
    let ny = fluid.ny;

    let max_vort = fluid.max_vorticity().max(1.0);

    let mut rgba = Vec::with_capacity(nx * ny * 4);

    // First pass: vorticity background
    for j in (0..ny).rev() {
        for i in 0..nx {
            let idx = fluid.index(i, j);
            let vort = fluid.vorticity[idx];

            // Vorticity colormap (blue/red)
            let normalized = (vort / max_vort).clamp(-1.0, 1.0);
            let t = (normalized * 0.5 + 0.5).clamp(0.0, 1.0);

            let mut rgb = colormaps::red_blue(t);

            // Mark disk boundary in white
            let pos = Vec2::new(i as f32 * fluid.spacing, j as f32 * fluid.spacing);
            let r = (pos - center).length();
            if (r - disk_radius).abs() < fluid.spacing * 1.5 {
                rgb = [255, 255, 255];
            }

            // Draw E-field vectors (every 8th cell)
            if i % 8 == 0 && j % 8 == 0 {
                let e = fluid.e_field[idx];
                if e.length() > 0.01 {
                    // Draw arrow tip
                    rgb = [0, 255, 0]; // Green arrows
                }
            }

            rgba.push(rgb[0]);
            rgba.push(rgb[1]);
            rgba.push(rgb[2]);
            rgba.push(255);
        }
    }

    image::save_buffer(
        format!("temp_frames/faraday_{:04}.png", frame),
        &rgba,
        nx as u32,
        ny as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
