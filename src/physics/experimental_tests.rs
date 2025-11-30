//! Experimental Test Suite for Hypergraph Physics
//!
//! Tests based on HISTORICAL NULL/EQUILIBRIUM EXPERIMENTS
//!
//! Philosophy: Map graph properties to observable ratios, not absolute values.
//! This avoids the need to assign coordinates or units.
//!
//! Each test:
//! 1. Sets up initial graph configuration
//! 2. Evolves until equilibrium
//! 3. Measures dimensionless ratios
//! 4. Compares to known physics

use crate::physics::hypergraph::{Hypergraph, RewriteRule};
use std::collections::HashMap;

/// Result of a physics test
#[derive(Debug, Clone)]
pub struct TestResult {
    pub passed: bool,
    pub measured_value: f32,
    pub expected_value: f32,
    pub relative_error: f32,
    pub details: HashMap<String, f32>,
}

impl TestResult {
    pub fn new(measured: f32, expected: f32, tolerance: f32) -> Self {
        let relative_error = ((measured - expected) / expected).abs();
        let passed = relative_error < tolerance;

        Self {
            passed,
            measured_value: measured,
            expected_value: expected,
            relative_error,
            details: HashMap::new(),
        }
    }

    pub fn with_detail(mut self, key: &str, value: f32) -> Self {
        self.details.insert(key.to_string(), value);
        self
    }
}

/// Trait for all physics tests
pub trait PhysicsTest {
    /// Name of the test
    fn name(&self) -> &str;

    /// Historical experiment this is based on
    fn historical_basis(&self) -> &str;

    /// Run the test with a given rewrite rule
    fn run(&self, rule: &RewriteRule) -> TestResult;

    /// Importance weight (0.0 - 1.0)
    fn weight(&self) -> f32 {
        1.0
    }

    /// Expected tolerance for passing
    fn tolerance(&self) -> f32 {
        0.1  // 10% by default
    }
}

//
// ============================================================================
// CATEGORY A: ELECTROMAGNETISM TESTS (NULL METHOD)
// ============================================================================
//

/// A1: Coulomb Balance Experiment (Cavendish 1773)
///
/// Historical: Cavendish used conducting sphere to test inverse-square law
/// Null method: Charge on inner vs outer sphere when connected
///
/// Hypergraph: Two charged nodes, measure force balance ratio
pub struct CoulombBalanceTest {
    pub charge_ratio: f32,
    pub distance_ratio: f32,
}

impl PhysicsTest for CoulombBalanceTest {
    fn name(&self) -> &str {
        "A1: Coulomb Balance (Inverse Square Law)"
    }

    fn historical_basis(&self) -> &str {
        "Cavendish (1773): Concentric spheres - no charge on inner when connected.
        Tests F ∝ 1/r² via null detection."
    }

    fn run(&self, rule: &RewriteRule) -> TestResult {
        let mut graph = Hypergraph::new();

        // Create two charged nodes
        let q1 = graph.add_node();
        let q2 = graph.add_node();

        graph.set_scalar(q1, "charge", 1.0);
        graph.set_scalar(q2, "charge", self.charge_ratio);

        // Add test node at two different distances
        let test_node_near = graph.add_node();
        let test_node_far = graph.add_node();

        // Set graph distances (via edge structure)
        graph.set_graph_separation(q1, test_node_near, 10);
        graph.set_graph_separation(q1, test_node_far, 20);

        // Evolve to equilibrium
        for _ in 0..10000 {
            graph.apply_rule_public(rule);
            if graph.is_at_equilibrium(1e-6) {
                break;
            }
        }

        // Measure force ratio
        let f_near = graph.measure_interaction_strength(q1, test_node_near);
        let f_far = graph.measure_interaction_strength(q1, test_node_far);

        let force_ratio = f_near / f_far;

        // Expected: F ∝ 1/r² → F(r₁)/F(r₂) = (r₂/r₁)²
        let distance_ratio: f32 = 20.0 / 10.0;  // 2.0
        let expected_ratio = distance_ratio.powi(2);  // 4.0

        TestResult::new(force_ratio, expected_ratio, self.tolerance())
            .with_detail("force_near", f_near)
            .with_detail("force_far", f_far)
            .with_detail("distance_ratio", distance_ratio)
    }
}

