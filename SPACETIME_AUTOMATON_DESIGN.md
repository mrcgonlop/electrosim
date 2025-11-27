# Spacetime Cellular Automaton Architecture

## Core Philosophy

**Spacetime is fundamental. Particles, fields, and fluids are emergent properties.**

Every voxel represents a unit of spacetime with:
- Geometric properties (position, metric, volume)
- Field properties (E, B, potentials)
- Fluid properties (velocity, density, pressure)
- Particle properties (charge density, matter density)
- Network properties (non-local connections)

## Unified Voxel Structure

```rust
// src/simulation/spacetime_cell.rs

use glam::{Vec3, Vec4};

/// Universal spacetime cell - the fundamental unit
#[derive(Debug, Clone)]
pub struct SpacetimeCell {
    // === GEOMETRIC ===
    /// Spacetime position (x, y, z, t)
    pub position: Vec4,

    /// Proper volume (can change with metric)
    pub volume: f32,

    /// Local metric tensor (for GR, simplified to scale factor)
    pub metric_scale: f32,

    // === FIELD STATE ===
    /// Electric field (V/m)
    pub e_field: Vec3,

    /// Magnetic field (T)
    pub b_field: Vec3,

    /// Electromagnetic 4-potential (φ, Ax, Ay, Az)
    pub potential: Vec4,

    // === FLUID STATE ===
    /// Fluid velocity (m/s) - vacuum/aether flow
    pub velocity: Vec3,

    /// Mass-energy density (kg/m³)
    pub density: f32,

    /// Pressure (Pa)
    pub pressure: f32,

    /// Vorticity ∇×v (1/s)
    pub vorticity: Vec3,

    /// Temperature (K) - optional thermodynamic state
    pub temperature: f32,

    // === PARTICLE STATE ===
    /// Charge density (C/m³)
    pub charge_density: f32,

    /// Current density (A/m²)
    pub current_density: Vec3,

    /// Particle count (discretized)
    pub particle_count: u32,

    // === MATERIAL PROPERTIES ===
    /// Permittivity (can vary with fluid density)
    pub epsilon: f32,

    /// Permeability (can vary with fluid properties)
    pub mu: f32,

    /// Conductivity (for lossy media)
    pub sigma: f32,
}

impl Default for SpacetimeCell {
    fn default() -> Self {
        Self {
            position: Vec4::ZERO,
            volume: 1.0,
            metric_scale: 1.0,

            e_field: Vec3::ZERO,
            b_field: Vec3::ZERO,
            potential: Vec4::ZERO,

            velocity: Vec3::ZERO,
            density: 1.0,  // Nominal vacuum density
            pressure: 0.0,
            vorticity: Vec3::ZERO,
            temperature: 300.0,

            charge_density: 0.0,
            current_density: Vec3::ZERO,
            particle_count: 0,

            epsilon: 8.854e-12,  // ε₀
            mu: 1.257e-6,        // μ₀
            sigma: 0.0,
        }
    }
}

impl SpacetimeCell {
    /// Check if this cell contains a "particle" (localized charge)
    pub fn is_particle(&self) -> bool {
        const PARTICLE_THRESHOLD: f32 = 1e-6;  // Coulombs/m³
        self.charge_density.abs() > PARTICLE_THRESHOLD
    }

    /// Total energy density at this cell
    pub fn energy_density(&self) -> f32 {
        // EM energy density
        let em_energy = 0.5 * (
            self.epsilon * self.e_field.length_squared() +
            self.b_field.length_squared() / self.mu
        );

        // Kinetic energy density (fluid)
        let kinetic_energy = 0.5 * self.density * self.velocity.length_squared();

        // Pressure energy
        let pressure_energy = self.pressure;

        em_energy + kinetic_energy + pressure_energy
    }

    /// Poynting vector (EM energy flow)
    pub fn poynting_vector(&self) -> Vec3 {
        self.e_field.cross(self.b_field) / self.mu
    }
}
```

## Network Structure

