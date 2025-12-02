//! Electromagnetic Emergence Tests
//!
//! NEW PHILOSOPHY:
//! - Don't assume Maxwell/Weber equations
//! - Evolve hypergraph with simple rules
//! - Measure forces via dimensional gradients in automata
//! - Compare to historical experiments via NULL HYPOTHESIS
//!
//! Null Hypothesis: "Forces emerge from dimensional structure alone"
//!
//! Each test:
//! 1. Create graph with defects (particles, currents, etc.)
//! 2. Embed \u2192 Automata (dimensional field)
//! 3. Measure forces via dimension gradients
//! 4. Compare to experimental ratios

use crate::physics::{Hypergraph, GraphEmbedding, AdaptiveAutomata};
use glam::Vec3;
use std::collections::HashMap;

/// Test result comparing emergent behavior to experiments
#[derive(Debug, Clone)]
pub struct EmergenceTestResult {
    pub test_name: String,
    pub passed: bool,

    pub measured_ratio: f32,
    pub expected_ratio: f32,
    pub relative_error: f32,

    pub details: HashMap<String, f32>,
}

impl EmergenceTestResult {
    pub fn new(name: &str, measured: f32, expected: f32, tolerance: f32) -> Self {
        let relative_error = ((measured - expected) / expected).abs();
        let passed = relative_error < tolerance;

        Self {
            test_name: name.to_string(),
            passed,
            measured_ratio: measured,
            expected_ratio: expected,
            relative_error,
            details: HashMap::new(),
        }
    }

    pub fn with_detail(mut self, key: &str, value: f32) -> Self {
        self.details.insert(key.to_string(), value);
        self
    }
}

// ============================================================================
// FORCE MEASUREMENT VIA DIMENSIONAL GRADIENTS
// ============================================================================

/// Measure force between two positions via dimensional tension
///
/// Force = -∇U where U = ∫ (d - d_background)² dV
///
/// This is the KEY PRINCIPLE: dimensional defects create "tension"
/// that manifests as observable forces!
pub fn measure_dimensional_force(
    automata: &AdaptiveAutomata,
    pos1: Vec3,
    pos2: Vec3,
) -> f32 {
    let direction = (pos2 - pos1).normalize();
    let distance = (pos2 - pos1).length();

    let mut total_force = 0.0;

    // Sample along line connecting the two positions
    let samples = 50;
    for i in 0..samples {
        let t = i as f32 / samples as f32;
        let pos = pos1.lerp(pos2, t);

        // Dimensional gradient at this point
        let grad = automata.dimension_gradient_at(pos);

        // Project gradient onto direction
        let force_component = grad.dot(direction);

        // Dimensional tension contributes to force
        let dim = automata.dimension_at(pos);
        let tension = (dim - automata.background_dimension).powi(2);

        total_force += force_component * tension * (distance / samples as f32);
    }

    total_force.abs()
}

/// Measure circulation of dimensional field around a loop
///
/// This tests Ampère's law: ∮ B·dl = μ₀ I
/// In our framework: circulation around string defect (current)
pub fn measure_dimensional_circulation(
    automata: &AdaptiveAutomata,
    center: Vec3,
    radius: f32,
    axis: Vec3,
) -> f32 {
    let mut circulation = 0.0;
    let samples = 32;

    // Get two perpendicular vectors to axis
    let perp1 = if axis.x.abs() < 0.9 {
        axis.cross(Vec3::X).normalize()
    } else {
        axis.cross(Vec3::Y).normalize()
    };
    let perp2 = axis.cross(perp1).normalize();

    for i in 0..samples {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / samples as f32;
        let angle_next = 2.0 * std::f32::consts::PI * (i + 1) as f32 / samples as f32;

        let pos = center + perp1 * radius * angle.cos() + perp2 * radius * angle.sin();
        let pos_next = center + perp1 * radius * angle_next.cos() + perp2 * radius * angle_next.sin();

        let tangent = (pos_next - pos).normalize();
        let grad = automata.dimension_gradient_at(pos);

        circulation += grad.dot(tangent) * (pos_next - pos).length();
    }

    circulation
}