/// A2: Ampere Force Balance (1820)
///
/// Historical: Two parallel wires with adjustable currents
/// Null method: Adjust I₂ until force on wire 1 = 0
///
/// Hypergraph: Two "current loops" (circulating field patterns)
pub struct AmpereForceBalanceTest {
    pub wire_separation: usize,
}

impl PhysicsTest for AmpereForceBalanceTest {
    fn name(&self) -> &str {
        "A2: Ampere Force Balance (Parallel Currents)"
    }

    fn historical_basis(&self) -> &str {
        "Ampère (1820): Parallel wires attract if currents same direction.
        Force ∝ I₁·I₂/r. Null method: balance gravitational vs EM force."
    }

    fn run(&self, rule: &RewriteRule) -> TestResult {
        let mut graph = Hypergraph::new_2d_lattice(100, 100);

        // Create two "current loops" (vorticity patterns)
        let wire1_center = graph.node_at(25, 50);
        let wire2_center = graph.node_at(75, 50);

        // Set circulating field (current analog)
        graph.set_circulation(wire1_center, 10.0, 5);
        graph.set_circulation(wire2_center, 10.0, 5);

        // Evolve
        for _ in 0..5000 {
            graph.apply_rule_public(rule);
        }

        // Measure force between wires (via momentum transfer)
        let force_12 = graph.measure_net_flux(wire1_center, wire2_center);

        // Now reverse current in wire 2
        graph.set_circulation(wire2_center, -10.0, 5);

        for _ in 0..5000 {
            graph.apply_rule_public(rule);
        }

        let force_12_reversed = graph.measure_net_flux(wire1_center, wire2_center);

        // Expected: force should reverse sign (attraction ↔ repulsion)
        let ratio = force_12_reversed / force_12;
        let expected_ratio = -1.0;

        TestResult::new(ratio, expected_ratio, 0.2)
            .with_detail("force_same_dir", force_12)
            .with_detail("force_opp_dir", force_12_reversed)
    }
}

/// A3: Faraday Induction Balance (1831)
///
/// Historical: Changing flux through loop induces EMF
/// Null method: Balance induced EMF against known voltage
///
/// Hypergraph: Changing vorticity (B-analog) induces circulation gradient (E-analog)
pub struct FaradayInductionBalanceTest;

impl PhysicsTest for FaradayInductionBalanceTest {
    fn name(&self) -> &str {
        "A3: Faraday Induction Balance (EMF = -dΦ/dt)"
    }

    fn historical_basis(&self) -> &str {
        "Faraday (1831): Changing magnetic flux induces current.
        Null method: Balance induced current against external voltage."
    }

    fn run(&self, rule: &RewriteRule) -> TestResult {
        let mut graph = Hypergraph::new_2d_lattice(64, 64);

        // Create loop (closed path in graph)
        let loop_nodes = graph.create_closed_loop((32, 32), 10);

        // Measure initial flux through loop
        let flux_initial = graph.measure_flux_through_loop(&loop_nodes);

        // Increase "magnetic field" (vorticity) through loop
        let t0 = 0.0;
        let t1 = 1.0;  // 1 second
        let steps = 1000;
        let dt = (t1 - t0) / steps as f32;

        for step in 0..steps {
            // Linearly increase B-field
            let b_strength = 1.0 + 0.5 * (step as f32 / steps as f32);
            graph.set_vorticity_in_region((32, 32), 15, b_strength);

            graph.apply_rule_public(rule);
        }

        let flux_final = graph.measure_flux_through_loop(&loop_nodes);
        let d_flux_dt = (flux_final - flux_initial) / (t1 - t0);

        // Measure induced EMF (circulation of E-field around loop)
        let emf_measured = graph.measure_circulation_around_loop(&loop_nodes);

        // Expected: EMF = -dΦ/dt
        let expected_emf = -d_flux_dt;

        TestResult::new(emf_measured, expected_emf, 0.1)
            .with_detail("flux_initial", flux_initial)
            .with_detail("flux_final", flux_final)
            .with_detail("d_flux_dt", d_flux_dt)
    }
}

