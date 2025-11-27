//! Physics validation tests
//!
//! These tests verify that the simulations obey fundamental physical laws
//! and match known analytical solutions.

use em_physics_sandbox::{
    physics::{EMField, EMTheory, MaxwellTheory, ScalarWaveTheory, Wave2D},
    simulation::VoxelGrid,
};
use approx::assert_relative_eq;
use glam::Vec3;

/// Test that energy is conserved in the 2D wave equation
#[test]
fn test_2d_wave_energy_conservation() {
    let wave = Wave2D::new(1.0);
    let nx = 64;
    let ny = 64;
    let mut field = vec![0.0; nx * ny];
    let mut velocity = vec![0.0; nx * ny];

    wave.set_gaussian_pulse(&mut field, (32, 32), 1.0, 5.0, nx, ny);

    let initial_energy = wave.total_energy(&field, &velocity);

    // Run for 100 steps
    for _ in 0..100 {
        wave.update_scalar_field(&mut field, &mut velocity, nx, ny, 0.1, 0.005);
    }

    let final_energy = wave.total_energy(&field, &velocity);
    let energy_change = ((final_energy - initial_energy) / initial_energy).abs();

    // Energy should be conserved within 20% (generous for simple scheme)
    assert!(
        energy_change < 0.2,
        "Energy not conserved: change = {:.2}%",
        energy_change * 100.0
    );
}

/// Test that energy never spontaneously increases in isolated system
#[test]
fn test_energy_never_increases() {
    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(16, 16, 16, 0.01);

    maxwell.initialize(&mut grid);

    let initial_energy = maxwell.total_energy(&grid);

    let dt = 1e-12;

    for _ in 0..50 {
        maxwell.update_fields(&mut grid, dt);

        let current_energy = maxwell.total_energy(&grid);

        // Energy should not increase (may decrease due to numerical dissipation)
        // Allow 1% tolerance for numerical noise
        assert!(
            current_energy <= initial_energy * 1.01,
            "Energy increased: {} -> {}",
            initial_energy,
            current_energy
        );
    }
}

/// Test that static fields remain static
#[test]
fn test_static_field_stability() {
    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(16, 16, 16, 0.01);

    // Start with zero fields
    grid.clear();

    let dt = 1e-12;

    // Run simulation
    for _ in 0..100 {
        maxwell.update_fields(&mut grid, dt);
    }

    // All fields should still be zero
    for i in 0..grid.size() {
        assert_relative_eq!(grid.e_field[i].length(), 0.0, epsilon = 1e-6);
        assert_relative_eq!(grid.b_field[i].length(), 0.0, epsilon = 1e-6);
    }
}

/// Test that E and B fields remain perpendicular in a plane wave
#[test]
fn test_plane_wave_perpendicularity() {
    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(32, 8, 8, 0.01);

    // Initialize plane wave propagating in x direction
    maxwell.initialize_plane_wave(&mut grid, 1e9, Vec3::X);

    let dt = 1e-12;

    // Propagate
    for _ in 0..20 {
        maxwell.update_fields(&mut grid, dt);
    }

    // Sample some interior points
    let mut max_dot_product = 0.0;

    for i in 4..28 {
        for j in 2..6 {
            for k in 2..6 {
                let idx = grid.index(i, j, k);
                let e = grid.e_field[idx];
                let b = grid.b_field[idx];

                if e.length() > 0.01 && b.length() > 0.01 {
                    let dot = e.normalize().dot(b.normalize()).abs();
                    max_dot_product = max_dot_product.max(dot);
                }
            }
        }
    }

    // E and B should be perpendicular (dot product near zero)
    // Allow some tolerance for numerical errors
    assert!(
        max_dot_product < 0.3,
        "E and B not perpendicular: max dot = {}",
        max_dot_product
    );
}

/// Test Gauss's law: ∇·E = ρ/ε₀ (in vacuum, should be zero)
#[test]
fn test_gauss_law_in_vacuum() {
    use em_physics_sandbox::utils::math::divergence;

    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(16, 16, 16, 0.01);

    maxwell.initialize(&mut grid);

    let dt = 1e-12;

    // Run for a few steps
    for _ in 0..20 {
        maxwell.update_fields(&mut grid, dt);
    }

    // Check divergence of E field in interior (should be near zero in vacuum)
    let mut max_divergence = 0.0;

    for i in 2..14 {
        for j in 2..14 {
            for k in 2..14 {
                let div = divergence(
                    &grid.e_field,
                    i,
                    j,
                    k,
                    grid.nx,
                    grid.ny,
                    grid.nz,
                    grid.spacing,
                );
                max_divergence = max_divergence.max(div.abs());
            }
        }
    }

    // Divergence should be small (no charges in vacuum)
    // This is a rough check - FDTD doesn't perfectly conserve this
    assert!(
        max_divergence < 1e6,
        "Divergence of E too large: {}",
        max_divergence
    );
}