```rust
// src/simulation/spacetime_network.rs

use std::collections::HashMap;

/// The complete spacetime network
pub struct SpacetimeNetwork {
    /// All cells (regular 3D grid)
    pub cells: Vec<SpacetimeCell>,

    /// Grid dimensions
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub spacing: f32,

    /// Current simulation time
    pub time: f32,

    /// Non-local connections (Weber, entanglement, etc.)
    pub nonlocal_edges: Vec<NonLocalEdge>,

    /// Edge lookup table for fast queries
    pub edge_index: HashMap<(usize, usize), usize>,
}

/// Non-local edge connecting distant cells
#[derive(Debug, Clone)]
pub struct NonLocalEdge {
    /// Source cell index
    pub source: usize,

    /// Target cell index
    pub target: usize,

    /// Interaction strength
    pub coupling: f32,

    /// Type of interaction
    pub kind: InteractionKind,

    /// Time delay (for retarded interactions)
    pub delay: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionKind {
    /// Instantaneous Weber force
    WeberInstantaneous,

    /// Retarded (light-speed) interaction
    WeberRetarded,

    /// Quantum entanglement
    Entanglement,

    /// Speculative wormhole connection
    Wormhole,
}

impl SpacetimeNetwork {
    pub fn new(nx: usize, ny: usize, nz: usize, spacing: f32) -> Self {
        let size = nx * ny * nz;
        let mut cells = Vec::with_capacity(size);

        // Initialize cells with spacetime positions
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let pos = Vec4::new(
                        i as f32 * spacing,
                        j as f32 * spacing,
                        k as f32 * spacing,
                        0.0,  // Initial time
                    );

                    let mut cell = SpacetimeCell::default();
                    cell.position = pos;
                    cells.push(cell);
                }
            }
        }

        Self {
            cells,
            nx,
            ny,
            nz,
            spacing,
            time: 0.0,
            nonlocal_edges: Vec::new(),
            edge_index: HashMap::new(),
        }
    }

    /// Get cell index from 3D coordinates
    pub fn index(&self, i: usize, j: usize, k: usize) -> usize {
        i + self.nx * (j + self.ny * k)
    }

    /// Get 3D coordinates from cell index
    pub fn coords(&self, idx: usize) -> (usize, usize, usize) {
        let i = idx % self.nx;
        let j = (idx / self.nx) % self.ny;
        let k = idx / (self.nx * self.ny);
        (i, j, k)
    }

    /// Get 26 nearest neighbors (3D von Neumann + diagonals)
    pub fn neighbors(&self, idx: usize) -> Vec<usize> {
        let (i, j, k) = self.coords(idx);
        let mut neighbors = Vec::new();

        for dk in -1..=1i32 {
            for dj in -1..=1i32 {
                for di in -1..=1i32 {
                    if di == 0 && dj == 0 && dk == 0 {
                        continue;  // Skip self
                    }

                    let ni = i as i32 + di;
                    let nj = j as i32 + dj;
                    let nk = k as i32 + dk;

                    // Check bounds
                    if ni >= 0 && ni < self.nx as i32 &&
                       nj >= 0 && nj < self.ny as i32 &&
                       nk >= 0 && nk < self.nz as i32 {
                        neighbors.push(self.index(ni as usize, nj as usize, nk as usize));
                    }
                }
            }
        }

        neighbors
    }

    /// Add non-local connection between cells
    pub fn add_nonlocal_edge(&mut self, source: usize, target: usize,
                            coupling: f32, kind: InteractionKind) {
        let edge = NonLocalEdge {
            source,
            target,
            coupling,
            kind,
            delay: self.compute_light_travel_time(source, target),
        };

        let edge_idx = self.nonlocal_edges.len();
        self.nonlocal_edges.push(edge);
        self.edge_index.insert((source, target), edge_idx);
    }

    fn compute_light_travel_time(&self, source: usize, target: usize) -> f32 {
        let pos_s = self.cells[source].position.truncate();
        let pos_t = self.cells[target].position.truncate();
        let distance = (pos_t - pos_s).length();

        const C: f32 = 299_792_458.0;  // Speed of light
        distance / C
    }

    /// Automatically detect and create Weber edges for particles
    pub fn update_weber_graph(&mut self) {
        // Clear old edges
        self.nonlocal_edges.retain(|e| e.kind != InteractionKind::WeberInstantaneous);

        // Find all particles
        let particle_indices: Vec<usize> = self.cells.iter()
            .enumerate()
            .filter(|(_, cell)| cell.is_particle())
            .map(|(idx, _)| idx)
            .collect();

        // Create pairwise Weber connections
        for i in 0..particle_indices.len() {
            for j in (i+1)..particle_indices.len() {
                let idx_i = particle_indices[i];
                let idx_j = particle_indices[j];

                // Coupling strength proportional to charges
                let q_i = self.cells[idx_i].charge_density * self.cells[idx_i].volume;
                let q_j = self.cells[idx_j].charge_density * self.cells[idx_j].volume;
                let coupling = q_i * q_j;

                self.add_nonlocal_edge(idx_i, idx_j, coupling, InteractionKind::WeberInstantaneous);
            }
        }
    }
}
```

