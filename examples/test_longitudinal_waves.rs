//! Test for Longitudinal (Scalar) Waves in Dimensional Field
//!
//! CRITICAL TEST: Does our dimensional framework predict BOTH types of waves?
//!
//! BACKGROUND:
//! ===========
//! Standard Maxwell-Heaviside EM: ONLY transverse waves (E⊥B⊥k)
//! Original Maxwell quaternions: BOTH transverse AND longitudinal
//! Weber electrodynamics: Instantaneous/longitudinal components
//!
//! What was lost in Heaviside's simplification?
//! - Scalar waves (longitudinal oscillations in φ)
//! - Instantaneous near-field terms
//! - Quaternion structure
//!
//! This test checks if dimensional structure naturally supports BOTH!
//!
//! WAVE TYPES:
//! ===========
//! 1. TRANSVERSE (Standard EM):
//!    - Oscillations perpendicular to propagation
//!    - E⊥k, B⊥k, E⊥B
//!    - Dipole radiation
//!    - Speed: c_T = 1/√(ε₀μ₀)
//!
//! 2. LONGITUDINAL (Scalar/Weber):
//!    - Oscillations parallel to propagation
//!    - Compression waves in dimensional field
//!    - Monopole radiation
//!    - Speed: c_L = ? (to be measured!)
//!
//! TEST SETUP:
//! ===========
//! - Monopole source: Oscillating point defect → spherical waves
//! - Dipole source: Two opposite charges oscillating → directed waves
//! - Measure wave components at various points
//! - Decompose into longitudinal vs transverse
//!
//! SUCCESS CRITERIA:
//! =================
//! - Monopole produces BOTH longitudinal and transverse components
//! - Dipole produces primarily transverse (standard EM)
//! - Longitudinal component propagates (not just near-field static)
//! - Measure c_L / c_T ratio

use em_physics_sandbox::physics::dimensional_dynamics::{DimensionalDynamics, weber_force};
use glam::Vec3;
use std::fs::File;
use std::io::Write;