/// A4: Michelson-Morley Null Result (1887)
///
/// Historical: Speed of light same in all directions (no ether)
/// Null method: Interference pattern shouldn't shift when rotated
///
/// Hypergraph: Wave speed should be isotropic
pub struct MichelsonMorleyNullTest;

impl PhysicsTest for MichelsonMorleyNullTest {
    fn name(&self) -> &str {
        "A4: Michelson-Morley Null Test (Light Speed Isotropy)"
    }

    fn historical_basis(&self) -> &str {
        "Michelson & Morley (1887): No ether wind detected.
        Null method: Interference fringes unchanged when rotated → c is constant."
    }

    fn run(&self, rule: &RewriteRule) -> TestResult {
        let mut graph = Hypergraph::new_3d_lattice(100, 100, 100);

        // Source at center
        let source = graph.center_node();

        // Measure wave speed in multiple directions
        let directions = vec![
            (1.0, 0.0, 0.0),   // +X
            (0.0, 1.0, 0.0),   // +Y
            (0.0, 0.0, 1.0),   // +Z
            (-1.0, 0.0, 0.0),  // -X
            (1.0, 1.0, 0.0),   // Diagonal
        ];

        let mut speeds = Vec::new();

        for &dir in &directions {
            // Emit pulse
            graph.create_wave_pulse(source, 1.0);

            // Propagate
            for _ in 0..1000 {
                graph.apply_rule_public(rule);
            }

            // Measure wavefront position in this direction
            let wavefront_pos = graph.find_wavefront_position(source, dir);
            let speed = wavefront_pos / 1000.0;

            speeds.push(speed);
        }

        // Compute variance (should be near zero)
        let mean_speed = speeds.iter().sum::<f32>() / speeds.len() as f32;
        let variance = speeds.iter()
            .map(|s| (s - mean_speed).powi(2))
            .sum::<f32>() / speeds.len() as f32;

        let relative_variance = variance.sqrt() / mean_speed;

        // Expected: variance ~0 (isotropic)
        let expected_variance = 0.0;

        TestResult::new(relative_variance, expected_variance, 0.05)
            .with_detail("mean_speed", mean_speed)
            .with_detail("std_dev", variance.sqrt())
            .with_detail("speed_x", speeds[0])
            .with_detail("speed_y", speeds[1])
            .with_detail("speed_z", speeds[2])
    }
}

//
// ============================================================================
// CATEGORY B: GRAVITY TESTS (NULL METHOD)
// ============================================================================
//

/// B1: Cavendish Torsion Balance (1798)
///
/// Historical: Measure G by balancing gravitational vs torsion force
/// Null method: Twist wire until forces balance
///
/// Hypergraph: Two masses, measure torque equilibrium
pub struct CavendishTorsionBalanceTest;

impl PhysicsTest for CavendishTorsionBalanceTest {
    fn name(&self) -> &str {
        "B1: Cavendish Torsion Balance (Measure G)"
    }

    fn historical_basis(&self) -> &str {
        "Cavendish (1798): Torsion balance measures gravitational force.
        Null method: Equilibrium angle → G = f(angle, geometry, masses)."
    }

    fn run(&self, rule: &RewriteRule) -> TestResult {
        let mut graph = Hypergraph::new();

        // Create four nodes: two test masses, two large masses
        let m_test_1 = graph.add_node();
        let m_test_2 = graph.add_node();
        let m_large_1 = graph.add_node();
        let m_large_2 = graph.add_node();

        graph.set_scalar(m_test_1, "mass", 1.0);
        graph.set_scalar(m_test_2, "mass", 1.0);
        graph.set_scalar(m_large_1, "mass", 1000.0);
        graph.set_scalar(m_large_2, "mass", 1000.0);

        // Arrange in Cavendish geometry
        graph.set_graph_separation(m_test_1, m_large_1, 10);
        graph.set_graph_separation(m_test_2, m_large_2, 10);
        graph.set_graph_separation(m_test_1, m_test_2, 50);

        // Evolve to equilibrium
        for _ in 0..10000 {
            graph.apply_rule_public(rule);
        }

        // Measure force on test mass 1
        let f_grav = graph.measure_gravitational_force(m_test_1, m_large_1);

        // Theoretical: F = G·m₁·m₂/r²
        // With m_test=1, m_large=1000, r=10:
        // F = G·1000/100 = 10·G

        // We measure F, back-calculate G
        let g_measured = f_grav / 10.0;

        // Expected G (in graph units - should emerge as dimensionless constant)
        let expected_g = 1.0;  // To be calibrated

        TestResult::new(g_measured, expected_g, 0.1)
            .with_detail("force_measured", f_grav)
            .with_detail("g_constant", g_measured)
    }
}

