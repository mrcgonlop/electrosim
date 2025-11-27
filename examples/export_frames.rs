//! Export field visualization frames as PNG images
//!
//! This example runs a simulation and exports visualization frames
//! without requiring a GUI window. Great for creating animations or
//! analyzing simulation results.
//!
//! Run with: cargo run --example export_frames --release

use em_physics_sandbox::{
    physics::{EMTheory, MaxwellTheory, Wave2D, ScalarWaveTheory},
    simulation::VoxelGrid,
    visualization::{FieldComponent, FieldSlice, SlicePlane, slice2d::colormaps},
};
use std::fs;

fn main() {
    env_logger::init();

    println!("EM Physics Sandbox - Frame Export Demo");
    println!("======================================\n");

    // Create output directory
    fs::create_dir_all("output").expect("Failed to create output directory");

    // Run 2D wave simulation
    run_2d_wave_demo();

    // Run 3D Maxwell simulation
    run_maxwell_demo();

    println!("\nDone! Check the 'output/' directory for PNG frames.");
}

fn run_2d_wave_demo() {
    println!("Running 2D wave simulation...");

    let wave = Wave2D::new(1.0);
    let nx = 128;
    let ny = 128;
    let mut field = vec![0.0; nx * ny];
    let mut velocity = vec![0.0; nx * ny];

    // Set Gaussian pulse
    wave.set_gaussian_pulse(&mut field, (64, 64), 1.0, 10.0, nx, ny);

    // Simulate and export frames
    let frames = 50;
    for frame in 0..frames {
        // Update simulation
        for _ in 0..4 {
            wave.update_scalar_field(&mut field, &mut velocity, nx, ny, 0.1, 0.005);
        }

        // Export frame
        if frame % 5 == 0 {
            export_2d_wave_frame(&field, nx, ny, frame);
            println!("  Exported frame {}/{}", frame, frames);
        }
    }

    println!("2D wave simulation complete!");
}

fn export_2d_wave_frame(field: &[f32], width: usize, height: usize, frame: usize) {
    // Find min/max for normalization
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;

    for &val in field {
        min_val = min_val.min(val);
        max_val = max_val.max(val);
    }

    let range = max_val - min_val;
    let range = if range > 1e-10 { range } else { 1.0 };

    // Create RGBA image
    let mut rgba = Vec::with_capacity(width * height * 4);

    for &val in field {
        let normalized = ((val - min_val) / range).clamp(0.0, 1.0);
        let rgb = colormaps::viridis(normalized);

        rgba.push(rgb[0]);
        rgba.push(rgb[1]);
        rgba.push(rgb[2]);
        rgba.push(255);
    }

    // Save as PNG
    image::save_buffer(
        format!("output/wave2d_frame_{:04}.png", frame),
        &rgba,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .expect("Failed to save image");
}

fn run_maxwell_demo() {
    println!("\nRunning 3D Maxwell simulation...");

    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(64, 64, 64, 0.01);

    // Initialize with Gaussian pulse
    maxwell.initialize(&mut grid);

    let dt = 1e-12;
    let frames = 50;

    for frame in 0..frames {
        // Update simulation
        for _ in 0..10 {
            maxwell.update_fields(&mut grid, dt);
        }

        // Export frame (XY plane at middle)
        if frame % 5 == 0 {
            export_maxwell_frame(&grid, frame);
            println!("  Exported frame {}/{}", frame, frames);
        }
    }

    println!("3D Maxwell simulation complete!");
}

fn export_maxwell_frame(grid: &VoxelGrid, frame: usize) {
    // Extract XY slice at middle
    let slice = FieldSlice::from_grid(
        grid,
        SlicePlane::XY,
        0.5,
        FieldComponent::EMagnitude,
    );

    // Convert to RGBA
    let rgba = slice.to_rgba8(colormaps::hot);

    // Save as PNG
    image::save_buffer(
        format!("output/maxwell3d_frame_{:04}.png", frame),
        &rgba,
        slice.width as u32,
        slice.height as u32,
        image::ColorType::Rgba8,
    )
    .expect("Failed to save image");
}