/// Measure flux of dimensional field through surface
///
/// Tests Gauss's law: ∮ E·dA = Q/ε₀
/// In our framework: flux from particle defect
pub fn measure_dimensional_flux(
    automata: &AdaptiveAutomata,
    center: Vec3,
    radius: f32,
) -> f32 {
    let mut flux = 0.0;
    let theta_samples = 16;
    let phi_samples = 32;

    for i in 0..theta_samples {
        for j in 0..phi_samples {
            let theta = std::f32::consts::PI * i as f32 / theta_samples as f32;
            let phi = 2.0 * std::f32::consts::PI * j as f32 / phi_samples as f32;

            let normal = Vec3::new(
                theta.sin() * phi.cos(),
                theta.sin() * phi.sin(),
                theta.cos(),
            );

            let pos = center + normal * radius;
            let grad = automata.dimension_gradient_at(pos);

            let d_area = radius * radius * theta.sin() *
                        (std::f32::consts::PI / theta_samples as f32) *
                        (2.0 * std::f32::consts::PI / phi_samples as f32);

            flux += grad.dot(normal) * d_area;
        }
    }

    flux
}

// ============================================================================
// EXPERIMENTAL TESTS
// ============================================================================

/// Test 1: Coulomb's Law (Inverse Square)
///
/// Historical: Cavendish (1773), Coulomb (1785)
/// Method: Two particle defects at different distances
/// Null hypothesis: F ∝ 1/r² emerges from d=3 space
pub fn test_coulomb_law() -> EmergenceTestResult {
    println!("\n--- Test 1: Coulomb's Law ---");

    // Create automata with two particle defects
    let mut automata = AdaptiveAutomata::new_uniform(50, 50, 50, 3.0);

    let pos1 = Vec3::new(15.0, 25.0, 25.0);
    let pos2_near = Vec3::new(25.0, 25.0, 25.0);   // Distance = 10
    let pos2_far = Vec3::new(35.0, 25.0, 25.0);    // Distance = 20

    automata.create_particle_defect(pos1, 2.0);

    // Measure force at near distance
    automata.create_particle_defect(pos2_near, 2.0);
    let force_near = measure_dimensional_force(&automata, pos1, pos2_near);
    println!("  Force at r=10: {:.4}", force_near);

    // Reset and measure at far distance
    automata = AdaptiveAutomata::new_uniform(50, 50, 50, 3.0);
    automata.create_particle_defect(pos1, 2.0);
    automata.create_particle_defect(pos2_far, 2.0);
    let force_far = measure_dimensional_force(&automata, pos1, pos2_far);
    println!("  Force at r=20: {:.4}", force_far);

    // Ratio test: F(r) / F(2r) should be 4.0 for 1/r²
    let measured_ratio = force_near / force_far;
    let expected_ratio = 4.0;

    println!("  Measured ratio: {:.2}", measured_ratio);
    println!("  Expected ratio: {:.2}", expected_ratio);

    EmergenceTestResult::new("Coulomb's Law", measured_ratio, expected_ratio, 0.2)
        .with_detail("force_near", force_near)
        .with_detail("force_far", force_far)
}

/// Test 2: Biot-Savart Law (Magnetic Field from Current)
///
/// Historical: Biot and Savart (1820)
/// Method: String defect (current) and measure field at distance
/// Null hypothesis: B ∝ 1/r emerges from string defect
pub fn test_biot_savart() -> EmergenceTestResult {
    println!("\n--- Test 2: Biot-Savart Law ---");

    // String defect = current (1D structure moving through 3D space)
    let mut automata = AdaptiveAutomata::new_uniform(50, 50, 50, 3.0);

    let current_start = Vec3::new(25.0, 10.0, 25.0);
    let current_end = Vec3::new(25.0, 40.0, 25.0);
    automata.create_string_defect(current_start, current_end, 1.5);

    // Measure circulation at two different radii
    let center = Vec3::new(25.0, 25.0, 25.0);
    let axis = Vec3::Y;

    let circ_near = measure_dimensional_circulation(&automata, center, 5.0, axis);
    let circ_far = measure_dimensional_circulation(&automata, center, 10.0, axis);

    println!("  Circulation at r=5: {:.4}", circ_near);
    println!("  Circulation at r=10: {:.4}", circ_far);

    // For current-carrying wire: B ∝ 1/r
    // So circulation at same radius should be constant
    // But at different radii, circulation scales with enclosed current
    let measured_ratio = circ_near / circ_far;
    let expected_ratio = 1.0;  // Both should see same enclosed current

    println!("  Measured ratio: {:.2}", measured_ratio);
    println!("  Expected ratio: {:.2}", expected_ratio);

    EmergenceTestResult::new("Biot-Savart Law", measured_ratio, expected_ratio, 0.2)
        .with_detail("circ_near", circ_near)
        .with_detail("circ_far", circ_far)
}