/// B2: Eötvös Balance (1890s)
///
/// Historical: Test equivalence principle
/// Null method: Two different materials fall at same rate
///
/// Hypergraph: Gravitational mass = inertial mass
pub struct EotvosBalanceTest;

impl PhysicsTest for EotvosBalanceTest {
    fn name(&self) -> &str {
        "B2: Eötvös Balance (Equivalence Principle)"
    }

    fn historical_basis(&self) -> &str {
        "Eötvös (1890s): Torsion balance tests if m_gravitational = m_inertial.
        Null method: No twist when materials changed → equivalence holds."
    }

    fn run(&self, rule: &RewriteRule) -> TestResult {
        let mut graph = Hypergraph::new_2d_lattice(100, 100);

        // Massive object (Earth analog)
        let earth = graph.node_at(50, 10);
        graph.set_scalar(earth, "mass", 1e6);

        // Two test objects: different masses
        let light = graph.node_at(48, 80);
        let heavy = graph.node_at(52, 80);

        graph.set_scalar(light, "mass", 1.0);
        graph.set_scalar(heavy, "mass", 1000.0);

        // Track positions
        let mut pos_light = Vec::new();
        let mut pos_heavy = Vec::new();

        // Let them fall
        for step in 0..5000 {
            graph.apply_rule_public(rule);

            if step % 100 == 0 {
                pos_light.push(graph.get_position_proxy(light));
                pos_heavy.push(graph.get_position_proxy(heavy));
            }
        }

        // Compute accelerations
        let accel_light = compute_acceleration(&pos_light);
        let accel_heavy = compute_acceleration(&pos_heavy);

        // Ratio should be 1.0 (same acceleration regardless of mass)
        let accel_ratio = accel_heavy / accel_light;
        let expected_ratio = 1.0;

        TestResult::new(accel_ratio, expected_ratio, 0.01)
            .with_detail("accel_light", accel_light)
            .with_detail("accel_heavy", accel_heavy)
    }
}

/// B3: Pound-Rebka Experiment (1960)
///
/// Historical: Gravitational redshift
/// Null method: Balance frequency shift against height
///
/// Hypergraph: "Clock" oscillations slow in gravitational field
pub struct PoundRebkaRedshiftTest;

impl PhysicsTest for PoundRebkaRedshiftTest {
    fn name(&self) -> &str {
        "B3: Pound-Rebka (Gravitational Time Dilation)"
    }

    fn historical_basis(&self) -> &str {
        "Pound & Rebka (1960): Photon frequency changes with height in gravity.
        Null method: Mössbauer effect compensates for gravitational redshift."
    }

    fn run(&self, rule: &RewriteRule) -> TestResult {
        let mut graph = Hypergraph::new_2d_lattice(200, 200);

        // Massive object
        let mass = graph.node_at(100, 100);
        graph.set_scalar(mass, "mass", 1e10);

        // Evolve to establish spacetime curvature
        for _ in 0..1000 {
            graph.apply_rule_public(rule);
        }

        // "Clocks" at different heights
        let clock_low = graph.node_at(100, 80);   // Closer to mass
        let clock_high = graph.node_at(100, 150); // Farther from mass

        // Oscillate both clocks
        let periods_low = count_oscillations(&mut graph, clock_low, 10000, rule);
        let periods_high = count_oscillations(&mut graph, clock_high, 10000, rule);

        // Frequency ratio
        let freq_ratio = periods_high as f32 / periods_low as f32;

        // Expected from GR: f_high/f_low = √(1 + Δφ/c²)
        // For weak field: ≈ 1 + Δφ/(2c²)

        let phi_low = graph.measure_gravitational_potential(clock_low);
        let phi_high = graph.measure_gravitational_potential(clock_high);
        let delta_phi = phi_high - phi_low;

        let expected_ratio = 1.0 + delta_phi / (2.0 * C * C);

        TestResult::new(freq_ratio, expected_ratio, 0.05)
            .with_detail("periods_low", periods_low as f32)
            .with_detail("periods_high", periods_high as f32)
            .with_detail("delta_phi", delta_phi)
    }
}

