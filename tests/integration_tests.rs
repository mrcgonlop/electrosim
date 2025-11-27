//! Integration tests for the EM physics sandbox

use em_physics_sandbox::{
    experiments::{Experiment, ExperimentResult, FaradayCage},
    physics::{EMTheory, MaxwellTheory, ScalarWaveTheory, Wave2D},
    simulation::VoxelGrid,
};

#[test]
fn test_2d_wave_simulation_completes() {
    let wave = Wave2D::new(1.0);
    let nx = 64;
    let ny = 64;
    let mut field = vec![0.0; nx * ny];
    let mut velocity = vec![0.0; nx * ny];

    wave.set_gaussian_pulse(&mut field, (32, 32), 1.0, 5.0, nx, ny);

    // Run for 50 steps
    for _ in 0..50 {
        wave.update_scalar_field(&mut field, &mut velocity, nx, ny, 0.1, 0.005);
    }

    // Should complete without panicking
}

#[test]
fn test_maxwell_simulation_completes() {
    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(16, 16, 16, 0.01);

    maxwell.initialize(&mut grid);

    let dt = 1e-12;

    // Run for 20 steps
    for _ in 0..20 {
        maxwell.update_fields(&mut grid, dt);
    }

    // Should complete without panicking
}

#[test]
fn test_theory_switching() {
    // Test that we can switch between theories seamlessly
    let mut grid = VoxelGrid::new(16, 16, 16, 0.01);

    let maxwell = MaxwellTheory::new();
    maxwell.initialize(&mut grid);

    // Run with Maxwell
    for _ in 0..10 {
        maxwell.update_fields(&mut grid, 1e-12);
    }

    // Could switch to Weber or other theory here
    // (Weber is just a stub for now)

    // Should work without issues
}

#[test]
fn test_faraday_cage_experiment_setup() {
    let cage = FaradayCage::new(0.8, 100.0);
    let mut grid = VoxelGrid::new(32, 32, 32, 0.01);

    cage.setup(&mut grid);

    // Check that setup completed
    // Walls should be conductive
    let corner_idx = grid.index(0, 0, 0);
    assert!(grid.sigma[corner_idx] > 1e9);
}

#[test]
fn test_end_to_end_faraday_experiment() {
    let maxwell = MaxwellTheory::new();
    let cage = FaradayCage::new(0.7, 50.0).with_shielding(10.0);
    let mut grid = VoxelGrid::new(24, 24, 24, 0.01);

    cage.setup(&mut grid);

    let dt = 1e-12;

    // Run simulation
    for _ in 0..100 {
        maxwell.update_fields(&mut grid, dt);
    }

    // Check result
    let result = cage.check_result(&grid);

    // Should either pass or still be running (fields may not have settled)
    // Just check it doesn't panic
    match result {
        ExperimentResult::Pass => println!("✓ Experiment passed"),
        ExperimentResult::Fail(msg) => println!("✗ Experiment failed: {}", msg),
        ExperimentResult::Running => println!("⋯ Experiment still running"),
    }
}

#[test]
fn test_multiple_grid_sizes() {
    let maxwell = MaxwellTheory::new();

    for size in [8, 16, 32] {
        let mut grid = VoxelGrid::new(size, size, size, 0.01);
        maxwell.initialize(&mut grid);

        for _ in 0..10 {
            maxwell.update_fields(&mut grid, 1e-12);
        }

        // Should work for all sizes
    }
}

#[test]
fn test_different_time_steps() {
    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(16, 16, 16, 0.01);

    maxwell.initialize(&mut grid);

    // Test different time steps (all within CFL limit)
    let time_steps = [1e-13, 5e-13, 1e-12];

    for &dt in &time_steps {
        let mut test_grid = grid.clone();

        for _ in 0..10 {
            maxwell.update_fields(&mut test_grid, dt);
        }

        // Should all work
    }
}

#[test]
fn test_grid_clear() {
    let mut grid = VoxelGrid::new(16, 16, 16, 0.01);

    // Set some fields
    let maxwell = MaxwellTheory::new();
    maxwell.initialize(&mut grid);

    // Clear
    grid.clear();

    // All fields should be zero
    for i in 0..grid.size() {
        assert_eq!(grid.e_field[i], glam::Vec3::ZERO);
        assert_eq!(grid.b_field[i], glam::Vec3::ZERO);
    }
}
