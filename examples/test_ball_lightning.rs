// Ball Lightning Simulation
//
// Test creation and energy extraction from self-sustained toroidal plasma vortex
//
// This explores:
// 1. Can we create a stable ball lightning in simulation?
// 2. How long does it persist?
// 3. How much energy can we extract?
// 4. What is its topological winding number?

use std::f32::consts::PI;
use std::fs::File;
use std::io::Write as IoWrite;

const C: f32 = 1.0; // Speed of light in simulation units

// Vector in 3D space
#[derive(Debug, Clone, Copy)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn sub(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn add(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    fn scale(&self, s: f32) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
}

// Simplified field for ball lightning simulation
struct BallLightningField {
    nx: usize,
    ny: usize,
    nz: usize,
    spacing: f32,

    // Fields
    dimension: Vec<f32>,
    dimension_velocity: Vec<f32>,
    vector_potential: Vec<Vec3>,

    time: f32,
}

impl BallLightningField {
    fn new(nx: usize, ny: usize, nz: usize, spacing: f32) -> Self {
        let total = nx * ny * nz;

        BallLightningField {
            nx,
            ny,
            nz,
            spacing,
            dimension: vec![3.0; total],
            dimension_velocity: vec![0.0; total],
            vector_potential: vec![Vec3::new(0.0, 0.0, 0.0); total],
            time: 0.0,
        }
    }

    fn cell_index(&self, i: usize, j: usize, k: usize) -> usize {
        i + self.nx * (j + self.ny * k)
    }

    fn grid_position(&self, i: usize, j: usize, k: usize) -> Vec3 {
        let half_x = (self.nx as f32 * self.spacing) / 2.0;
        let half_y = (self.ny as f32 * self.spacing) / 2.0;
        let half_z = (self.nz as f32 * self.spacing) / 2.0;

        Vec3::new(
            i as f32 * self.spacing - half_x,
            j as f32 * self.spacing - half_y,
            k as f32 * self.spacing - half_z,
        )
    }

    // Evolve field forward one timestep
    fn evolve(&mut self, dt: f32) {
        // Wave equation: ∂²d/∂t² = c²∇²d
        let mut laplacian = vec![0.0; self.dimension.len()];

        for i in 1..self.nx - 1 {
            for j in 1..self.ny - 1 {
                for k in 1..self.nz - 1 {
                    let idx = self.cell_index(i, j, k);

                    let d_center = self.dimension[idx];

                    let d_xplus = self.dimension[self.cell_index(i + 1, j, k)];
                    let d_xminus = self.dimension[self.cell_index(i - 1, j, k)];
                    let d_yplus = self.dimension[self.cell_index(i, j + 1, k)];
                    let d_yminus = self.dimension[self.cell_index(i, j - 1, k)];
                    let d_zplus = self.dimension[self.cell_index(i, j, k + 1)];
                    let d_zminus = self.dimension[self.cell_index(i, j, k - 1)];

                    laplacian[idx] = (d_xplus + d_xminus + d_yplus +
                                     d_yminus + d_zplus + d_zminus - 6.0 * d_center)
                                     / (self.spacing * self.spacing);
                }
            }
        }

        // Update velocities
        for i in 0..self.dimension.len() {
            self.dimension_velocity[i] += C * C * laplacian[i] * dt;

            // Damping (models energy radiation)
            self.dimension_velocity[i] *= 0.9995;
        }

        // Update positions
        for i in 0..self.dimension.len() {
            self.dimension[i] += self.dimension_velocity[i] * dt;
        }

        self.time += dt;
    }

    // Compute total energy in field
    fn total_energy(&self) -> f32 {
        let mut energy = 0.0;

        for i in 1..self.nx - 1 {
            for j in 1..self.ny - 1 {
                for k in 1..self.nz - 1 {
                    let idx = self.cell_index(i, j, k);

                    // Kinetic energy: ½(∂d/∂t)²
                    let vel = self.dimension_velocity[idx];
                    let kinetic = 0.5 * vel * vel;

                    // Gradient energy: ½c²(∇d)²
                    let d_center = self.dimension[idx];
                    let d_xplus = self.dimension[self.cell_index(i + 1, j, k)];
                    let d_yplus = self.dimension[self.cell_index(i, j + 1, k)];
                    let d_zplus = self.dimension[self.cell_index(i, j, k + 1)];

                    let grad_x = (d_xplus - d_center) / self.spacing;
                    let grad_y = (d_yplus - d_center) / self.spacing;
                    let grad_z = (d_zplus - d_center) / self.spacing;

                    let gradient_energy = 0.5 * C * C * (grad_x * grad_x +
                                                         grad_y * grad_y +
                                                         grad_z * grad_z);

                    energy += (kinetic + gradient_energy) * self.spacing.powi(3);
                }
            }
        }

        energy
    }