//
// ============================================================================
// CATEGORY C: FLUID DYNAMICS TESTS (EQUILIBRIUM METHOD)
// ============================================================================
//

/// C1: Torricelli's Barometer (1643)
///
/// Historical: Mercury column height balances atmospheric pressure
/// Null method: Equilibrium height → pressure measurement
///
/// Hypergraph: Pressure gradient balances "gravity"
pub struct TorricelliBarometerTest;

impl PhysicsTest for TorricelliBarometerTest {
    fn name(&self) -> &str {
        "C1: Torricelli Barometer (Pressure Balance)"
    }

    fn historical_basis(&self) -> &str {
        "Torricelli (1643): Mercury column height = atmospheric pressure.
        Null method: Equilibrium when P_atm = ρ·g·h."
    }

    fn run(&self, rule: &RewriteRule) -> TestResult {
        let mut graph = Hypergraph::new_2d_lattice(50, 200);

        // Set pressure field (top = P_atm, bottom = P_atm + ρgh)
        graph.set_external_pressure_gradient(10.0);

        // Evolve to equilibrium
        for _ in 0..10000 {
            graph.apply_rule_public(rule);
            if graph.is_pressure_equilibrium(1e-6) {
                break;
            }
        }

        // Measure pressure at different heights
        let p_bottom = graph.get_scalar(graph.node_at(25, 10), "pressure").unwrap_or(0.0);
        let p_top = graph.get_scalar(graph.node_at(25, 190), "pressure").unwrap_or(0.0);

        let pressure_diff = p_bottom - p_top;

        // Expected: ΔP = ρ·g·h
        let height = 180.0;  // Graph units
        let rho = 1.0;  // Density
        let g = 10.0;
        let expected_diff = rho * g * height;

        TestResult::new(pressure_diff, expected_diff, 0.1)
            .with_detail("p_bottom", p_bottom)
            .with_detail("p_top", p_top)
            .with_detail("height", height)
    }
}

/// C2: Venturi Meter (1797)
///
/// Historical: Pressure drops where fluid speed increases
/// Null method: P₁ + ½ρv₁² = P₂ + ½ρv₂² (Bernoulli)
///
/// Hypergraph: Velocity-pressure balance
pub struct VenturiMeterTest;

impl PhysicsTest for VenturiMeterTest {
    fn name(&self) -> &str {
        "C2: Venturi Meter (Bernoulli's Principle)"
    }

    fn historical_basis(&self) -> &str {
        "Venturi (1797): Fluid speed up → pressure down.
        Null method: Total energy (P + ½ρv²) conserved along streamline."
    }

    fn run(&self, rule: &RewriteRule) -> TestResult {
        let mut graph = Hypergraph::new_2d_lattice(200, 50);

        // Create pipe with constriction
        // Wide section: rows 0-70
        // Narrow section: rows 71-130
        // Wide section: rows 131-200

        graph.initialize_pipe_flow(1.0);

        // Evolve to steady state
        for _ in 0..10000 {
            graph.apply_rule_public(rule);
        }

        // Measure at wide vs narrow sections
        let (p1, v1) = graph.measure_pressure_velocity((35, 25));  // Wide
        let (p2, v2) = graph.measure_pressure_velocity((100, 25)); // Narrow

        // Bernoulli: P₁ + ½ρv₁² = P₂ + ½ρv₂²
        let rho = 1.0;
        let e1 = p1 + 0.5 * rho * v1 * v1;
        let e2 = p2 + 0.5 * rho * v2 * v2;

        let energy_ratio = e2 / e1;
        let expected_ratio = 1.0;  // Energy conserved

        TestResult::new(energy_ratio, expected_ratio, 0.05)
            .with_detail("p1", p1)
            .with_detail("v1", v1)
            .with_detail("p2", p2)
            .with_detail("v2", v2)
            .with_detail("e1", e1)
            .with_detail("e2", e2)
    }
}

