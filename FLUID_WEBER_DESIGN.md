# Design Document: Fluid EM & Weber Electrodynamics Support

## Executive Summary

The current FDTD field-based framework is **insufficient** for:
1. **Alexandre Martins' Fluidic Electromagnetism** - requires vacuum as compressible fluid
2. **Weber Electrodynamics** - requires particle-based action-at-a-distance

This document proposes a **phased refactoring** to support multiple simulation paradigms while maintaining the existing Maxwell FDTD functionality.

---

## Current Architecture Analysis

### What Works Well
- ✅ Clean trait-based `EMTheory` abstraction
- ✅ Comprehensive testing infrastructure
- ✅ Modular `VoxelGrid` for field storage
- ✅ Visualization pipeline
- ✅ GPU compute foundation

### Critical Limitations
- ❌ **No fluid state** (velocity, density, pressure)
- ❌ **No advection** (material transport by flow)
- ❌ **No particle system** (needed for Weber)
- ❌ **Vacuum is passive** (just ε₀, μ₀ - no dynamics)
- ❌ **Grid-locked** (can't do action-at-a-distance efficiently)

---

## Proposed Architecture: Multi-Paradigm Physics Engine

### Core Abstraction

```rust
/// Universal physics engine trait
pub trait PhysicsEngine: Send + Sync {
    /// Advance simulation by timestep
    fn step(&mut self, dt: f32);

    /// Get observable quantities for visualization/analysis
    fn get_observables(&self) -> Observables;

    /// Get engine-specific diagnostics
    fn diagnostics(&self) -> EngineDiagnostics;
}

/// Observable quantities (theory-agnostic)
pub struct Observables {
    pub em_fields: Option<FieldData>,      // For field theories
    pub fluid_state: Option<FluidData>,    // For fluid theories
    pub particles: Option<ParticleData>,   // For particle theories
    pub energy: f32,
    pub momentum: Vec3,
}
```

### Three Simulation Paradigms

```rust
pub enum SimulationMode {
    /// Field-based (Current Maxwell FDTD)
    Field(FieldEngine),

    /// Computational Fluid Dynamics (Martins' fluid EM)
    Fluid(FluidEngine),

    /// N-body particle simulation (Weber electrodynamics)
    Particle(ParticleEngine),

    /// Hybrid coupling (e.g., particles + background fluid)
    Hybrid(HybridEngine),
}
```

---

## Phase 1: Fluid State Extension (Weeks 1-2)

### Goal
Add fluid dynamics capability to existing grid framework.

### Implementation

#### 1. Extend `VoxelGrid` with Fluid State

```rust
// src/simulation/fluid_grid.rs

/// Extended grid with fluid state
#[derive(Debug, Clone)]
pub struct FluidVoxelGrid {
    /// Base grid (existing EM fields)
    pub base: VoxelGrid,

    /// Fluid velocity field (m/s)
    pub velocity: Vec<Vec3>,

    /// Fluid density (kg/m³) - vacuum density
    pub density: Vec<f32>,

    /// Pressure field (Pa) - related to EM energy density
    pub pressure: Vec<f32>,

    /// Vorticity field ω = ∇×v (potentially related to B)
    pub vorticity: Vec<Vec3>,

    /// Temperature (if needed for thermodynamic models)
    pub temperature: Vec<f32>,
}

impl FluidVoxelGrid {
    pub fn new(nx: usize, ny: usize, nz: usize, spacing: f32) -> Self {
        let size = nx * ny * nz;
        Self {
            base: VoxelGrid::new(nx, ny, nz, spacing),
            velocity: vec![Vec3::ZERO; size],
            density: vec![1.0; size],  // Nominal vacuum density
            pressure: vec![0.0; size],
            vorticity: vec![Vec3::ZERO; size],
            temperature: vec![300.0; size],
        }
    }

    /// Compute vorticity from velocity field
    pub fn update_vorticity(&mut self) {
        // ω = ∇×v using finite differences
        for k in 1..self.base.nz-1 {
            for j in 1..self.base.ny-1 {
                for i in 1..self.base.nx-1 {
                    let idx = i + self.base.nx * (j + self.base.ny * k);

                    // Central differences for curl
                    let dx_inv = 1.0 / self.base.spacing;

                    let v_xp = self.velocity[idx + 1];
                    let v_xm = self.velocity[idx - 1];
                    let v_yp = self.velocity[idx + self.base.nx];
                    let v_ym = self.velocity[idx - self.base.nx];
                    let v_zp = self.velocity[idx + self.base.nx * self.base.ny];
                    let v_zm = self.velocity[idx - self.base.nx * self.base.ny];

                    let curl = Vec3::new(
                        (v_zp.y - v_zm.y - v_yp.z + v_ym.z) * dx_inv * 0.5,
                        (v_xp.z - v_xm.z - v_zp.x + v_zm.x) * dx_inv * 0.5,
                        (v_yp.x - v_ym.x - v_xp.y + v_xm.y) * dx_inv * 0.5,
                    );

                    self.vorticity[idx] = curl;
                }
            }
        }
    }
}
```

#### 2. Implement Fluid Dynamics Solver

```rust
// src/physics/fluid_dynamics.rs

/// Navier-Stokes solver for compressible fluid
pub struct NavierStokesSolver {
    /// Kinematic viscosity (m²/s)
    pub nu: f32,
    /// Speed of sound in medium (m/s)
    pub sound_speed: f32,
    /// Bulk viscosity
    pub bulk_viscosity: f32,
}

impl NavierStokesSolver {
    /// Compressible Navier-Stokes equations:
    /// ∂ρ/∂t + ∇·(ρv) = 0                    (continuity)
    /// ∂(ρv)/∂t + ∇·(ρv⊗v) = -∇p + μ∇²v + F (momentum)
    /// p = c_s² ρ                             (barotropic EOS)

    pub fn step(&self, grid: &mut FluidVoxelGrid, dt: f32, forcing: &[Vec3]) {
        // 1. Advection step (semi-Lagrangian or upwind)
        self.advect_velocity(grid, dt);

        // 2. Pressure projection (incompressibility or compressible correction)
        self.compute_pressure(grid);

        // 3. Viscous diffusion
        self.apply_viscosity(grid, dt);

        // 4. External forcing (EM forces, gravity, etc.)
        self.apply_forcing(grid, dt, forcing);

        // 5. Update density from continuity equation
        self.update_density(grid, dt);

        // 6. Compute derived quantities
        grid.update_vorticity();
    }

    fn advect_velocity(&self, grid: &mut FluidVoxelGrid, dt: f32) {
        // Semi-Lagrangian advection: trace particles backward
        let mut new_velocity = grid.velocity.clone();

        for k in 1..grid.base.nz-1 {
            for j in 1..grid.base.ny-1 {
                for i in 1..grid.base.nx-1 {
                    let idx = i + grid.base.nx * (j + grid.base.ny * k);
                    let pos = Vec3::new(
                        i as f32 * grid.base.spacing,
                        j as f32 * grid.base.spacing,
                        k as f32 * grid.base.spacing,
                    );

                    // Trace back along velocity
                    let back_pos = pos - grid.velocity[idx] * dt;

                    // Interpolate velocity at back_pos
                    new_velocity[idx] = self.interpolate_velocity(grid, back_pos);
                }
            }
        }

        grid.velocity = new_velocity;
    }

    fn compute_pressure(&self, grid: &mut FluidVoxelGrid) {
        // Barotropic equation of state: p = c_s² ρ
        for i in 0..grid.pressure.len() {
            grid.pressure[i] = self.sound_speed * self.sound_speed * grid.density[i];
        }
    }

    fn apply_viscosity(&self, grid: &mut FluidVoxelGrid, dt: f32) {
        // Diffusion: ∂v/∂t = ν∇²v
        let mut laplacian = vec![Vec3::ZERO; grid.velocity.len()];

        let dx2_inv = 1.0 / (grid.base.spacing * grid.base.spacing);

        for k in 1..grid.base.nz-1 {
            for j in 1..grid.base.ny-1 {
                for i in 1..grid.base.nx-1 {
                    let idx = i + grid.base.nx * (j + grid.base.ny * k);

                    // 7-point stencil for Laplacian
                    let lap = (
                        grid.velocity[idx + 1] + grid.velocity[idx - 1] +
                        grid.velocity[idx + grid.base.nx] + grid.velocity[idx - grid.base.nx] +
                        grid.velocity[idx + grid.base.nx * grid.base.ny] +
                        grid.velocity[idx - grid.base.nx * grid.base.ny] -
                        6.0 * grid.velocity[idx]
                    ) * dx2_inv;

                    laplacian[idx] = lap;
                }
            }
        }

        // Update velocity with viscous term
        for i in 0..grid.velocity.len() {
            grid.velocity[i] += self.nu * dt * laplacian[i];
        }
    }

    fn apply_forcing(&self, grid: &mut FluidVoxelGrid, dt: f32, forcing: &[Vec3]) {
        for i in 0..grid.velocity.len() {
            grid.velocity[i] += dt * forcing[i] / grid.density[i];
        }
    }

    fn update_density(&self, grid: &mut FluidVoxelGrid, dt: f32) {
        // ∂ρ/∂t = -∇·(ρv)
        let mut drho_dt = vec![0.0; grid.density.len()];
        let dx_inv = 1.0 / grid.base.spacing;

        for k in 1..grid.base.nz-1 {
            for j in 1..grid.base.ny-1 {
                for i in 1..grid.base.nx-1 {
                    let idx = i + grid.base.nx * (j + grid.base.ny * k);

                    // Compute flux divergence ∇·(ρv)
                    let flux_x = (
                        grid.density[idx + 1] * grid.velocity[idx + 1].x -
                        grid.density[idx - 1] * grid.velocity[idx - 1].x
                    ) * dx_inv * 0.5;

                    let flux_y = (
                        grid.density[idx + grid.base.nx] * grid.velocity[idx + grid.base.nx].y -
                        grid.density[idx - grid.base.nx] * grid.velocity[idx - grid.base.nx].y
                    ) * dx_inv * 0.5;

                    let flux_z = (
                        grid.density[idx + grid.base.nx * grid.base.ny] *
                            grid.velocity[idx + grid.base.nx * grid.base.ny].z -
                        grid.density[idx - grid.base.nx * grid.base.ny] *
                            grid.velocity[idx - grid.base.nx * grid.base.ny].z
                    ) * dx_inv * 0.5;

                    drho_dt[idx] = -(flux_x + flux_y + flux_z);
                }
            }
        }

        // Update density
        for i in 0..grid.density.len() {
            grid.density[i] += dt * drho_dt[i];
        }
    }

    fn interpolate_velocity(&self, grid: &FluidVoxelGrid, pos: Vec3) -> Vec3 {
        // Trilinear interpolation (reuse existing grid method)
        grid.base.interpolate_field(&grid.velocity, pos)
    }
}
```

#### 3. Martins' Fluid EM Theory

```rust
// src/physics/martins_fluid_em.rs

/// Alexandre Martins' fluidic representation of electromagnetism
///
/// Key hypothesis: EM phenomena emerge from vacuum fluid dynamics
/// - E field ~ pressure gradient: E ∝ -∇p
/// - B field ~ vorticity: B ∝ ∇×v
/// - Charge ~ density perturbation: ρ_charge ∝ δρ
/// - Light ~ acoustic/vortex waves in fluid
pub struct MartinsFluidEM {
    fluid_solver: NavierStokesSolver,

    /// Coupling constants
    pub pressure_to_efield: f32,  // E = k_p * (-∇p)
    pub vorticity_to_bfield: f32, // B = k_ω * ω
    pub density_to_charge: f32,   // q = k_ρ * δρ
}

impl MartinsFluidEM {
    pub fn new() -> Self {
        Self {
            fluid_solver: NavierStokesSolver {
                nu: 1e-6,           // Very low viscosity (inviscid vacuum)
                sound_speed: 3e8,   // Speed of light
                bulk_viscosity: 0.0,
            },
            pressure_to_efield: 1.0,
            vorticity_to_bfield: 1.0,
            density_to_charge: 1.0,
        }
    }

    /// Convert fluid state to EM observables
    pub fn fluid_to_em_fields(&self, grid: &FluidVoxelGrid) -> Vec<EMField> {
        let mut em_fields = Vec::with_capacity(grid.base.e_field.len());

        for k in 1..grid.base.nz-1 {
            for j in 1..grid.base.ny-1 {
                for i in 1..grid.base.nx-1 {
                    let idx = i + grid.base.nx * (j + grid.base.ny * k);

                    // E field from pressure gradient
                    let e_field = self.compute_electric_from_pressure(grid, i, j, k);

                    // B field from vorticity
                    let b_field = self.vorticity_to_bfield * grid.vorticity[idx];

                    em_fields.push(EMField {
                        electric: e_field,
                        magnetic: b_field,
                    });
                }
            }
        }

        em_fields
    }

    fn compute_electric_from_pressure(&self, grid: &FluidVoxelGrid, i: usize, j: usize, k: usize) -> Vec3 {
        let idx = i + grid.base.nx * (j + grid.base.ny * k);
        let dx_inv = 1.0 / grid.base.spacing;

        // E = -k_p * ∇p (like electric field from potential)
        let grad_p = Vec3::new(
            (grid.pressure[idx + 1] - grid.pressure[idx - 1]) * dx_inv * 0.5,
            (grid.pressure[idx + grid.base.nx] - grid.pressure[idx - grid.base.nx]) * dx_inv * 0.5,
            (grid.pressure[idx + grid.base.nx * grid.base.ny] -
             grid.pressure[idx - grid.base.nx * grid.base.ny]) * dx_inv * 0.5,
        );

        -self.pressure_to_efield * grad_p
    }

    /// Implement EMTheory trait for compatibility
    pub fn update_fields(&self, grid: &mut FluidVoxelGrid, dt: f32) {
        // Compute EM forces from current fluid state
        let em_forcing = self.compute_em_forcing(grid);

        // Step fluid dynamics with EM forcing
        self.fluid_solver.step(grid, dt, &em_forcing);

        // Update EM fields from new fluid state
        let em_fields = self.fluid_to_em_fields(grid);
        grid.base.e_field = em_fields.iter().map(|f| f.electric).collect();
        grid.base.b_field = em_fields.iter().map(|f| f.magnetic).collect();
    }

    fn compute_em_forcing(&self, grid: &FluidVoxelGrid) -> Vec<Vec3> {
        // EM stress tensor creates forces on fluid
        // This is the self-consistent coupling
        vec![Vec3::ZERO; grid.velocity.len()] // Placeholder
    }
}
```

---

## Phase 2: Weber Particle System (Weeks 3-4)

### Goal
Implement N-body particle simulation for Weber electrodynamics.

### Implementation

```rust
// src/physics/weber.rs

use glam::Vec3;

/// Charged particle for Weber electrodynamics
#[derive(Debug, Clone)]
pub struct ChargedParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub charge: f32,  // Coulombs
    pub mass: f32,    // kg
}

/// Weber electrodynamics N-body simulator
pub struct WeberTheory {
    pub particles: Vec<ChargedParticle>,
    pub c: f32,  // Speed of light

    /// History buffer for retarded interactions
    pub history: ParticleHistory,
}

impl WeberTheory {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            c: 299_792_458.0,
            history: ParticleHistory::new(),
        }
    }

    /// Weber force between two charges (action-at-a-distance)
    ///
    /// F = (q₁q₂/r²) [1 - (v_rel·r̂)²/(2c²) + r·a_rel/c²] r̂
    ///
    /// Key features:
    /// - Depends on relative velocity (unlike Coulomb)
    /// - Depends on relative acceleration (unlike Maxwell)
    /// - Instantaneous action-at-a-distance (or finite speed variant)
    pub fn weber_force(&self, i: usize, j: usize) -> Vec3 {
        let pi = &self.particles[i];
        let pj = &self.particles[j];

        let r_vec = pj.position - pi.position;
        let r = r_vec.length();

        if r < 1e-10 {
            return Vec3::ZERO;  // Avoid singularity
        }

        let r_hat = r_vec / r;

        // Relative velocity and acceleration
        let v_rel = pj.velocity - pi.velocity;
        let a_rel = pj.acceleration - pi.acceleration;

        // Weber force formula
        let coulomb_term = 1.0;
        let velocity_term = -(v_rel.dot(r_hat)).powi(2) / (2.0 * self.c * self.c);
        let acceleration_term = r * a_rel.dot(r_hat) / (self.c * self.c);

        let force_magnitude = (pi.charge * pj.charge) / (r * r) *
                             (coulomb_term + velocity_term + acceleration_term);

        force_magnitude * r_hat
    }

    /// Update all particles using Weber forces
    pub fn step(&mut self, dt: f32) {
        let n = self.particles.len();
        let mut forces = vec![Vec3::ZERO; n];

        // Compute all pairwise forces O(N²)
        for i in 0..n {
            for j in (i+1)..n {
                let f_ij = self.weber_force(i, j);
                forces[i] += f_ij;
                forces[j] -= f_ij;  // Newton's third law
            }
        }

        // Update positions and velocities (Velocity Verlet)
        for i in 0..n {
            let p = &mut self.particles[i];

            // Update position
            p.position += p.velocity * dt + 0.5 * p.acceleration * dt * dt;

            // Update velocity (using new acceleration)
            let new_accel = forces[i] / p.mass;
            p.velocity += 0.5 * (p.acceleration + new_accel) * dt;
            p.acceleration = new_accel;
        }
    }

    /// Convert particle system to field representation (for visualization)
    pub fn particles_to_fields(&self, grid: &mut VoxelGrid) {
        // Interpolate particle charges to grid for visualization
        // This is just for compatibility with field-based viz

        grid.e_field.fill(Vec3::ZERO);

        for particle in &self.particles {
            // Use cloud-in-cell (CIC) interpolation
            self.deposit_particle_to_grid(particle, grid);
        }
    }

    fn deposit_particle_to_grid(&self, particle: &ChargedParticle, grid: &mut VoxelGrid) {
        // CIC interpolation: distribute particle to 8 nearest grid points
        let pos_grid = particle.position / grid.spacing;
        let i = pos_grid.x.floor() as usize;
        let j = pos_grid.y.floor() as usize;
        let k = pos_grid.z.floor() as usize;

        // Fractional position within cell
        let fx = pos_grid.x - i as f32;
        let fy = pos_grid.y - j as f32;
        let fz = pos_grid.z - k as f32;

        // Deposit to 8 corners with linear weights
        let weights = [
            (1.0 - fx) * (1.0 - fy) * (1.0 - fz),
            fx * (1.0 - fy) * (1.0 - fz),
            (1.0 - fx) * fy * (1.0 - fz),
            fx * fy * (1.0 - fz),
            (1.0 - fx) * (1.0 - fy) * fz,
            fx * (1.0 - fy) * fz,
            (1.0 - fx) * fy * fz,
            fx * fy * fz,
        ];

        let offsets = [
            (0, 0, 0), (1, 0, 0), (0, 1, 0), (1, 1, 0),
            (0, 0, 1), (1, 0, 1), (0, 1, 1), (1, 1, 1),
        ];

        for (w, (di, dj, dk)) in weights.iter().zip(offsets.iter()) {
            if let Some(idx) = grid.index_checked(i + di, j + dj, k + dk) {
                // Simplified: just mark charge density
                grid.e_field[idx] += Vec3::splat(particle.charge * w);
            }
        }
    }
}

/// History buffer for retarded interactions (if implementing finite-speed Weber)
pub struct ParticleHistory {
    positions: Vec<Vec<Vec3>>,
    velocities: Vec<Vec<Vec3>>,
    max_history: usize,
}

impl ParticleHistory {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            max_history: 1000,  // Store ~1000 timesteps
        }
    }

    pub fn record(&mut self, particles: &[ChargedParticle]) {
        // Store current state for retarded calculations
        self.positions.push(particles.iter().map(|p| p.position).collect());
        self.velocities.push(particles.iter().map(|p| p.velocity).collect());

        // Trim old history
        if self.positions.len() > self.max_history {
            self.positions.remove(0);
            self.velocities.remove(0);
        }
    }
}
```

---

## Phase 3: Lattice Boltzmann Alternative (Optional, Weeks 5-6)

For a more sophisticated fluid EM approach:

```rust
// src/physics/lbm.rs

/// Lattice Boltzmann Method for fluid EM
///
/// Uses D3Q19 lattice (3D, 19 velocity directions)
pub struct LBMEngine {
    /// Distribution functions f_i(x,t)
    pub distributions: Vec<[f32; 19]>,

    /// Lattice velocities (unit vectors in 19 directions)
    pub lattice_velocities: [[i32; 3]; 19],

    /// Weights for equilibrium
    pub weights: [f32; 19],

    /// Relaxation parameter (related to viscosity)
    pub tau: f32,
}

impl LBMEngine {
    pub fn new(nx: usize, ny: usize, nz: usize) -> Self {
        // D3Q19 lattice velocities
        let velocities = [
            [0, 0, 0],   // Rest
            [1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1],  // 6 face neighbors
            [1, 1, 0], [-1, -1, 0], [1, -1, 0], [-1, 1, 0],  // 12 edge neighbors
            [1, 0, 1], [-1, 0, -1], [1, 0, -1], [-1, 0, 1],
            [0, 1, 1], [0, -1, -1], [0, 1, -1], [0, -1, 1],
        ];

        let weights = [
            1.0/3.0,  // Rest
            1.0/18.0, 1.0/18.0, 1.0/18.0, 1.0/18.0, 1.0/18.0, 1.0/18.0,  // Faces
            1.0/36.0, 1.0/36.0, 1.0/36.0, 1.0/36.0,  // Edges
            1.0/36.0, 1.0/36.0, 1.0/36.0, 1.0/36.0,
            1.0/36.0, 1.0/36.0, 1.0/36.0, 1.0/36.0,
        ];

        Self {
            distributions: vec![[0.0; 19]; nx * ny * nz],
            lattice_velocities: velocities,
            weights,
            tau: 0.6,  // Stability: 0.5 < tau < 2.0
        }
    }

    /// LBM collision-streaming cycle
    pub fn step(&mut self, nx: usize, ny: usize, nz: usize) {
        // 1. Collision step (local, GPU-friendly)
        self.collide();

        // 2. Streaming step (advection along lattice links)
        self.stream(nx, ny, nz);
    }

    fn collide(&mut self) {
        for cell in &mut self.distributions {
            // Compute macroscopic quantities
            let (rho, u) = self.compute_macroscopic(cell);

            // Compute equilibrium distribution
            let f_eq = self.equilibrium(rho, u);

            // BGK collision operator: f_new = f - (f - f_eq)/tau
            for i in 0..19 {
                cell[i] += (f_eq[i] - cell[i]) / self.tau;
            }
        }
    }

    fn stream(&mut self, nx: usize, ny: usize, nz: usize) {
        let mut new_dist = self.distributions.clone();

        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let idx = i + nx * (j + ny * k);

                    for dir in 0..19 {
                        let ci = self.lattice_velocities[dir];
                        let ni = (i as i32 + ci[0]) as usize;
                        let nj = (j as i32 + ci[1]) as usize;
                        let nk = (k as i32 + ci[2]) as usize;

                        if ni < nx && nj < ny && nk < nz {
                            let nidx = ni + nx * (nj + ny * nk);
                            new_dist[nidx][dir] = self.distributions[idx][dir];
                        }
                    }
                }
            }
        }

        self.distributions = new_dist;
    }

    fn compute_macroscopic(&self, f: &[f32; 19]) -> (f32, Vec3) {
        // Density: ρ = Σ f_i
        let rho: f32 = f.iter().sum();

        // Velocity: ρu = Σ f_i c_i
        let mut u = Vec3::ZERO;
        for i in 0..19 {
            let ci = Vec3::new(
                self.lattice_velocities[i][0] as f32,
                self.lattice_velocities[i][1] as f32,
                self.lattice_velocities[i][2] as f32,
            );
            u += f[i] * ci;
        }
        u /= rho;

        (rho, u)
    }

    fn equilibrium(&self, rho: f32, u: Vec3) -> [f32; 19] {
        let mut f_eq = [0.0; 19];
        let u_sq = u.length_squared();

        for i in 0..19 {
            let ci = Vec3::new(
                self.lattice_velocities[i][0] as f32,
                self.lattice_velocities[i][1] as f32,
                self.lattice_velocities[i][2] as f32,
            );

            let ci_dot_u = ci.dot(u);

            // Maxwell-Boltzmann equilibrium
            f_eq[i] = self.weights[i] * rho * (
                1.0 + 3.0 * ci_dot_u +
                4.5 * ci_dot_u * ci_dot_u -
                1.5 * u_sq
            );
        }

        f_eq
    }
}
```

---

## Recommended Implementation Path

### Immediate (This Week)
1. ✅ **Read this document thoroughly**
2. **Choose path**: Fluid extension vs. Weber particles vs. LBM
3. **Prototype decision**: Start with simplest (fluid extension)

### Phase 1 (Weeks 1-2): Fluid Extension
- [ ] Create `FluidVoxelGrid` in `src/simulation/fluid_grid.rs`
- [ ] Implement `NavierStokesSolver` in `src/physics/fluid_dynamics.rs`
- [ ] Implement `MartinsFluidEM` in `src/physics/martins_fluid_em.rs`
- [ ] Add fluid state to visualization (velocity arrows, vorticity)
- [ ] Write tests comparing fluid EM to Maxwell FDTD

### Phase 2 (Weeks 3-4): Weber Particles
- [ ] Create `ChargedParticle` and `WeberTheory` in `src/physics/weber.rs`
- [ ] Implement O(N²) force calculation
- [ ] Add particle-to-grid interpolation for visualization
- [ ] Benchmark performance (optimize later with Barnes-Hut tree)
- [ ] Test against known Weber experiments (two-body interactions)

### Phase 3 (Optional): LBM
- [ ] Implement D3Q19 LBM in `src/physics/lbm.rs`
- [ ] GPU acceleration for collision-streaming
- [ ] Couple LBM to EM via source terms
- [ ] Performance comparison with Navier-Stokes

### Phase 4: Integration
- [ ] Unified `PhysicsEngine` trait
- [ ] Hybrid mode: particles in background fluid
- [ ] Cross-validation between theories
- [ ] Performance benchmarking suite

---

## Key Design Principles

1. **Keep Maxwell FDTD intact** - it's working and tested
2. **Trait abstraction** - multiple theories behind same interface
3. **Test-driven** - validate each theory against known results
4. **Modular** - can use fluid without particles, etc.
5. **GPU-ready** - design for eventual GPU acceleration

---

## References & Next Steps

### For Martins' Fluid EM:
- Read Alexandre Martins' papers on fluid vacuum models
- Study vortex dynamics and their relation to magnetism
- Implement simple test: vortex ring should behave like current loop

### For Weber Electrodynamics:
- Read Weber's original force law derivation
- Study Assis & others' modern treatments
- Test case: two charges in relative motion

### Recommended Reading:
- "Computational Fluid Dynamics" - Anderson
- "Lattice Boltzmann Method" - Succi
- "Weber's Electrodynamics" - Assis
- "The Lattice Boltzmann Equation" - Krüger et al.

---

## Questions for You

1. **Which theory is highest priority?** Martins fluid EM or Weber?
2. **Scale**: How many particles for Weber? Grid size for fluid?
3. **Performance target**: Real-time? Batch processing?
4. **Validation**: Do you have experimental data to compare against?
5. **Coupling**: Should fluid and particles interact in hybrid mode?

Let me know your priorities and I can start implementing immediately!