## Update Rules

```rust
// src/simulation/update_rules.rs

use super::{SpacetimeCell, SpacetimeNetwork};

/// Universal update rule trait
pub trait UpdateRule: Send + Sync {
    /// Compute next state for a cell given its neighbors
    fn update(&self, cell: &SpacetimeCell, neighbors: &[&SpacetimeCell], dt: f32) -> SpacetimeCell;

    /// Name of this rule
    fn name(&self) -> &str;
}

/// Unified physics update rule
pub struct UnifiedPhysicsRule {
    /// Enable Maxwell field evolution
    pub enable_maxwell: bool,

    /// Enable fluid dynamics
    pub enable_fluid: bool,

    /// Enable Weber particle interactions
    pub enable_weber: bool,

    /// Cross-coupling constants
    pub coupling: CrossCoupling,
}

#[derive(Debug, Clone)]
pub struct CrossCoupling {
    /// Vorticity → B field: B = k_omega * ω
    pub vorticity_to_magnetic: f32,

    /// Pressure gradient → E field: E = -k_p * ∇p
    pub pressure_to_electric: f32,

    /// EM stress → Fluid force
    pub em_to_fluid: f32,

    /// Density → Permittivity: ε = ε₀ * (1 + k_rho * δρ)
    pub density_to_epsilon: f32,
}

impl Default for CrossCoupling {
    fn default() -> Self {
        Self {
            vorticity_to_magnetic: 1.0,
            pressure_to_electric: 1.0,
            em_to_fluid: 1.0,
            density_to_epsilon: 0.1,
        }
    }
}

impl UpdateRule for UnifiedPhysicsRule {
    fn update(&self, cell: &SpacetimeCell, neighbors: &[&SpacetimeCell], dt: f32) -> SpacetimeCell {
        let mut new_cell = cell.clone();

        // 1. Maxwell evolution (if enabled)
        if self.enable_maxwell {
            self.update_maxwell(&mut new_cell, neighbors, dt);
        }

        // 2. Fluid evolution (if enabled)
        if self.enable_fluid {
            self.update_fluid(&mut new_cell, neighbors, dt);
        }

        // 3. Cross-coupling (always active)
        self.apply_cross_coupling(&mut new_cell, neighbors);

        // 4. Update derived quantities
        new_cell.vorticity = self.compute_vorticity(neighbors);
        new_cell.current_density = new_cell.charge_density * new_cell.velocity;

        new_cell
    }

    fn name(&self) -> &str {
        "Unified Physics (Maxwell + Fluid + Weber)"
    }
}

impl UnifiedPhysicsRule {
    fn update_maxwell(&self, cell: &mut SpacetimeCell, neighbors: &[&SpacetimeCell], dt: f32) {
        // FDTD Maxwell update (simplified - full version in maxwell.rs)

        // Compute curl of B (from neighbors)
        let curl_b = self.compute_curl_b(neighbors);

        // Compute curl of E (from neighbors)
        let curl_e = self.compute_curl_e(neighbors);

        // Faraday's law: ∂B/∂t = -∇×E
        cell.b_field -= dt * curl_e;

        // Ampere-Maxwell: ∂E/∂t = (1/ε) * (∇×B/μ - J)
        let j_total = cell.current_density + cell.sigma * cell.e_field;
        cell.e_field += dt * (curl_b / cell.mu - j_total) / cell.epsilon;
    }

    fn update_fluid(&self, cell: &mut SpacetimeCell, neighbors: &[&SpacetimeCell], dt: f32) {
        // Navier-Stokes evolution (simplified)

        // Advection term: (v·∇)v
        let advection = self.compute_advection(cell, neighbors);

        // Pressure gradient: -∇p/ρ
        let pressure_grad = self.compute_pressure_gradient(neighbors) / cell.density;

        // Viscous term: ν∇²v
        let viscosity = 1e-6;  // Very low for vacuum
        let laplacian_v = self.compute_laplacian_velocity(neighbors);

        // External forcing (EM stress)
        let em_force = self.compute_em_forcing(cell);

        // Update velocity: ∂v/∂t = -(v·∇)v - ∇p/ρ + ν∇²v + F_em
        cell.velocity += dt * (-advection - pressure_grad + viscosity * laplacian_v + em_force);

        // Update density: ∂ρ/∂t = -∇·(ρv)
        let flux_divergence = self.compute_flux_divergence(neighbors);
        cell.density -= dt * flux_divergence;

        // Equation of state: p = c_s² ρ
        const SOUND_SPEED: f32 = 3e8;  // Speed of light
        cell.pressure = SOUND_SPEED * SOUND_SPEED * cell.density;
    }

    fn apply_cross_coupling(&self, cell: &mut SpacetimeCell, neighbors: &[&SpacetimeCell]) {
        // Martins' hypothesis: Vorticity → Magnetic field
        cell.b_field = self.coupling.vorticity_to_magnetic * cell.vorticity;

        // Pressure gradient → Electric field (computed from neighbors)
        let pressure_grad = self.compute_pressure_gradient(neighbors);
        cell.e_field = -self.coupling.pressure_to_electric * pressure_grad;

        // Density affects permittivity (vacuum polarization)
        let rho_0 = 1.0;  // Nominal vacuum density
        let delta_rho = cell.density - rho_0;
        cell.epsilon = 8.854e-12 * (1.0 + self.coupling.density_to_epsilon * delta_rho);
    }

    fn compute_curl_b(&self, neighbors: &[&SpacetimeCell]) -> Vec3 {
        // Finite difference curl (simplified)
        // In full version: use Yee lattice staggering
        Vec3::ZERO  // Placeholder
    }

    fn compute_curl_e(&self, neighbors: &[&SpacetimeCell]) -> Vec3 {
        Vec3::ZERO  // Placeholder
    }

    fn compute_vorticity(&self, neighbors: &[&SpacetimeCell]) -> Vec3 {
        // ω = ∇×v
        Vec3::ZERO  // Placeholder - need neighbor velocities
    }

    fn compute_advection(&self, cell: &SpacetimeCell, neighbors: &[&SpacetimeCell]) -> Vec3 {
        // (v·∇)v term
        Vec3::ZERO  // Placeholder
    }

    fn compute_pressure_gradient(&self, neighbors: &[&SpacetimeCell]) -> Vec3 {
        // Central difference gradient
        Vec3::ZERO  // Placeholder
    }

    fn compute_laplacian_velocity(&self, neighbors: &[&SpacetimeCell]) -> Vec3 {
        // ∇²v using 7-point stencil
        Vec3::ZERO  // Placeholder
    }

    fn compute_flux_divergence(&self, neighbors: &[&SpacetimeCell]) -> f32 {
        // ∇·(ρv)
        0.0  // Placeholder
    }

    fn compute_em_forcing(&self, cell: &SpacetimeCell) -> Vec3 {
        // EM stress tensor creates force on fluid
        // F = ε₀(E·∇)E + (1/μ₀)(B·∇)B
        Vec3::ZERO  // Simplified
    }
}
```

