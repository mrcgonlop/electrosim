//! Tune Dimensional Defect Parameters to Match Coulomb's Law
//!
//! Tests different charge and lambda values to find parameters
//! that produce F ∝ 1/r² force law

use em_physics_sandbox::physics::AdaptiveAutomata;
use em_physics_sandbox::physics::em_emergence_tests::measure_dimensional_force;
use glam::Vec3;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  TUNING DIMENSIONAL DEFECTS FOR COULOMB'S LAW               ║");
    println!("║  Goal: Find parameters that give F(r)/F(2r) ≈ 4.0          ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Grid of parameters to test
    let charge_values = vec![0.5, 1.0, 1.5, 2.0, 2.5, 3.0];
    let lambda_values = vec![2.0, 3.0, 4.0, 5.0, 6.0];

    println!("Testing {} combinations...\n", charge_values.len() * lambda_values.len());

    let mut best_ratio = f32::INFINITY;
    let mut best_charge = 0.0;
    let mut best_lambda = 0.0;
    let mut best_force_near = 0.0;
    let mut best_force_far = 0.0;

    for &charge in &charge_values {
        for &lambda in &lambda_values {
            // Test this parameter combination
            let (ratio, force_near, force_far) = test_parameters(charge, lambda);

            // How close to ideal 4.0?
            let error = (ratio - 4.0).abs();

            print!("charge={:.1}, λ={:.1}: F_near={:.4}, F_far={:.4}, ratio={:.2}",
                   charge, lambda, force_near, force_far, ratio);

            if error < (best_ratio - 4.0).abs() && !ratio.is_nan() && !ratio.is_infinite() {
                best_ratio = ratio;
                best_charge = charge;
                best_lambda = lambda;
                best_force_near = force_near;
                best_force_far = force_far;
                println!("  ← BEST SO FAR!");
            } else {
                println!();
            }
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("BEST PARAMETERS");
    println!("{}", "=".repeat(70));
    println!("  Charge: {:.2}", best_charge);
    println!("  Lambda: {:.2}", best_lambda);
    println!("  Force at r=10: {:.4}", best_force_near);
    println!("  Force at r=20: {:.4}", best_force_far);
    println!("  Measured ratio: {:.2}", best_ratio);
    println!("  Expected ratio: 4.00");
    println!("  Error: {:.1}%", ((best_ratio - 4.0) / 4.0).abs() * 100.0);

    if (best_ratio - 4.0).abs() / 4.0 < 0.15 {
        println!("\n✓ SUCCESS! Within 15% of Coulomb's Law");
        println!("\nUse these parameters in your simulations:");
        println!("  automata.create_particle_defect_smooth(pos, {:.2}, {:.2});",
                 best_charge, best_lambda);
    } else {
        println!("\n⚠ Need better parameters. Try:");
        println!("  - Higher grid resolution (50x50x50 → 80x80x80)");
        println!("  - Different charge/lambda ranges");
        println!("  - Improved force measurement method");
    }
}

fn test_parameters(charge: f32, lambda: f32) -> (f32, f32, f32) {
    // Create automata with two particle defects at different distances
    let grid_size = 60;  // Higher resolution for better gradient measurement

    // Test at r=10
    let mut automata_near = AdaptiveAutomata::new_uniform(grid_size, grid_size, grid_size, 3.0);
    let pos1 = Vec3::new(20.0, 30.0, 30.0);
    let pos2_near = Vec3::new(30.0, 30.0, 30.0);  // Distance = 10

    automata_near.create_particle_defect_smooth(pos1, charge, lambda);
    automata_near.create_particle_defect_smooth(pos2_near, charge, lambda);

    let force_near = measure_dimensional_force(&automata_near, pos1, pos2_near);

    // Test at r=20
    let mut automata_far = AdaptiveAutomata::new_uniform(grid_size, grid_size, grid_size, 3.0);
    let pos2_far = Vec3::new(40.0, 30.0, 30.0);  // Distance = 20

    automata_far.create_particle_defect_smooth(pos1, charge, lambda);
    automata_far.create_particle_defect_smooth(pos2_far, charge, lambda);

    let force_far = measure_dimensional_force(&automata_far, pos1, pos2_far);

    // Compute ratio
    let ratio = if force_far > 1e-6 {
        force_near / force_far
    } else {
        f32::INFINITY
    };

    (ratio, force_near, force_far)
}
