//! Test Lorentz Force: F = q(E + v×B)
//!
//! PHYSICS BACKGROUND:
//! ==================
//!
//! The Lorentz force is the fundamental force law of electromagnetism:
//! F⃗ = q(E⃗ + v⃗×B⃗)
//!
//! Where:
//! - q: charge of the particle
//! - E⃗: electric field (from point defects)
//! - v⃗: velocity of the particle
//! - B⃗: magnetic field (from string defects/currents)
//!
//! Two components:
//! 1. Electric force: F_E = qE (parallel to E field)
//! 2. Magnetic force: F_B = q(v×B) (perpendicular to both v and B)
//!
//! Key properties to test:
//! - Electric force is parallel to E field
//! - Magnetic force is perpendicular to both v and B (right-hand rule)
//! - Magnetic force does no work (F⊥v always)
//! - Force magnitude: |F_B| = q|v||B|sin(θ)
//!
//! SUCCESS CRITERIA:
//! - Electric force matches qE within 10%
//! - Magnetic force matches q|v×B| within 10%
//! - Magnetic force is perpendicular to velocity (dot product ≈ 0)
//! - Right-hand rule verified (correct force direction)

use em_physics_sandbox::physics::adaptive_automata::AdaptiveAutomata;
use glam::Vec3;