## Simulation Engine

```rust
// src/simulation/spacetime_engine.rs

use super::{SpacetimeNetwork, UpdateRule};

pub struct SpacetimeEngine {
    pub network: SpacetimeNetwork,
    pub rule: Box<dyn UpdateRule>,
    pub dt: f32,
}

impl SpacetimeEngine {
    pub fn new(network: SpacetimeNetwork, rule: Box<dyn UpdateRule>, dt: f32) -> Self {
        Self { network, rule, dt }
    }

    /// Main simulation step
    pub fn step(&mut self) {
        // 1. Local update (using cellular automaton rule)
        let new_cells = self.local_update();

        // 2. Non-local update (Weber, entanglement)
        self.nonlocal_update();

        // 3. Update Weber graph (detect new particles)
        self.network.update_weber_graph();

        // 4. Commit new states
        self.network.cells = new_cells;

        // 5. Advance time
        self.network.time += self.dt;
        for cell in &mut self.network.cells {
            cell.position.w = self.network.time;
        }
    }

    fn local_update(&self) -> Vec<SpacetimeCell> {
        let mut new_cells = Vec::with_capacity(self.network.cells.len());

        for idx in 0..self.network.cells.len() {
            let cell = &self.network.cells[idx];
            let neighbor_indices = self.network.neighbors(idx);
            let neighbors: Vec<&SpacetimeCell> = neighbor_indices.iter()
                .map(|&i| &self.network.cells[i])
                .collect();

            let new_cell = self.rule.update(cell, &neighbors, self.dt);
            new_cells.push(new_cell);
        }

        new_cells
    }

    fn nonlocal_update(&mut self) {
        // Weber forces between particles
        for edge in &self.network.nonlocal_edges {
            if edge.kind == InteractionKind::WeberInstantaneous {
                let force = self.compute_weber_force(edge);

                // Apply to both cells
                let idx_s = edge.source;
                let idx_t = edge.target;

                let m_s = self.network.cells[idx_s].density *
                         self.network.cells[idx_s].volume;
                let m_t = self.network.cells[idx_t].density *
                         self.network.cells[idx_t].volume;

                self.network.cells[idx_s].velocity += self.dt * force / m_s;
                self.network.cells[idx_t].velocity -= self.dt * force / m_t;
            }
        }
    }

    fn compute_weber_force(&self, edge: &NonLocalEdge) -> Vec3 {
        let cell_s = &self.network.cells[edge.source];
        let cell_t = &self.network.cells[edge.target];

        let r_vec = cell_t.position.truncate() - cell_s.position.truncate();
        let r = r_vec.length();
        if r < 1e-10 {
            return Vec3::ZERO;
        }

        let r_hat = r_vec / r;
        let v_rel = cell_t.velocity - cell_s.velocity;

        // Simplified Weber: F = qq'/r² [1 - (v_rel·r̂)²/2c²]
        const C: f32 = 3e8;
        let velocity_correction = 1.0 - v_rel.dot(r_hat).powi(2) / (2.0 * C * C);

        let force_mag = edge.coupling / (r * r) * velocity_correction;
        force_mag * r_hat
    }
}
```

## Next Steps: Implementation Plan

1. **Phase 1** (Week 1): Core infrastructure
   - [ ] Implement `SpacetimeCell`
   - [ ] Implement `SpacetimeNetwork`
   - [ ] Basic visualization (cell states as colored voxels)

2. **Phase 2** (Week 2): Local update rules
   - [ ] Maxwell update (existing FDTD adapted)
   - [ ] Fluid update (Navier-Stokes)
   - [ ] Cross-coupling

3. **Phase 3** (Week 3): Non-local graph
   - [ ] Weber edge detection
   - [ ] Weber force calculation
   - [ ] Graph visualization

4. **Phase 4** (Week 4): Integration & testing
   - [ ] Unified physics rule
   - [ ] Validation against known results
   - [ ] Performance optimization

Would you like me to start implementing this architecture?
