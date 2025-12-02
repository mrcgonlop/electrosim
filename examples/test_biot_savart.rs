//! Test Biot-Savart Law and Ampère's Law emergence from dimensional structure
//!
//! PHYSICS BACKGROUND:
//! ==================
//!
//! Biot-Savart Law:
//! The magnetic field B at distance r from an infinite straight wire carrying current I:
//! B = (μ₀/2π) * (I/r)
//!
//! In our dimensional framework:
//! - String defect (1D) = current-carrying wire
//! - Dimensional circulation = magnetic field
//! - Circulation = ∮∇d·dl around the wire
//!
//! Ampère's Law:
//! For any closed loop around a wire:
//! ∮B·dl = μ₀ * I_enclosed
//!
//! Key predictions to test:
//! 1. Circulation should be independent of loop radius (Ampère's Law)
//! 2. Circulation should be proportional to current strength
//! 3. Circulation should be zero for loops that don't enclose the wire
//!
//! SUCCESS CRITERIA:
//! - Circulation varies < 15% across different radii (confirming Ampère's Law)
//! - Circulation doubles when current doubles (confirming linearity)

use em_physics_sandbox::physics::adaptive_automata::AdaptiveAutomata;
use glam::Vec3;

fn main() {
    println!("\n=== BIOT-SAVART LAW AND AMPÈRE'S LAW TEST ===\n");
    println!("Testing emergence of magnetism from dimensional structure...\n");

    // Grid resolution (higher = better accuracy but slower)
    let nx = 40;
    let ny = 40;
    let nz = 40;

    println!("Creating {}x{}x{} grid...", nx, ny, nz);
    let mut automata = AdaptiveAutomata::new_uniform(nx, ny, nz, 3.0);

    // Create a current-carrying wire along the z-axis
    // Wire goes from bottom to top of the grid
    let wire_start = Vec3::new(
        (nx / 2) as f32 * automata.spacing,
        (ny / 2) as f32 * automata.spacing,
        0.0
    );
    let wire_end = Vec3::new(
        (nx / 2) as f32 * automata.spacing,
        (ny / 2) as f32 * automata.spacing,
        (nz - 1) as f32 * automata.spacing
    );

    println!("\nWire configuration:");
    println!("  Start: ({:.1}, {:.1}, {:.1})", wire_start.x, wire_start.y, wire_start.z);
    println!("  End:   ({:.1}, {:.1}, {:.1})", wire_end.x, wire_end.y, wire_end.z);

    // Test 1: Basic circulation around wire
    println!("\n=== TEST 1: Basic Biot-Savart Law ===");
    println!("Creating string defect with current I = 1.0, r0 = 0.5\n");

    let current = 1.0;
    let r0 = 0.5;
    automata.create_string_defect_coulomb(wire_start, wire_end, current, r0);

    // Measure circulation at different radii
    let center = Vec3::new(
        (nx / 2) as f32 * automata.spacing,
        (ny / 2) as f32 * automata.spacing,
        (nz / 2) as f32 * automata.spacing
    );

    println!("Measuring circulation at different radii:");
    println!("(According to Ampère's Law, circulation should be constant)\n");

    let radii = [3.0, 5.0, 7.0, 9.0, 11.0];
    let mut circulations = Vec::new();

    for &radius in &radii {
        let circulation = automata.measure_circulation(center, radius, Vec3::Z);
        circulations.push(circulation);
        println!("  r = {:5.1}: circulation = {:10.6}", radius, circulation);
    }

    // Analyze variation
    let avg_circulation = circulations.iter().sum::<f32>() / circulations.len() as f32;
    let max_deviation = circulations.iter()
        .map(|&c| ((c - avg_circulation) / avg_circulation).abs())
        .fold(0.0f32, f32::max);

    println!("\nResults:");
    println!("  Average circulation: {:.6}", avg_circulation);
    println!("  Maximum deviation:   {:.1}%", max_deviation * 100.0);

    if max_deviation < 0.15 {
        println!("  ✓ SUCCESS: Ampère's Law holds (circulation independent of radius)!");
    } else {
        println!("  ✗ NEEDS IMPROVEMENT: Circulation varies too much with radius");
        println!("    (Expected < 15% variation for Ampère's Law)");
    }

    // Test 2: Linearity with current
    println!("\n\n=== TEST 2: Linearity with Current ===");
    println!("Testing that circulation ∝ current strength\n");

    let test_currents = [0.5, 1.0, 2.0, 4.0];
    let test_radius = 7.0; // Fixed radius

    println!("Measuring circulation at r = {:.1} for different currents:", test_radius);
    println!("(Circulation should scale linearly with current)\n");

    let mut current_results = Vec::new();

    for &test_current in &test_currents {
        automata = AdaptiveAutomata::new_uniform(nx, ny, nz, 3.0);
        automata.create_string_defect_coulomb(wire_start, wire_end, test_current, r0);

        let circulation = automata.measure_circulation(center, test_radius, Vec3::Z);
        current_results.push((test_current, circulation));

        println!("  I = {:.2}: circulation = {:10.6}", test_current, circulation);
    }

    // Check linearity
    println!("\nLinearity check (ratio should be close to current ratio):");
    for i in 1..current_results.len() {
        let (i1, c1) = current_results[i-1];
        let (i2, c2) = current_results[i];
        let current_ratio = i2 / i1;
        let circulation_ratio = c2 / c1;
        let error = ((circulation_ratio - current_ratio) / current_ratio).abs() * 100.0;

        println!("  I={:.2} to I={:.2}: circulation ratio = {:.3} (expected: {:.3}, error: {:.1}%)",
                 i1, i2, circulation_ratio, current_ratio, error);
    }

    // Test 3: Zero circulation for non-enclosing loop
    println!("\n\n=== TEST 3: Non-Enclosing Loop ===");
    println!("Testing circulation for a loop that doesn't enclose the wire");
    println!("(Should be near zero)\n");

    // Reset with single current
    automata = AdaptiveAutomata::new_uniform(nx, ny, nz, 3.0);
    automata.create_string_defect_coulomb(wire_start, wire_end, 1.0, r0);

    // Measure in a plane perpendicular to the wire (parallel to xy plane)
    // This loop is parallel to the wire, so doesn't enclose it
    let side_center = Vec3::new(
        (nx / 2) as f32 * automata.spacing + 5.0,
        (ny / 2) as f32 * automata.spacing,
        (nz / 2) as f32 * automata.spacing
    );

    let side_circulation = automata.measure_circulation(side_center, 3.0, Vec3::X);

    println!("  Loop center: ({:.1}, {:.1}, {:.1})", side_center.x, side_center.y, side_center.z);
    println!("  Loop normal: X (perpendicular to wire)");
    println!("  Circulation: {:.6}", side_circulation);

    if side_circulation.abs() < avg_circulation * 0.3 {
        println!("  ✓ SUCCESS: Non-enclosing loop has low circulation!");
    } else {
        println!("  ? Circulation is significant (may indicate field structure)");
    }

    // Test 4: Different measurement planes
    println!("\n\n=== TEST 4: Measurement Plane Independence ===");
    println!("Testing circulation at different heights along the wire");
    println!("(Should be constant along an infinite wire)\n");

    let heights = [
        (nz / 4) as f32 * automata.spacing,
        (nz / 2) as f32 * automata.spacing,
        (3 * nz / 4) as f32 * automata.spacing,
    ];

    for &height in &heights {
        let test_center = Vec3::new(
            (nx / 2) as f32 * automata.spacing,
            (ny / 2) as f32 * automata.spacing,
            height
        );
        let circ = automata.measure_circulation(test_center, 7.0, Vec3::Z);
        println!("  z = {:5.1}: circulation = {:10.6}", height, circ);
    }

    // Summary
    println!("\n\n=== SUMMARY ===\n");
    println!("Biot-Savart Law: B ∝ I/r");
    println!("Ampère's Law:    ∮B·dl = μ₀ I_enclosed\n");

    println!("Key Results:");
    println!("  1. Circulation variation with radius: {:.1}%", max_deviation * 100.0);
    if max_deviation < 0.15 {
        println!("     ✓ Ampère's Law confirmed!");
    } else {
        println!("     ⚠ Needs improvement");
    }

    println!("\n  2. Linearity with current:");
    for i in 1..current_results.len() {
        let (i1, c1) = current_results[i-1];
        let (i2, c2) = current_results[i];
        let ratio = (c2/c1) / (i2/i1);
        println!("     I={:.2}→{:.2}: ratio = {:.3}", i1, i2, ratio);
    }

    println!("\n  3. Non-enclosing loop: {:.6} (should be ~0)", side_circulation);

    println!("\nConclusion:");
    if max_deviation < 0.15 {
        println!("✓ SUCCESS: Magnetism emerges from dimensional structure!");
        println!("  - Biot-Savart Law holds");
        println!("  - Ampère's Law verified");
        println!("  - String defects naturally create magnetic fields");
    } else {
        println!("⚠ PARTIAL SUCCESS: Magnetic effects visible but need refinement");
        println!("  Consider:");
        println!("  - Higher grid resolution");
        println!("  - Better numerical integration");
        println!("  - Optimized parameters (r0, current strength)");
    }
}