/// Test that electromagnetic waves propagate at speed c
#[test]
fn test_wave_speed() {
    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(64, 8, 8, 0.001); // 64mm in x direction

    // Initialize Gaussian pulse at left side
    let center = Vec3::new(0.01, 0.004, 0.004); // 10mm, centered in y,z
    maxwell.initialize_gaussian_pulse(&mut grid, center, 0.002, 100.0);

    let dt = 1e-13; // 0.1 ps
    let steps = 200; // Total time: 20 ps

    // Track the position of maximum field
    let mut max_positions = Vec::new();

    for step in 0..steps {
        if step % 20 == 0 {
            // Find position of maximum E field
            let mut max_field = 0.0;
            let mut max_pos = 0.0;

            for i in 0..grid.nx {
                let idx = grid.index(i, 4, 4);
                let field_magnitude = grid.e_field[idx].length();
                if field_magnitude > max_field {
                    max_field = field_magnitude;
                    max_pos = i as f32 * grid.spacing;
                }
            }

            max_positions.push((step as f32 * dt, max_pos));
        }

        maxwell.update_fields(&mut grid, dt);
    }

    // Calculate average speed from position vs time
    if max_positions.len() >= 2 {
        let (t0, x0) = max_positions[0];
        let (t1, x1) = max_positions[max_positions.len() - 1];

        let distance = x1 - x0;
        let time = t1 - t0;
        let speed = distance / time;

        // Speed should be close to c (within 30% for this coarse simulation)
        let c = 3e8;
        let speed_ratio = speed / c;

        println!("Measured speed: {:.2e} m/s ({:.1}% of c)", speed, speed_ratio * 100.0);

        assert!(
            speed_ratio > 0.5 && speed_ratio < 1.5,
            "Wave speed not close to c: {:.2}% of c",
            speed_ratio * 100.0
        );
    }
}

/// Test that perfect conductors zero out fields
#[test]
fn test_perfect_conductor_boundary() {
    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(16, 16, 16, 0.01);

    // Set entire grid to have some field
    maxwell.initialize(&mut grid);

    // Make a conducting region
    grid.set_conductor_region((4, 4, 4), (12, 12, 12));

    let dt = 1e-12;

    // Run simulation
    for _ in 0..20 {
        maxwell.update_fields(&mut grid, dt);
    }

    // Check that fields are zeroed in conductor
    for i in 4..=12 {
        for j in 4..=12 {
            for k in 4..=12 {
                let idx = grid.index(i, j, k);
                assert_relative_eq!(grid.e_field[idx].length(), 0.0, epsilon = 1e-3);
                assert_relative_eq!(grid.b_field[idx].length(), 0.0, epsilon = 1e-3);
            }
        }
    }
}

/// Test that the Poynting vector points in the direction of energy flow
#[test]
fn test_poynting_vector_direction() {
    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(32, 8, 8, 0.01);

    // Plane wave propagating in +x direction
    maxwell.initialize_plane_wave(&mut grid, 1e9, Vec3::X);

    let dt = 1e-12;

    // Propagate for a few steps
    for _ in 0..10 {
        maxwell.update_fields(&mut grid, dt);
    }

    // Check Poynting vectors in middle of grid
    let mut has_positive_x_flow = false;

    for i in 8..24 {
        for j in 2..6 {
            for k in 2..6 {
                let idx = grid.index(i, j, k);
                let field = EMField::new(grid.e_field[idx], grid.b_field[idx]);
                let poynting = field.poynting_vector();

                if poynting.length() > 1e-6 {
                    // Poynting vector should have positive x component (energy flowing in +x)
                    if poynting.x > 0.0 {
                        has_positive_x_flow = true;
                    }
                }
            }
        }
    }

    assert!(
        has_positive_x_flow,
        "Poynting vector should indicate energy flow in +x direction"
    );
}

/// Property-based test: Field magnitude should remain bounded
#[test]
fn test_field_magnitude_bounded() {
    let maxwell = MaxwellTheory::new();
    let mut grid = VoxelGrid::new(16, 16, 16, 0.01);

    maxwell.initialize(&mut grid);

    // Find initial maximum field
    let mut initial_max = 0.0;
    for i in 0..grid.size() {
        initial_max = initial_max.max(grid.e_field[i].length());
        initial_max = initial_max.max(grid.b_field[i].length());
    }

    let dt = 1e-12;

    // Run simulation
    for _ in 0..100 {
        maxwell.update_fields(&mut grid, dt);

        // Check that fields don't explode
        for i in 0..grid.size() {
            let e_mag = grid.e_field[i].length();
            let b_mag = grid.b_field[i].length();

            // Fields shouldn't grow by more than 10x (generous bound)
            assert!(
                e_mag < initial_max * 10.0,
                "E field exploded: {} > {}",
                e_mag,
                initial_max * 10.0
            );
            assert!(
                b_mag < initial_max * 10.0,
                "B field exploded: {} > {}",
                b_mag,
                initial_max * 10.0
            );
        }
    }
}
