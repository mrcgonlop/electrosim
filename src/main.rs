//! Main entry point for the EM physics sandbox

use em_physics_sandbox::{
    experiments::{Experiment, FaradayCage},
    physics::{MaxwellTheory, Wave2D, EMTheory, ScalarWaveTheory},
    simulation::VoxelGrid,
};
use std::time::Instant;

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("EM Physics Sandbox starting...");
    log::info!("Choose a demo:");
    log::info!("  1. 2D Wave Equation");
    log::info!("  2. 3D Maxwell FDTD");
    log::info!("  3. Faraday Cage Experiment");

    // For now, run all demos in sequence
    run_2d_wave_demo();
    run_maxwell_demo();
    run_faraday_cage_demo();

    log::info!("All demos complete!");
}

/// Run the 2D wave equation demo
fn run_2d_wave_demo() {
    log::info!("\n=== 2D Wave Equation Demo ===");

    let wave = Wave2D::new(1.0); // wave speed = 1.0 m/s
    let nx = 128;
    let ny = 128;
    let dx = 0.1;
    let dt = 0.005; // CFL condition satisfied

    let mut field = vec![0.0; nx * ny];
    let mut velocity = vec![0.0; nx * ny];

    // Set Gaussian pulse in center
    wave.set_gaussian_pulse(&mut field, (nx / 2, ny / 2), 1.0, 8.0, nx, ny);

    let initial_energy = wave.total_energy(&field, &velocity);
    log::info!("Initial energy: {:.6}", initial_energy);

    let steps = 200;
    let start = Instant::now();

    for step in 0..steps {
        wave.update_scalar_field(&mut field, &mut velocity, nx, ny, dx, dt);

        if step % 50 == 0 {
            let energy = wave.total_energy(&field, &velocity);
            let energy_change = ((energy - initial_energy) / initial_energy).abs();
            log::info!(
                "Step {}: energy = {:.6}, energy change = {:.4}%",
                step,
                energy,
                energy_change * 100.0
            );
        }
    }

    let elapsed = start.elapsed();
    let final_energy = wave.total_energy(&field, &velocity);
    let energy_conservation = ((final_energy - initial_energy) / initial_energy).abs();

    log::info!("Simulation complete in {:.3}s", elapsed.as_secs_f32());
    log::info!("Final energy: {:.6}", final_energy);
    log::info!("Energy conservation: {:.4}%", energy_conservation * 100.0);

    if energy_conservation < 0.2 {
        log::info!("✓ Energy well conserved");
    } else {
        log::warn!("✗ Significant energy drift");
    }
}

/// Run the 3D Maxwell FDTD demo
fn run_maxwell_demo() {
    log::info!("\n=== 3D Maxwell FDTD Demo ===");

    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(32, 32, 32, 0.01); // 32³ grid, 1cm spacing

    // Initialize with Gaussian pulse
    maxwell.initialize(&mut grid);

    let initial_energy = maxwell.total_energy(&grid);
    log::info!("Initial energy: {:.6e} J", initial_energy);

    let dt = 1e-12; // 1 picosecond
    let steps = 100;
    let start = Instant::now();

    for step in 0..steps {
        maxwell.update_fields(&mut grid, dt);

        if step % 25 == 0 {
            let energy = maxwell.total_energy(&grid);
            let energy_change = if initial_energy > 0.0 {
                ((energy - initial_energy) / initial_energy).abs()
            } else {
                0.0
            };

            log::info!(
                "Step {}: time = {:.2e} s, energy = {:.6e} J, change = {:.4}%",
                step,
                step as f32 * dt,
                energy,
                energy_change * 100.0
            );
        }
    }

    let elapsed = start.elapsed();
    let final_energy = maxwell.total_energy(&grid);

    log::info!("Simulation complete in {:.3}s", elapsed.as_secs_f32());
    log::info!("Final energy: {:.6e} J", final_energy);
    log::info!(
        "Performance: {:.1} steps/s, {:.1} Mvoxel-updates/s",
        steps as f32 / elapsed.as_secs_f32(),
        (steps as f32 * grid.size() as f32) / (elapsed.as_secs_f32() * 1e6)
    );
}

/// Run the Faraday cage experiment demo
fn run_faraday_cage_demo() {
    log::info!("\n=== Faraday Cage Experiment Demo ===");

    let maxwell = MaxwellTheory::new();
    let cage = FaradayCage::new(0.7, 100.0).with_shielding(15.0); // Require 15 dB shielding

    let mut grid = VoxelGrid::new(32, 32, 32, 0.01);

    // Set up the experiment
    cage.setup(&mut grid);

    log::info!("Running simulation to allow fields to settle...");

    let dt = 1e-12;
    let steps = 200;
    let start = Instant::now();

    for step in 0..steps {
        maxwell.update_fields(&mut grid, dt);

        if step % 50 == 0 {
            let result = cage.check_result(&grid);
            log::info!("Step {}: {:?}", step, result);

            if result.is_pass() {
                log::info!("✓ Experiment passed at step {}!", step);
                break;
            }
        }
    }

    let elapsed = start.elapsed();
    let final_result = cage.check_result(&grid);

    log::info!("Simulation complete in {:.3}s", elapsed.as_secs_f32());
    log::info!("Final result: {:?}", final_result);

    if final_result.is_pass() {
        log::info!("✓ Faraday cage successfully shields electromagnetic field");
    } else {
        log::warn!("✗ Faraday cage did not provide sufficient shielding");
    }
}