    // Measure energy in a spherical region
    fn energy_in_sphere(&self, center: Vec3, radius: f32) -> f32 {
        let mut energy = 0.0;

        for i in 1..self.nx - 1 {
            for j in 1..self.ny - 1 {
                for k in 1..self.nz - 1 {
                    let pos = self.grid_position(i, j, k);
                    let r = pos.sub(&center).length();

                    if r < radius {
                        let idx = self.cell_index(i, j, k);

                        let vel = self.dimension_velocity[idx];
                        let kinetic = 0.5 * vel * vel;

                        let d_center = self.dimension[idx];
                        let d_xplus = self.dimension[self.cell_index(i + 1, j, k)];
                        let d_yplus = self.dimension[self.cell_index(i, j + 1, k)];
                        let d_zplus = self.dimension[self.cell_index(i, j, k + 1)];

                        let grad_x = (d_xplus - d_center) / self.spacing;
                        let grad_y = (d_yplus - d_center) / self.spacing;
                        let grad_z = (d_zplus - d_center) / self.spacing;

                        let gradient_energy = 0.5 * C * C * (grad_x * grad_x +
                                                             grad_y * grad_y +
                                                             grad_z * grad_z);

                        energy += (kinetic + gradient_energy) * self.spacing.powi(3);
                    }
                }
            }
        }

        energy
    }
}

// Create initial ball lightning configuration
fn create_ball_lightning(
    field: &mut BallLightningField,
    center: Vec3,
    major_radius: f32,
    minor_radius: f32,
    n_toroidal: i32,
    n_poloidal: i32,
) {
    println!("Creating ball lightning:");
    println!("  Center: ({:.1}, {:.1}, {:.1})", center.x, center.y, center.z);
    println!("  Major radius: {} units", major_radius);
    println!("  Minor radius: {} units", minor_radius);
    println!("  Winding: {}×{} (φ = {:.3})",
             n_toroidal, n_poloidal,
             n_toroidal as f32 / n_poloidal as f32);

    for i in 0..field.nx {
        for j in 0..field.ny {
            for k in 0..field.nz {
                let pos = field.grid_position(i, j, k);
                let r_vec = pos.sub(&center);

                // Toroidal coordinates
                let rho = (r_vec.x * r_vec.x + r_vec.y * r_vec.y).sqrt();
                let dist_from_ring = ((rho - major_radius).powi(2) + r_vec.z.powi(2)).sqrt();

                // Dimensional deficit in torus
                if dist_from_ring < minor_radius {
                    let idx = field.cell_index(i, j, k);

                    // Smooth profile
                    let profile = (1.0 - dist_from_ring / minor_radius).max(0.0);
                    let deficit = 1.0 * profile; // Reduce d by up to 1.0

                    field.dimension[idx] = 3.0 - deficit;

                    // Add circulation (both toroidal and poloidal)
                    let theta_toroidal = r_vec.y.atan2(r_vec.x);
                    let theta_poloidal = r_vec.z.atan2(rho - major_radius);

                    let circ_strength = 0.5 * profile;

                    // Vector potential creates circulation
                    let a_toroidal = circ_strength * (n_toroidal as f32);
                    let a_poloidal = circ_strength * (n_poloidal as f32);

                    field.vector_potential[idx] = Vec3::new(
                        -r_vec.y / (rho + 0.01) * a_toroidal,
                        r_vec.x / (rho + 0.01) * a_toroidal,
                        a_poloidal,
                    );

                    // Initial velocity (rotating plasma)
                    field.dimension_velocity[idx] = 0.2 * profile;
                }
            }
        }
    }

    println!("  Ball lightning created!");
}