//
// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================
//

fn compute_acceleration(positions: &[f32]) -> f32 {
    if positions.len() < 3 {
        return 0.0;
    }

    // Fit parabola: x = x₀ + v₀·t + ½a·t²
    // Simple finite difference: a ≈ (x[i+1] - 2x[i] + x[i-1]) / dt²

    let mut accels = Vec::new();
    for i in 1..positions.len() - 1 {
        let a = positions[i + 1] - 2.0 * positions[i] + positions[i - 1];
        accels.push(a);
    }

    accels.iter().sum::<f32>() / accels.len() as f32
}

fn count_oscillations(
    graph: &mut Hypergraph,
    node: usize,
    steps: usize,
    rule: &RewriteRule,
) -> usize {
    let mut values = Vec::new();

    for _ in 0..steps {
        graph.apply_rule_public(rule);
        let val = graph.get_scalar(node, "field").unwrap_or(0.0);
        values.push(val);
    }

    // Count zero crossings
    let mut crossings = 0;
    for i in 1..values.len() {
        if values[i - 1] * values[i] < 0.0 {
            crossings += 1;
        }
    }

    crossings / 2  // Two crossings per period
}

// Constant for speed of light (in graph units)
const C: f32 = 1.0;  // To be calibrated from experiments

//
// ============================================================================
// TEST SUITE RUNNER
// ============================================================================
//

pub struct ExperimentalTestSuite {
    tests: Vec<Box<dyn PhysicsTest>>,
}

impl ExperimentalTestSuite {
    pub fn new() -> Self {
        let mut tests: Vec<Box<dyn PhysicsTest>> = Vec::new();

        // Electromagnetism
        tests.push(Box::new(CoulombBalanceTest {
            charge_ratio: 1.0,
            distance_ratio: 2.0,
        }));
        tests.push(Box::new(AmpereForceBalanceTest {
            wire_separation: 50,
        }));
        tests.push(Box::new(FaradayInductionBalanceTest));
        tests.push(Box::new(MichelsonMorleyNullTest));

        // Gravity
        tests.push(Box::new(CavendishTorsionBalanceTest));
        tests.push(Box::new(EotvosBalanceTest));
        tests.push(Box::new(PoundRebkaRedshiftTest));

        // Fluids
        tests.push(Box::new(TorricelliBarometerTest));
        tests.push(Box::new(VenturiMeterTest));

        Self { tests }
    }

    pub fn run_all(&self, rule: &RewriteRule) -> Vec<(String, TestResult)> {
        let mut results = Vec::new();

        for test in &self.tests {
            println!("Running: {}", test.name());
            println!("  Basis: {}", test.historical_basis());

            let result = test.run(rule);

            println!("  Result: {} (error: {:.2}%)",
                     if result.passed { "PASS" } else { "FAIL" },
                     result.relative_error * 100.0);
            println!("  Measured: {:.4}, Expected: {:.4}",
                     result.measured_value, result.expected_value);
            println!();

            results.push((test.name().to_string(), result));
        }

        results
    }

    pub fn compute_score(&self, results: &[(String, TestResult)]) -> f32 {
        let total_weight: f32 = self.tests.iter().map(|t| t.weight()).sum();
        let passed_weight: f32 = results.iter()
            .zip(&self.tests)
            .map(|((_, result), test)| {
                if result.passed {
                    test.weight()
                } else {
                    0.0
                }
            })
            .sum();

        (passed_weight / total_weight) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suite_creation() {
        let suite = ExperimentalTestSuite::new();
        assert!(suite.tests.len() >= 9, "Should have at least 9 tests");
    }

    #[test]
    fn test_coulomb_balance_structure() {
        let test = CoulombBalanceTest {
            charge_ratio: 1.0,
            distance_ratio: 2.0,
        };

        assert_eq!(test.name(), "A1: Coulomb Balance (Inverse Square Law)");
        assert!(test.historical_basis().contains("Cavendish"));
    }
}