/// Test 3: Gauss's Law (Electric Flux)
///
/// Historical: Gauss (1835)
/// Method: Particle defect and measure flux through spheres
/// Null hypothesis: Flux independent of radius (charge conservation)
pub fn test_gauss_law() -> EmergenceTestResult {
    println!("\n--- Test 3: Gauss's Law ---");

    let mut automata = AdaptiveAutomata::new_uniform(50, 50, 50, 3.0);

    let charge_pos = Vec3::new(25.0, 25.0, 25.0);
    automata.create_particle_defect(charge_pos, 2.0);

    // Measure flux through two concentric spheres
    let flux_small = measure_dimensional_flux(&automata, charge_pos, 5.0);
    let flux_large = measure_dimensional_flux(&automata, charge_pos, 10.0);

    println!("  Flux through r=5 sphere: {:.4}", flux_small);
    println!("  Flux through r=10 sphere: {:.4}", flux_large);

    // Gauss: flux should be same (charge conservation)
    let measured_ratio = flux_small / flux_large;
    let expected_ratio = 1.0;

    println!("  Measured ratio: {:.2}", measured_ratio);
    println!("  Expected ratio: {:.2}", expected_ratio);

    EmergenceTestResult::new("Gauss's Law", measured_ratio, expected_ratio, 0.15)
        .with_detail("flux_small", flux_small)
        .with_detail("flux_large", flux_large)
}

/// Test Suite: Run all emergence tests
pub fn run_all_emergence_tests() -> Vec<EmergenceTestResult> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  ELECTROMAGNETIC EMERGENCE TEST SUITE                        ║");
    println!("║  Testing null hypothesis: Forces emerge from dimension      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    let tests: Vec<Box<dyn Fn() -> EmergenceTestResult>> = vec![
        Box::new(test_coulomb_law),
        Box::new(test_biot_savart),
        Box::new(test_gauss_law),
    ];

    let mut results = Vec::new();

    for test in tests {
        let result = test();

        let status = if result.passed { "PASS ✓" } else { "FAIL ✗" };
        println!("\n{}: {} (error: {:.1}%)",
                 result.test_name,
                 status,
                 result.relative_error * 100.0);

        results.push(result);
    }

    // Summary
    println!("\n{}", "=".repeat(70));
    println!("SUMMARY");
    println!("{}", "=".repeat(70));

    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();

    println!("Passed: {}/{}", passed, total);
    println!("");

    if passed == total {
        println!("✓ NULL HYPOTHESIS CONFIRMED:");
        println!("  Electromagnetic forces emerge from dimensional structure!");
        println!("  No Maxwell/Weber equations needed at fundamental level.");
    } else {
        println!("⚠ NULL HYPOTHESIS NEEDS TUNING:");
        println!("  Adjust graph rules or defect parameters");
        println!("  Some forces don't match experimental ratios yet");
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_force_measurement() {
        let automata = AdaptiveAutomata::new_uniform(30, 30, 30, 3.0);
        let pos1 = Vec3::new(10.0, 15.0, 15.0);
        let pos2 = Vec3::new(20.0, 15.0, 15.0);

        let force = measure_dimensional_force(&automata, pos1, pos2);

        // In uniform space, force should be near zero
        assert!(force < 1.0);
    }

    #[test]
    fn test_flux_spherical() {
        let mut automata = AdaptiveAutomata::new_uniform(40, 40, 40, 3.0);
        let center = Vec3::new(20.0, 20.0, 20.0);

        automata.create_particle_defect(center, 2.0);

        let flux = measure_dimensional_flux(&automata, center, 5.0);

        // Should have non-zero flux from defect
        assert!(flux.abs() > 0.1);
    }
}