fn main() {
    println!("======================================================================");
    println!("BALL LIGHTNING SIMULATION");
    println!("======================================================================");
    println!();
    println!("Exploring persistent toroidal plasma vortex as energy source");
    println!();

    // Grid setup
    let nx = 40;
    let ny = 40;
    let nz = 40;
    let spacing = 0.5;

    println!("Grid: {}×{}×{} cells", nx, ny, nz);
    println!("Spacing: {} units", spacing);
    println!("Domain: [{:.1}, {:.1}]³",
             -(nx as f32 * spacing) / 2.0,
             (nx as f32 * spacing) / 2.0);
    println!();

    let mut field = BallLightningField::new(nx, ny, nz, spacing);

    // Create Fibonacci-ratio ball lightning
    let center = Vec3::new(0.0, 0.0, 0.0);
    let major_radius = 6.0;  // 6 units
    let minor_radius = 2.0;   // 2 units
    let n_toroidal = 21;      // Fibonacci
    let n_poloidal = 13;      // Fibonacci

    create_ball_lightning(&mut field, center, major_radius, minor_radius,
                         n_toroidal, n_poloidal);

    let initial_energy = field.total_energy();
    let initial_core_energy = field.energy_in_sphere(center, minor_radius);

    println!();
    println!("Initial conditions:");
    println!("  Total energy: {:.6}", initial_energy);
    println!("  Core energy: {:.6}", initial_core_energy);
    println!();

    // Time evolution
    let dt = 0.05;
    let steps = 2000;
    let save_interval = 100;

    println!("Time evolution:");
    println!("  Timestep: {}", dt);
    println!("  Total steps: {}", steps);
    println!("  Total time: {}", steps as f32 * dt);
    println!();

    let mut energy_file = File::create("ball_lightning_energy.csv")
        .expect("Could not create file");
    writeln!(energy_file, "time,total_energy,core_energy,persistence")
        .expect("Could not write header");

    println!("Simulating ball lightning evolution...");
    println!();

    for step in 0..=steps {
        if step % save_interval == 0 {
            let progress = (step as f32 / steps as f32) * 100.0;
            let time = step as f32 * dt;

            let total_energy = field.total_energy();
            let core_energy = field.energy_in_sphere(center, minor_radius);

            // Persistence = how much energy remains vs initial
            let persistence = core_energy / initial_core_energy;

            println!("  t = {:.1}  |  Energy: {:.6}  |  Core: {:.6}  |  Persistence: {:.1}%",
                     time, total_energy, core_energy, persistence * 100.0);

            writeln!(energy_file, "{},{},{},{}",
                     time, total_energy, core_energy, persistence)
                .expect("Could not write data");
        }

        field.evolve(dt);
    }

    println!();
    println!("======================================================================");
    println!("BALL LIGHTNING ANALYSIS");
    println!("======================================================================");

    let final_energy = field.total_energy();
    let final_core_energy = field.energy_in_sphere(center, minor_radius);
    let persistence_time = steps as f32 * dt;

    println!();
    println!("Results:");
    println!("  Initial energy:     {:.6}", initial_energy);
    println!("  Final energy:       {:.6}", final_energy);
    println!("  Energy retained:    {:.1}%", (final_energy / initial_energy) * 100.0);
    println!("  Persistence time:   {:.1} time units", persistence_time);
    println!();

    // Calculate half-life (time for energy to drop to 50%)
    println!("Energy decay analysis:");
    println!("  If energy decays as E(t) = E₀ × e^(-t/τ)");
    let tau = -persistence_time / (final_core_energy / initial_core_energy).ln();
    println!("  Decay constant τ:   {:.1} time units", tau);
    println!("  Half-life:          {:.1} time units", tau * 0.693);
    println!();

    // Energy extraction estimate
    println!("Energy extraction potential:");
    let extractable_energy = final_core_energy * 0.8; // Can extract 80% before unstable
    println!("  Extractable energy: {:.6} simulation units", extractable_energy);
    println!("  Energy density:     {:.3} units/volume",
             extractable_energy / (4.0/3.0 * PI * minor_radius.powi(3)));
    println!();

    // Comparison to conventional energy storage
    println!("Comparison (if 1 sim unit = 1 kJ):");
    println!("  Extractable:        {:.2} kJ", extractable_energy);
    println!("  Energy density:     {:.0} kJ/m³",
             extractable_energy / (4.0/3.0 * PI * (minor_radius * 0.1).powi(3)));
    println!("  vs Lithium battery: ~100 kJ/m³");
    println!("  vs Gasoline:        ~30,000 kJ/m³");
    println!();

    println!("Analyze time-series: ball_lightning_energy.csv");
    println!("======================================================================");
}