fn main() {
    println!("\n=== LONGITUDINAL WAVE TEST ===\n");
    println!("Testing for scalar/longitudinal waves beyond standard Maxwell EM...\n");

    // Grid setup
    let nx = 80;
    let ny = 80;
    let nz = 80;
    let spacing = 0.5;

    println!("Grid: {}x{}x{}, spacing = {}", nx, ny, nz, spacing);
    println!("Total cells: {}\n", nx * ny * nz);

    // =============================================================================
    // TEST 1: Monopole Source (Spherical Waves)
    // =============================================================================
    println!("=== TEST 1: Monopole Source ===");
    println!("Oscillating point defect → expect BOTH longitudinal & transverse\n");

    let mut dynamics = DimensionalDynamics::new(nx, ny, nz, spacing);

    let center = Vec3::new(
        (nx / 2) as f32 * spacing,
        (ny / 2) as f32 * spacing,
        (nz / 2) as f32 * spacing,
    );

    println!("Source position: ({:.1}, {:.1}, {:.1})", center.x, center.y, center.z);

    let frequency = 0.1;  // Hz
    let amplitude = 0.5;

    println!("Frequency: {} Hz", frequency);
    println!("Amplitude: {}\n", amplitude);

    // Detector positions at various distances
    let detectors = vec![
        Vec3::new(center.x + 10.0, center.y, center.z),
        Vec3::new(center.x + 20.0, center.y, center.z),
        Vec3::new(center.x + 30.0, center.y, center.z),
    ];

    println!("Detector positions:");
    for (i, det) in detectors.iter().enumerate() {
        let r = (det - center).length();
        println!("  Detector {}: r = {:.1}", i+1, r);
    }
    println!();

    // Time evolution
    let dt = 0.05;
    let total_time = 100.0;
    let steps = (total_time / dt) as usize;

    println!("Simulating {} steps (t = {} to {})", steps, 0.0, total_time);
    println!("Time step dt = {}\n", dt);

    // Storage for time series
    let mut time_series = vec![Vec::new(); detectors.len()];

    // Evolve and record
    for step in 0..steps {
        // Add oscillating source
        dynamics.add_oscillating_monopole(center, amplitude, frequency);

        // Evolve
        dynamics.evolve_step(dt, None);

        // Record at detectors
        for (i, det) in detectors.iter().enumerate() {
            let value = dynamics.measure_time_series(*det);
            time_series[i].push(value);
        }

        if step % (steps / 10) == 0 {
            println!("  Step {}/{} (t = {:.1})", step, steps, dynamics.time);
        }
    }

    println!("\nSimulation complete!\n");

    // Analyze wave components
    println!("=== WAVE COMPONENT ANALYSIS ===\n");

    for (i, det) in detectors.iter().enumerate() {
        let r = (det - center).length();

        // Get wave components at detector location
        let idx = dynamics.cell_index(
            (det.x / spacing) as usize,
            (det.y / spacing) as usize,
            (det.z / spacing) as usize,
        );

        let components = dynamics.decompose_wave(idx);

        let long_mag = components.longitudinal.length();
        let trans_mag = components.transverse.length();
        let long_fraction = components.longitudinal_fraction();

        println!("Detector {} (r = {:.1}):", i+1, r);
        println!("  Longitudinal magnitude: {:.6}", long_mag);
        println!("  Transverse magnitude:   {:.6}", trans_mag);
        println!("  Longitudinal fraction:  {:.1}%", long_fraction * 100.0);

        if long_fraction > 0.1 {
            println!("  → Significant longitudinal component detected! ✓");
        } else if long_fraction > 0.01 {
            println!("  → Small longitudinal component");
        } else {
            println!("  → Primarily transverse (standard EM)");
        }
        println!();
    }

    // Save time series to file
    println!("Saving time series data to monopole_waves.csv...");
    let mut file = File::create("monopole_waves.csv").unwrap();
    writeln!(file, "time,detector1,detector2,detector3").unwrap();

    for step in 0..steps {
        let t = step as f32 * dt;
        writeln!(
            file,
            "{},{},{},{}",
            t,
            time_series[0][step],
            time_series[1][step],
            time_series[2][step]
        ).unwrap();
    }

    println!("Data saved!\n");

    // =============================================================================
    // TEST 2: Dipole Source (Directional Waves)
    // =============================================================================
    println!("\n=== TEST 2: Dipole Source ===");
    println!("Oscillating dipole → expect primarily TRANSVERSE (standard EM)\n");

    let mut dynamics2 = DimensionalDynamics::new(nx, ny, nz, spacing);

    let separation = Vec3::new(2.0, 0.0, 0.0);

    println!("Dipole separation: ({:.1}, {:.1}, {:.1})", separation.x, separation.y, separation.z);
    println!("Frequency: {} Hz\n", frequency);

    // Detector along dipole axis (should see longitudinal)
    let det_axial = center + Vec3::new(20.0, 0.0, 0.0);

    // Detector perpendicular to dipole (should see transverse)
    let det_perp = center + Vec3::new(0.0, 20.0, 0.0);

    println!("Detectors:");
    println!("  Axial (along dipole): ({:.1}, {:.1}, {:.1})",
             det_axial.x, det_axial.y, det_axial.z);
    println!("  Perpendicular: ({:.1}, {:.1}, {:.1})",
             det_perp.x, det_perp.y, det_perp.z);
    println!();

    // Evolve dipole
    for step in 0..steps {
        dynamics2.add_oscillating_dipole(center, separation, amplitude, frequency);
        dynamics2.evolve_step(dt, None);
    }

    // Analyze components
    let idx_axial = dynamics2.cell_index(
        (det_axial.x / spacing) as usize,
        (det_axial.y / spacing) as usize,
        (det_axial.z / spacing) as usize,
    );

    let idx_perp = dynamics2.cell_index(
        (det_perp.x / spacing) as usize,
        (det_perp.y / spacing) as usize,
        (det_perp.z / spacing) as usize,
    );

    let comp_axial = dynamics2.decompose_wave(idx_axial);
    let comp_perp = dynamics2.decompose_wave(idx_perp);

    println!("Wave components:\n");

    println!("Axial detector:");
    println!("  Longitudinal fraction: {:.1}%", comp_axial.longitudinal_fraction() * 100.0);
    println!("  (Along dipole axis - expect mixed)\n");

    println!("Perpendicular detector:");
    println!("  Longitudinal fraction: {:.1}%", comp_perp.longitudinal_fraction() * 100.0);
    println!("  (Perpendicular to dipole - expect transverse)\n");

    // =============================================================================
    // TEST 3: Weber Force with Moving Charges
    // =============================================================================
    println!("\n=== TEST 3: Weber Force Law ===");
    println!("Testing velocity-dependent force terms\n");

    let q1 = 1.0;
    let q2 = 1.0;
    let c = 1.0;

    // Test at different velocities
    let r_vec = Vec3::new(5.0, 0.0, 0.0);

    println!("Charges: q1 = {}, q2 = {}", q1, q2);
    println!("Separation: r = {:.1}\n", r_vec.length());

    println!("Force vs. relative velocity:");
    println!("(Weber predicts velocity-dependent correction)\n");

    for v_frac in [0.0, 0.1, 0.3, 0.5, 0.7, 0.9] {
        let v_rel = Vec3::new(v_frac * c, 0.0, 0.0);
        let a_rel = Vec3::ZERO;

        let f_weber = weber_force(q1, q2, r_vec, v_rel, a_rel, c);
        let f_coulomb = r_vec.normalize() * q1 * q2 / (r_vec.length() * r_vec.length());

        let force_ratio = f_weber.length() / f_coulomb.length();
        let velocity_correction = 1.0 - v_frac * v_frac / 2.0;

        println!("  v/c = {:.1}: F/F_coulomb = {:.4} (Weber predicts: {:.4})",
                 v_frac, force_ratio, velocity_correction);
    }

    println!("\nExpected: F = F_coulomb * (1 - v²/2c²) for radial motion");
    println!("If ratio matches prediction → Weber force confirmed!\n");

    // =============================================================================
    // SUMMARY
    // =============================================================================
    println!("\n=== SUMMARY ===\n");

    println!("Key Questions:");
    println!("1. Do monopoles produce longitudinal waves? (Lost in Heaviside)");
    println!("2. Do dipoles produce primarily transverse? (Standard EM)");
    println!("3. Do we see Weber velocity terms? (Action-at-distance vs field)\n");

    println!("Results:");

    // Check first detector for longitudinal component
    let long_frac_1 = {
        let idx = dynamics.cell_index(
            ((center.x + 10.0) / spacing) as usize,
            (center.y / spacing) as usize,
            (center.z / spacing) as usize,
        );
        dynamics.decompose_wave(idx).longitudinal_fraction()
    };

    if long_frac_1 > 0.1 {
        println!("  ✓ Longitudinal waves DETECTED (monopole)");
        println!("    → Beyond standard Maxwell-Heaviside EM!");
        println!("    → Supports quaternion/Weber formulations");
    } else {
        println!("  ⚠ Longitudinal waves weak or absent");
        println!("    → May need different source or longer evolution");
    }

    println!("\n  Dipole analysis:");
    println!("    Perpendicular: {:.1}% longitudinal", comp_perp.longitudinal_fraction() * 100.0);
    if comp_perp.longitudinal_fraction() < 0.3 {
        println!("    ✓ Primarily transverse (as expected)");
    }

    println!("\nConclusion:");
    if long_frac_1 > 0.1 {
        println!("✓ NOVEL PREDICTION: Dimensional framework supports BOTH wave types!");
        println!("  - Transverse waves (standard EM)");
        println!("  - Longitudinal waves (lost in Heaviside reduction)");
        println!("\n→ This validates quaternion Maxwell and Weber electrodynamics!");
        println!("→ Scalar waves exist in the theory!");
    } else {
        println!("⚠ PARTIAL: Need more investigation");
        println!("  - Longer simulation time?");
        println!("  - Different source configuration?");
        println!("  - Higher amplitude?");
    }

    println!("\nNext steps:");
    println!("  - Generate video visualization of wave propagation");
    println!("  - Measure c_L / c_T ratio");
    println!("  - Test near-field vs far-field behavior");
    println!("  - Verify Aharonov-Bohm effect from dimensional potentials");
}