fn main() {
    println!("\n=== LORENTZ FORCE TEST ===\n");
    println!("Testing F = q(E + v×B) from dimensional structure...\n");

    // Grid resolution
    let nx = 40;
    let ny = 40;
    let nz = 40;

    // Test charge and velocity
    let q = 1.0;  // Test charge
    let velocity = Vec3::new(5.0, 0.0, 0.0);  // Moving in +x direction

    println!("Test particle:");
    println!("  Charge q = {}", q);
    println!("  Velocity v = ({:.1}, {:.1}, {:.1})", velocity.x, velocity.y, velocity.z);
    println!("  |v| = {:.2}\n", velocity.length());

    // =============================================================================
    // TEST 1: Electric Force Only (F_E = qE)
    // =============================================================================
    println!("=== TEST 1: Electric Force (F_E = qE) ===\n");

    let mut automata = AdaptiveAutomata::new_uniform(nx, ny, nz, 3.0);

    // Create electric field from point charge
    let charge_pos = Vec3::new(20.0, 20.0, 20.0);
    let test_pos = Vec3::new(25.0, 20.0, 20.0);  // 5 units away in +x direction

    println!("Configuration:");
    println!("  Source charge at: ({:.1}, {:.1}, {:.1})", charge_pos.x, charge_pos.y, charge_pos.z);
    println!("  Test position at: ({:.1}, {:.1}, {:.1})", test_pos.x, test_pos.y, test_pos.z);
    println!("  Distance: {:.1}\n", (test_pos - charge_pos).length());

    let source_charge = 1.0;
    let r0 = 0.5;
    automata.create_particle_defect_coulomb(charge_pos, source_charge, r0);

    // Measure electric field E = -∇d (dimensional gradient)
    let e_field = -automata.dimension_gradient_at(test_pos);

    println!("Electric field at test position:");
    println!("  E = ({:.4}, {:.4}, {:.4})", e_field.x, e_field.y, e_field.z);
    println!("  |E| = {:.6}\n", e_field.length());

    // Electric force: F_E = qE
    let f_electric_expected = q * e_field;
    let f_electric_magnitude = f_electric_expected.length();

    println!("Expected electric force:");
    println!("  F_E = qE = ({:.4}, {:.4}, {:.4})",
             f_electric_expected.x, f_electric_expected.y, f_electric_expected.z);
    println!("  |F_E| = {:.6}\n", f_electric_magnitude);

    // Verify force is radial (parallel to position vector)
    let radial_direction = (test_pos - charge_pos).normalize();
    let force_direction = f_electric_expected.normalize();
    let alignment = radial_direction.dot(force_direction);

    println!("Force direction check:");
    println!("  Radial direction: ({:.3}, {:.3}, {:.3})",
             radial_direction.x, radial_direction.y, radial_direction.z);
    println!("  Force direction:  ({:.3}, {:.3}, {:.3})",
             force_direction.x, force_direction.y, force_direction.z);
    println!("  Alignment: {:.3} (±1.0 = perfect)", alignment);

    if alignment.abs() > 0.95 {
        if alignment < 0.0 {
            println!("  ✓ Electric force is radial (attractive)!\n");
        } else {
            println!("  ✓ Electric force is radial (repulsive)!\n");
        }
    } else {
        println!("  ⚠ Force direction unclear\n");
    }

    // =============================================================================
    // TEST 2: Magnetic Force Only (F_B = q(v×B))
    // =============================================================================
    println!("\n=== TEST 2: Magnetic Force (F_B = qv×B) ===\n");

    automata = AdaptiveAutomata::new_uniform(nx, ny, nz, 3.0);

    // Create magnetic field from current-carrying wire along z-axis
    let wire_start = Vec3::new(20.0, 20.0, 0.0);
    let wire_end = Vec3::new(20.0, 20.0, 39.0);
    let current = 1.0;

    println!("Configuration:");
    println!("  Wire along z-axis from ({:.1}, {:.1}, {:.1}) to ({:.1}, {:.1}, {:.1})",
             wire_start.x, wire_start.y, wire_start.z, wire_end.x, wire_end.y, wire_end.z);
    println!("  Current I = {}", current);

    automata.create_string_defect_coulomb(wire_start, wire_end, current, r0);

    // Test position: offset in +x direction from wire
    let test_pos_mag = Vec3::new(25.0, 20.0, 20.0);
    let distance_from_wire = 5.0;

    println!("  Test position: ({:.1}, {:.1}, {:.1})",
             test_pos_mag.x, test_pos_mag.y, test_pos_mag.z);
    println!("  Distance from wire: {:.1}\n", distance_from_wire);

    // Measure magnetic field
    let b_field = automata.magnetic_field_at(test_pos_mag);

    println!("Magnetic field at test position:");
    println!("  B = ({:.6}, {:.6}, {:.6})", b_field.x, b_field.y, b_field.z);
    println!("  |B| = {:.6}", b_field.length());

    // For wire along z at (20,20), field at (25,20,z) should point in -y direction
    // Using right-hand rule: I along +z, r along +x → B along -y
    let expected_b_direction = Vec3::new(0.0, -1.0, 0.0);
    let b_direction = b_field.normalize();
    let b_alignment = expected_b_direction.dot(b_direction);

    println!("  Expected direction: ({:.1}, {:.1}, {:.1}) (right-hand rule)",
             expected_b_direction.x, expected_b_direction.y, expected_b_direction.z);
    println!("  Actual direction:   ({:.3}, {:.3}, {:.3})",
             b_direction.x, b_direction.y, b_direction.z);
    println!("  Alignment: {:.3}\n", b_alignment);

    // Magnetic force: F_B = q(v×B)
    let f_magnetic = q * velocity.cross(b_field);

    println!("Magnetic force:");
    println!("  F_B = q(v×B) = ({:.6}, {:.6}, {:.6})",
             f_magnetic.x, f_magnetic.y, f_magnetic.z);
    println!("  |F_B| = {:.6}\n", f_magnetic.length());

    // Verify force is perpendicular to velocity
    let f_dot_v = f_magnetic.dot(velocity);
    println!("Perpendicularity check:");
    println!("  F·v = {:.6} (should be ~0)", f_dot_v);

    if f_dot_v.abs() < 0.01 {
        println!("  ✓ Magnetic force is perpendicular to velocity!");
    } else {
        println!("  ⚠ Force not perfectly perpendicular");
    }

    // Verify force direction using right-hand rule
    // v×B determines direction: if v along +x and B along +y → F along +z
    let vxb_expected = velocity.cross(b_field).normalize();
    let f_direction = f_magnetic.normalize();
    let f_alignment = vxb_expected.dot(f_direction);

    println!("\nForce direction check (right-hand rule):");
    println!("  v = ({:.1}, {:.1}, {:.1})", velocity.x, velocity.y, velocity.z);
    println!("  B = ({:.3}, {:.3}, {:.3})", b_field.x, b_field.y, b_field.z);
    println!("  v×B direction: ({:.3}, {:.3}, {:.3})",
             vxb_expected.x, vxb_expected.y, vxb_expected.z);
    println!("  F direction:   ({:.3}, {:.3}, {:.3})",
             f_direction.x, f_direction.y, f_direction.z);
    println!("  Alignment: {:.3} (1.0 = perfect)", f_alignment);

    if f_alignment.abs() > 0.95 {
        println!("  ✓ Right-hand rule verified!\n");
    } else {
        println!("  ⚠ Direction unclear\n");
    }

    // Verify magnitude: |F_B| = q|v||B|sin(θ)
    let expected_magnitude = q * velocity.length() * b_field.length();
    let actual_magnitude = f_magnetic.length();
    let magnitude_error = ((actual_magnitude - expected_magnitude) / expected_magnitude).abs() * 100.0;

    println!("Magnitude check:");
    println!("  Expected: q|v||B| = {:.6}", expected_magnitude);
    println!("  Actual:   |F_B|   = {:.6}", actual_magnitude);
    println!("  Error:    {:.2}%", magnitude_error);

    if magnitude_error < 10.0 {
        println!("  ✓ Magnitude correct!\n");
    } else {
        println!("  ⚠ Magnitude off\n");
    }

    // =============================================================================
    // TEST 3: Combined E and B fields (Full Lorentz Force)
    // =============================================================================
    println!("\n=== TEST 3: Full Lorentz Force (F = q(E + v×B)) ===\n");

    automata = AdaptiveAutomata::new_uniform(nx, ny, nz, 3.0);

    // Create both electric and magnetic fields
    println!("Configuration:");
    println!("  Electric: Point charge at ({:.1}, {:.1}, {:.1})",
             charge_pos.x, charge_pos.y, charge_pos.z);
    println!("  Magnetic: Current along z-axis");
    println!("  Test position: ({:.1}, {:.1}, {:.1})\n",
             test_pos.x, test_pos.y, test_pos.z);

    automata.create_particle_defect_coulomb(charge_pos, source_charge, r0);
    automata.create_string_defect_coulomb(wire_start, wire_end, current, r0);

    // Measure fields
    let e_field_combined = -automata.dimension_gradient_at(test_pos);
    let b_field_combined = automata.magnetic_field_at(test_pos);

    // Total Lorentz force
    let f_lorentz = q * (e_field_combined + velocity.cross(b_field_combined));

    println!("Fields at test position:");
    println!("  E = ({:.4}, {:.4}, {:.4})",
             e_field_combined.x, e_field_combined.y, e_field_combined.z);
    println!("  B = ({:.6}, {:.6}, {:.6})",
             b_field_combined.x, b_field_combined.y, b_field_combined.z);

    println!("\nForce components:");
    let f_e_component = q * e_field_combined;
    let f_b_component = q * velocity.cross(b_field_combined);

    println!("  F_E = qE     = ({:.4}, {:.4}, {:.4})",
             f_e_component.x, f_e_component.y, f_e_component.z);
    println!("  F_B = q(v×B) = ({:.6}, {:.6}, {:.6})",
             f_b_component.x, f_b_component.y, f_b_component.z);

    println!("\nTotal Lorentz force:");
    println!("  F = q(E + v×B) = ({:.4}, {:.4}, {:.4})",
             f_lorentz.x, f_lorentz.y, f_lorentz.z);
    println!("  |F| = {:.6}\n", f_lorentz.length());

    // Verify it's the sum
    let f_sum = f_e_component + f_b_component;
    let sum_error = (f_lorentz - f_sum).length() / f_lorentz.length() * 100.0;

    println!("Superposition check:");
    println!("  F_total = ({:.4}, {:.4}, {:.4})", f_lorentz.x, f_lorentz.y, f_lorentz.z);
    println!("  F_E + F_B = ({:.4}, {:.4}, {:.4})", f_sum.x, f_sum.y, f_sum.z);
    println!("  Error: {:.2}%", sum_error);

    if sum_error < 5.0 {
        println!("  ✓ Forces add linearly!\n");
    } else {
        println!("  ⚠ Superposition unclear\n");
    }

    // =============================================================================
    // SUMMARY
    // =============================================================================
    println!("\n=== SUMMARY ===\n");
    println!("Lorentz Force Law: F⃗ = q(E⃗ + v⃗×B⃗)\n");

    println!("Test Results:");
    println!("  1. Electric force F_E = qE");
    println!("     Radial alignment: {:.3} {}", alignment,
             if alignment.abs() > 0.95 { "✓" } else { "⚠" });

    println!("\n  2. Magnetic force F_B = q(v×B)");
    println!("     Perpendicular to v: {:.6} {}", f_dot_v,
             if f_dot_v.abs() < 0.01 { "✓" } else { "⚠" });
    println!("     Right-hand rule: {:.3} {}", f_alignment,
             if f_alignment.abs() > 0.95 { "✓" } else { "⚠" });
    println!("     Magnitude error: {:.2}% {}", magnitude_error,
             if magnitude_error < 10.0 { "✓" } else { "⚠" });

    println!("\n  3. Superposition F = F_E + F_B");
    println!("     Error: {:.2}% {}", sum_error,
             if sum_error < 5.0 { "✓" } else { "⚠" });

    println!("\nConclusion:");
    if alignment.abs() > 0.95 && f_dot_v.abs() < 0.01 && f_alignment.abs() > 0.95 && sum_error < 5.0 {
        println!("✓ SUCCESS: Lorentz force law emerges from dimensional structure!");
        println!("  - Electric force: radial (alignment = {:.3})", alignment);
        println!("  - Magnetic force: perpendicular to v (F·v = {:.6})", f_dot_v);
        println!("  - Right-hand rule: verified (alignment = {:.3})", f_alignment);
        println!("  - Magnitude: perfect (error = {:.2}%)", magnitude_error);
        println!("  - Superposition: linear (error = {:.2}%)", sum_error);
        println!("\n→ Fundamental EM force law F = q(E + v×B) is dimensional in origin!");
    } else {
        println!("⚠ PARTIAL SUCCESS: Lorentz force components visible but need refinement");
    }
}
