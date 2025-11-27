//! Simple Vortex Visualization - Stable Version
//!
//! Visualizes vortex velocity fields without time-stepping.
//! Creates a beautiful MP4 showing two vortices rotating.
//!
//! Run with: cargo run --example vortex_simple --release

use em_physics_sandbox::physics::Fluid2D;
use em_physics_sandbox::visualization::slice2d::colormaps;
use glam::Vec2;
use std::fs;
use std::process::Command;

fn main() {
    println!("Simple Vortex Visualization");
    println!("============================\n");

    // Create output directories
    fs::create_dir_all("output").ok();
    fs::create_dir_all("temp_frames").ok();

    let nx = 256;
    let ny = 256;
    let spacing = 0.0025; // 2.5mm cells
    let total_frames = 300;
    let fps = 30;

    println!("Grid: {}×{}", nx, ny);
    println!("Frames: {}", total_frames);
    println!("FPS: {}\n", fps);

    println!("Generating frames...");

    for frame in 0..total_frames {
        let t = frame as f32 / total_frames as f32;
        let angle = t * 2.0 * std::f32::consts::PI;

        // Create fluid for this frame
        let mut fluid = Fluid2D::new(nx, ny, spacing, 0.0, 0.001);

        // Two vortices that orbit each other
        let center = Vec2::new(0.32, 0.32);
        let radius = 0.05;

        let vortex1_pos = center + Vec2::new(angle.cos(), angle.sin()) * radius;
        let vortex2_pos = center - Vec2::new(angle.cos(), angle.sin()) * radius;

        fluid.add_vortex(vortex1_pos, 5.0, 0.04);
        fluid.add_vortex(vortex2_pos, 5.0, 0.04);

        // Export frame
        export_vorticity_frame(&fluid, frame);

        if frame % 30 == 0 || frame == total_frames - 1 {
            let progress = 100.0 * (frame + 1) as f32 / total_frames as f32;
            println!("  Frame {}/{} ({:.0}%)", frame + 1, total_frames, progress);
        }
    }

    println!("\n✓ Generated {} frames\n", total_frames);

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
            "-preset", "slow",
            "output/vortex_orbit.mp4",
        ])
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("✓ Video complete!\n");
            fs::remove_dir_all("temp_frames").ok();

            if let Ok(metadata) = fs::metadata("output/vortex_orbit.mp4") {
                println!("Size: {:.1} MB", metadata.len() as f64 / 1_048_576.0);
            }

            println!("\n✓ Video: output/vortex_orbit.mp4");
            println!("\nShows:");
            println!("  • Two counter-rotating vortices");
            println!("  • Orbital motion around center");
            println!("  • Vorticity field visualization");
            println!("  • Red = positive rotation");
            println!("  • Blue = negative rotation");
        }
        _ => {
            eprintln!("✗ ffmpeg encoding failed");
            eprintln!("Frames in temp_frames/");
        }
    }
}

fn export_vorticity_frame(fluid: &Fluid2D, frame: usize) {
    let nx = fluid.nx;
    let ny = fluid.ny;

    let max_vort = fluid.max_vorticity().max(0.1);

    let mut rgba = Vec::with_capacity(nx * ny * 4);

    for j in (0..ny).rev() {
        for i in 0..nx {
            let idx = fluid.index(i, j);
            let vort = fluid.vorticity[idx];

            // Map to [0, 1] with symmetric scale
            let t = (vort / (2.0 * max_vort) + 0.5).clamp(0.0, 1.0);

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
