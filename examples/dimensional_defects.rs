//! Dimensional Defects Example
//!
//! Demonstrates:
//! 1. Particles as 0D point defects
//! 2. Strings as 1D line defects
//! 3. Membranes as 2D sheet defects
//! 4. 4D bubbles as higher-dimensional regions
//!
//! Key insight: Different dimensions coexist in 3D space!

use em_physics_sandbox::physics::AdaptiveAutomata;
use em_physics_sandbox::visualization::{DimensionalSlice, MultiSliceView, DimensionMode};
use glam::Vec3;

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  DIMENSIONAL DEFECTS: Seeing Hidden Dimensions");
    println!("═══════════════════════════════════════════════════════════\n");

    // Create 3D automata (base reality)
    let nx = 50;
    let ny = 50;
    let nz = 50;

    println!("Creating 3D space ({}x{}x{})...", nx, ny, nz);
    let mut automata = AdaptiveAutomata::new_uniform(nx, ny, nz, 3.0);

    println!("Initial state:");
    println!("  Average dimension: {:.2}", automata.avg_dimension);
    println!("  Min dimension: {:.2}", automata.min_dimension);
    println!("  Max dimension: {:.2}", automata.max_dimension);
    println!();

    // 1. Create particle defect (0D point)
    println!("Creating particle defect (0D point)...");
    let particle_pos = Vec3::new(15.0, 25.0, 25.0);
    automata.create_particle_defect(particle_pos, 3.0);
    println!("  → Particle at ({:.1}, {:.1}, {:.1})", particle_pos.x, particle_pos.y, particle_pos.z);

    // 2. Create string defect (1D line)
    println!("Creating string defect (1D line)...");
    let string_start = Vec3::new(25.0, 10.0, 25.0);
    let string_end = Vec3::new(25.0, 40.0, 25.0);
    automata.create_string_defect(string_start, string_end, 2.0);
    println!("  → String from ({:.1}, {:.1}, {:.1}) to ({:.1}, {:.1}, {:.1})",
             string_start.x, string_start.y, string_start.z,
             string_end.x, string_end.y, string_end.z);

    // 3. Create membrane defect (2D sheet)
    println!("Creating membrane defect (2D sheet)...");
    let membrane_center = Vec3::new(35.0, 25.0, 25.0);
    let membrane_normal = Vec3::new(1.0, 0.0, 0.0);  // YZ plane
    automata.create_membrane_defect(membrane_center, membrane_normal, 8.0, 1.5);
    println!("  → Membrane at ({:.1}, {:.1}, {:.1}), normal ({:.1}, {:.1}, {:.1})",
             membrane_center.x, membrane_center.y, membrane_center.z,
             membrane_normal.x, membrane_normal.y, membrane_normal.z);

    // 4. Create 4D bubble (higher dimension)
    println!("Creating 4D bubble (exotic region)...");
    let bubble_pos = Vec3::new(40.0, 40.0, 25.0);
    automata.create_4d_bubble(bubble_pos, 4.0);
    println!("  → 4D bubble at ({:.1}, {:.1}, {:.1})", bubble_pos.x, bubble_pos.y, bubble_pos.z);
    println!();

    println!("Final state:");
    println!("  Average dimension: {:.2}", automata.avg_dimension);
    println!("  Min dimension: {:.2}", automata.min_dimension);
    println!("  Max dimension: {:.2}", automata.max_dimension);
    println!();

    // Analyze dimensional structure
    println!("═══════════════════════════════════════════════════════════");
    println!("  DIMENSIONAL ANALYSIS");
    println!("═══════════════════════════════════════════════════════════\n");

    let mut count_by_dim = [0; 5];  // Count cells in each dimension range

    for cell in &automata.cells {
        let d = cell.dimension;
        let bin = if d < 1.0 { 0 }
                  else if d < 2.0 { 1 }
                  else if d < 3.0 { 2 }
                  else if d < 4.0 { 3 }
                  else { 4 };
        count_by_dim[bin] += 1;
    }

    let total = automata.cells.len();
    println!("Dimensional distribution:");
    println!("  0D-1D (particles):  {:5} cells ({:.1}%)", count_by_dim[0], 100.0 * count_by_dim[0] as f32 / total as f32);
    println!("  1D-2D (strings):    {:5} cells ({:.1}%)", count_by_dim[1], 100.0 * count_by_dim[1] as f32 / total as f32);
    println!("  2D-3D (membranes):  {:5} cells ({:.1}%)", count_by_dim[2], 100.0 * count_by_dim[2] as f32 / total as f32);
    println!("  3D-4D (normal):     {:5} cells ({:.1}%)", count_by_dim[3], 100.0 * count_by_dim[3] as f32 / total as f32);
    println!("  4D+ (exotic):       {:5} cells ({:.1}%)", count_by_dim[4], 100.0 * count_by_dim[4] as f32 / total as f32);
    println!();

    // Compute dimensional forces
    println!("═══════════════════════════════════════════════════════════");
    println!("  DIMENSIONAL FORCES");
    println!("═══════════════════════════════════════════════════════════\n");

    // Sample a few cells and show forces
    let sample_indices = [
        automata.cell_index(15, 25, 25),  // Near particle
        automata.cell_index(25, 25, 25),  // On string
        automata.cell_index(35, 25, 25),  // On membrane
        automata.cell_index(40, 40, 25),  // In 4D bubble
    ];

    for &idx in &sample_indices {
        if idx < automata.cells.len() {
            let cell = &automata.cells[idx];
            let force = automata.dimensional_force(idx);
            let grad = automata.dimensional_gradient(idx);

            println!("Cell at ({:.1}, {:.1}, {:.1}):", cell.position.x, cell.position.y, cell.position.z);
            println!("  Dimension: {:.2}", cell.dimension);
            println!("  Gradient: ({:.3}, {:.3}, {:.3}), magnitude: {:.3}",
                     grad.x, grad.y, grad.z, grad.length());
            println!("  Force: ({:.3}, {:.3}, {:.3}), magnitude: {:.3}",
                     force.x, force.y, force.z, force.length());
            println!();
        }
    }

    // Render slices
    println!("═══════════════════════════════════════════════════════════");
    println!("  RENDERING DIMENSIONAL SLICES");
    println!("═══════════════════════════════════════════════════════════\n");

    // XY plane through middle
    println!("Rendering XY slice (z={})...", nz / 2);
    let mut slice_xy = DimensionalSlice::new(0, nz / 2);
    slice_xy.mode = DimensionMode::ColorCoded;
    slice_xy.dim_min = automata.min_dimension;
    slice_xy.dim_max = automata.max_dimension;

    let img_xy = slice_xy.render(&automata, 512, 512);
    img_xy.save("dimensional_slice_xy.png").expect("Failed to save image");
    println!("  → Saved dimensional_slice_xy.png");

    // XZ plane through middle
    println!("Rendering XZ slice (y={})...", ny / 2);
    let mut slice_xz = DimensionalSlice::new(1, ny / 2);
    slice_xz.mode = DimensionMode::ColorCoded;
    slice_xz.dim_min = automata.min_dimension;
    slice_xz.dim_max = automata.max_dimension;

    let img_xz = slice_xz.render(&automata, 512, 512);
    img_xz.save("dimensional_slice_xz.png").expect("Failed to save image");
    println!("  → Saved dimensional_slice_xz.png");

    // Gradient view
    println!("Rendering dimensional gradient view...");
    let mut slice_grad = DimensionalSlice::new(0, nz / 2);
    slice_grad.mode = DimensionMode::Gradient;

    let img_grad = slice_grad.render(&automata, 512, 512);
    img_grad.save("dimensional_gradient.png").expect("Failed to save image");
    println!("  → Saved dimensional_gradient.png");

    // Multi-slice view
    println!("Rendering multi-slice view (9 slices through Z)...");
    let multi = MultiSliceView::new(0, 9);
    let img_multi = multi.render(&automata, 170, 170);
    img_multi.save("dimensional_multislice.png").expect("Failed to save image");
    println!("  → Saved dimensional_multislice.png");

    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("  INTERPRETATION");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("What you're seeing:");
    println!();
    println!("  BLUE regions (d≈1): String-like structures");
    println!("    → Particles confined to 1D paths");
    println!("    → Could represent quarks confined in hadrons");
    println!();
    println!("  GREEN regions (d≈2): Sheet-like structures");
    println!("    → 2D membranes embedded in 3D");
    println!("    → Could represent domain walls or dark matter");
    println!();
    println!("  YELLOW regions (d≈3): Normal space");
    println!("    → Our observable 3D universe");
    println!("    → Where we live and measure");
    println!();
    println!("  RED regions (d≈4): Exotic higher dimensions");
    println!("    → Compactified or emergent 4th dimension");
    println!("    → Could represent black hole interiors or early universe");
    println!();
    println!("Dimensional forces:");
    println!("  → Particles are \"pushed\" down dimensional gradients");
    println!("  → This could explain confinement, dark matter, etc.");
    println!("  → Force laws depend on local dimension!");
    println!();

    println!("═══════════════════════════════════════════════════════════");
    println!("  NEXT STEPS");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("1. Project hypergraph → adaptive automata");
    println!("2. Evolve with dimension-dependent physics");
    println!("3. Measure particle trajectories through dimensional defects");
    println!("4. Test if dimensional engineering is possible!");
    println!();
}
