//! Export electromagnetic field visualization as MP4 video
//!
//! Creates a video showing field evolution over time.
//! Requires ffmpeg to be installed on your system.
//!
//! Run with: cargo run --example export_video --release

use em_physics_sandbox::{
    physics::{EMTheory, MaxwellTheory, Wave2D, ScalarWaveTheory},
    simulation::VoxelGrid,
    visualization::{FieldComponent, FieldSlice, SlicePlane, slice2d::colormaps},
};
use std::fs;
use std::process::Command;

fn main() {
    env_logger::init();

    println!("EM Physics Sandbox - Video Export");
    println!("==================================\n");

    // Create temp directory for frames
    fs::create_dir_all("temp_frames").expect("Failed to create temp directory");

    // Choose which simulation to record
    println!("Choose simulation:");
    println!("1. 2D Wave Equation (fast, 128x128)");
    println!("2. 3D Maxwell FDTD (slower, 64³ grid)");
    println!("3. Faraday Cage Experiment");

    // Default to 2D wave for this demo
    record_2d_wave_video();

    println!("\n✓ Video created: output/wave_simulation.mp4");
    println!("  You can play it with any video player!");
}

fn record_2d_wave_video() {
    println!("\nRecording 2D wave simulation...");

    let wave = Wave2D::new(1.0);
    let nx = 256;  // Higher resolution for video
    let ny = 256;
    let mut field = vec![0.0; nx * ny];
    let mut velocity = vec![0.0; nx * ny];

    // Set initial condition - Gaussian pulse
    wave.set_gaussian_pulse(&mut field, (nx/2, ny/2), 1.0, 15.0, nx, ny);

    // Record 300 frames (10 seconds at 30fps)
    let total_frames = 300;
    let fps = 30;

    println!("Generating {} frames at {}fps...", total_frames, fps);

    for frame in 0..total_frames {
        // Update simulation (multiple steps per frame for smoother animation)
        for _ in 0..2 {
            wave.update_scalar_field(&mut field, &mut velocity, nx, ny, 0.1, 0.005);
        }

        // Export frame
        export_2d_frame(&field, nx, ny, frame);

        if frame % 30 == 0 {
            println!("  Progress: {}/{} frames ({:.0}%)",
                     frame, total_frames,
                     100.0 * frame as f32 / total_frames as f32);
        }
    }

    println!("  Progress: {}/{} frames (100%)", total_frames, total_frames);
    println!("\nEncoding video with ffmpeg...");

    // Create output directory
    fs::create_dir_all("output").ok();

    // Use ffmpeg to create MP4
    let status = Command::new("ffmpeg")
        .args(&[
            "-y",  // Overwrite output file
            "-framerate", &fps.to_string(),
            "-i", "temp_frames/frame_%04d.png",
            "-c:v", "libx264",  // H.264 codec
            "-pix_fmt", "yuv420p",  // Compatible with most players
            "-crf", "23",  // Quality (lower = better, 23 is good)
            "-preset", "medium",  // Encoding speed
            "output/wave_simulation.mp4"
        ])
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("✓ Video encoding complete!");

            // Clean up temporary frames
            println!("Cleaning up temporary files...");
            fs::remove_dir_all("temp_frames").ok();

            // Get video file size
            if let Ok(metadata) = fs::metadata("output/wave_simulation.mp4") {
                let size_mb = metadata.len() as f64 / 1_048_576.0;
                println!("Video size: {:.1} MB", size_mb);
            }
        }
        Ok(_) => {
            eprintln!("✗ ffmpeg failed to encode video");
            eprintln!("  Frames are available in temp_frames/ directory");
        }
        Err(e) => {
            eprintln!("✗ ffmpeg not found: {}", e);
            eprintln!("  Please install ffmpeg:");
            eprintln!("  - Windows: choco install ffmpeg");
            eprintln!("  - Mac: brew install ffmpeg");
            eprintln!("  - Linux: sudo apt install ffmpeg");
            eprintln!("\n  Frames are available in temp_frames/ directory");
        }
    }
}

fn export_2d_frame(field: &[f32], width: usize, height: usize, frame: usize) {
    // Find min/max for normalization
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;

    for &val in field {
        min_val = min_val.min(val);
        max_val = max_val.max(val);
    }

    let range = max_val - min_val;
    let range = if range > 1e-10 { range } else { 1.0 };

    // Create RGBA image with viridis colormap
    let mut rgba = Vec::with_capacity(width * height * 4);

    for &val in field {
        let normalized = ((val - min_val) / range).clamp(0.0, 1.0);
        let rgb = colormaps::viridis(normalized);

        rgba.push(rgb[0]);
        rgba.push(rgb[1]);
        rgba.push(rgb[2]);
        rgba.push(255);
    }

    // Save frame
    image::save_buffer(
        format!("temp_frames/frame_{:04}.png", frame),
        &rgba,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .expect("Failed to save frame");
}

/// Record 3D Maxwell simulation
#[allow(dead_code)]
fn record_maxwell_video() {
    println!("\nRecording 3D Maxwell simulation...");

    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(64, 64, 64, 0.01);

    // Initialize with Gaussian pulse
    maxwell.initialize(&mut grid);

    let dt = 1e-12;
    let total_frames = 300;
    let fps = 30;

    println!("Generating {} frames at {}fps...", total_frames, fps);

    for frame in 0..total_frames {
        // Update simulation
        for _ in 0..20 {
            maxwell.update_fields(&mut grid, dt);
        }

        // Export XY slice at middle
        let slice = FieldSlice::from_grid(
            &grid,
            SlicePlane::XY,
            0.5,
            FieldComponent::EMagnitude,
        );

        let rgba = slice.to_rgba8(colormaps::hot);

        image::save_buffer(
            format!("temp_frames/frame_{:04}.png", frame),
            &rgba,
            slice.width as u32,
            slice.height as u32,
            image::ColorType::Rgba8,
        )
        .expect("Failed to save frame");

        if frame % 30 == 0 {
            println!("  Progress: {}/{} frames ({:.0}%)",
                     frame, total_frames,
                     100.0 * frame as f32 / total_frames as f32);
        }
    }

    println!("  Progress: {}/{} frames (100%)", total_frames, total_frames);
    println!("\nEncoding video with ffmpeg...");

    // Use ffmpeg to create MP4
    let status = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-framerate", &fps.to_string(),
            "-i", "temp_frames/frame_%04d.png",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-crf", "23",
            "-preset", "medium",
            "output/maxwell_simulation.mp4"
        ])
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("✓ Video encoding complete!");
            fs::remove_dir_all("temp_frames").ok();
        }
        Ok(_) => eprintln!("✗ ffmpeg failed"),
        Err(_) => eprintln!("✗ ffmpeg not found. Frames saved in temp_frames/"),
    }
}
