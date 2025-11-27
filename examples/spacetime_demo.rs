//! Spacetime Automaton Demo
//!
//! Demonstrates the new unified spacetime cell architecture where
//! particles, fields, and fluids coexist in the same substrate.
//!
//! Run with: cargo run --example spacetime_demo --release

use em_physics_sandbox::simulation::{SpacetimeNetwork, InteractionKind};
use glam::Vec3;

fn main() {
    println!("Spacetime Automaton Demo");
    println!("========================\n");

    // Create a 32³ spacetime network with 1cm spacing
    let mut network = SpacetimeNetwork::new(32, 32, 32, 0.01);

    println!("Network created:");
    println!("  Cells: {}", network.cells.len());
    println!("  Grid: {}×{}×{}", network.nx, network.ny, network.nz);
    println!("  Spacing: {:.3} m\n", network.spacing);

    // === Demo 1: Add two charged particles ===
    println!("Demo 1: Creating two charged particles");
    println!("---------------------------------------");

    let particle1_idx = network.index(10, 16, 16);
    let particle2_idx = network.index(22, 16, 16);

    // Set up particles with opposite charges
    network.cells[particle1_idx].set_point_charge(1e-15);  // Positive
    network.cells[particle2_idx].set_point_charge(-1e-15); // Negative

    println!("Particle 1: index {}, charge {:.2e} C",
             particle1_idx,
             network.cells[particle1_idx].charge_density * network.cells[particle1_idx].volume);
    println!("Particle 2: index {}, charge {:.2e} C\n",
             particle2_idx,
             network.cells[particle2_idx].charge_density * network.cells[particle2_idx].volume);

    // === Demo 2: Create Weber graph ===
    println!("Demo 2: Creating Weber interaction graph");
    println!("-----------------------------------------");

    network.update_weber_graph();

    println!("Weber edges created: {}", network.nonlocal_edges.len());
    if !network.nonlocal_edges.is_empty() {
        let edge = &network.nonlocal_edges[0];
        println!("  Edge 0:");
        println!("    Source: {}", edge.source);
        println!("    Target: {}", edge.target);
        println!("    Distance: {:.4} m", edge.distance);
        println!("    Coupling: {:.2e}", edge.coupling);
        println!("    Kind: {:?}\n", edge.kind);
    }

    // === Demo 3: Initialize EM plane wave ===
    println!("Demo 3: Initializing EM plane wave");
    println!("-----------------------------------");

    let wave_direction = Vec3::new(1.0, 0.0, 0.0).normalize();
    let amplitude = 1000.0; // V/m
    let wavelength = 0.01; // 1 cm

    network.initialize_plane_wave(wave_direction, amplitude, wavelength);

    // Sample field at a few points
    let sample_indices = [
        network.index(5, 16, 16),
        network.index(16, 16, 16),
        network.index(27, 16, 16),
    ];

    println!("E field samples:");
    for &idx in &sample_indices {
        let (i, j, k) = network.coords(idx);
        let e = network.cells[idx].e_field;
        println!("  ({}, {}, {}): E = ({:.1}, {:.1}, {:.1}) V/m",
                 i, j, k, e.x, e.y, e.z);
    }
    println!();

    // === Demo 4: Add Gaussian charge distribution ===
    println!("Demo 4: Adding Gaussian charge distribution");
    println!("--------------------------------------------");

    let center = Vec3::new(0.16, 0.16, 0.16); // Center of grid
    let total_charge = 1e-12; // 1 pC
    let sigma = 0.03; // 3 cm spread

    network.add_gaussian_charge_distribution(center, total_charge, sigma);

    println!("Gaussian distribution added:");
    println!("  Center: ({:.2}, {:.2}, {:.2})", center.x, center.y, center.z);
    println!("  Total charge: {:.2e} C", total_charge);
    println!("  Sigma: {:.3} m\n", sigma);

    // === Demo 5: Set up conductor region ===
    println!("Demo 5: Creating conductor region (Faraday cage)");
    println!("------------------------------------------------");

    // Create hollow conducting box
    network.set_conductor_region(|pos| {
        let x = pos.x;
        let y = pos.y;
        let z = pos.z;

        // Walls at boundaries
        let thickness = 0.02;
        (x < thickness || x > 0.30 ||
         y < thickness || y > 0.30 ||
         z < thickness || z > 0.30) &&
        !(x < 0.0 || x > 0.31 || y < 0.0 || y > 0.31 || z < 0.0 || z > 0.31)
    });

    let conductor_count = network.cells.iter().filter(|c| c.is_conductor()).count();
    println!("Conductor cells: {}\n", conductor_count);

    // === Demo 6: Network statistics ===
    println!("Demo 6: Network statistics");
    println!("--------------------------");

    let stats = network.stats();
    println!("Total cells: {}", stats.total_cells);
    println!("Particles: {}", stats.particle_count);
    println!("Non-local edges: {}", stats.nonlocal_edge_count);
    println!("Total energy: {:.2e} J", stats.total_energy);
    println!("Total charge: {:.2e} C", stats.total_charge);
    println!("Time: {:.2e} s\n", stats.time);

    // === Demo 7: Time evolution ===
    println!("Demo 7: Time evolution");
    println!("----------------------");

    println!("Advancing time...");
    for step in 1..=5 {
        network.advance_time(1e-12);
        println!("  Step {}: t = {:.2e} s", step, network.time);
    }
    println!();

    // === Demo 8: Energy and momentum ===
    println!("Demo 8: Conservation quantities");
    println!("--------------------------------");

    let total_energy = network.total_energy();
    let total_momentum = network.total_momentum();
    let total_charge = network.total_charge();

    println!("Total energy: {:.6e} J", total_energy);
    println!("Total momentum: ({:.2e}, {:.2e}, {:.2e}) kg·m/s",
             total_momentum.x, total_momentum.y, total_momentum.z);
    println!("Total charge: {:.6e} C (should be conserved)\n", total_charge);

    // === Summary ===
    println!("Summary");
    println!("=======");
    println!("✓ Created unified spacetime network");
    println!("✓ Particles represented as localized charge densities");
    println!("✓ EM fields coexist with particles");
    println!("✓ Non-local Weber graph automatically generated");
    println!("✓ Conductor regions defined");
    println!("✓ Conservation quantities computed\n");

    println!("Next steps:");
    println!("  1. Implement update rules (Maxwell + Fluid + Weber)");
    println!("  2. Time-step the simulation");
    println!("  3. Add visualization");
    println!("  4. Compare with pure FDTD and particle methods");
}
