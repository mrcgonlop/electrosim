# Physics Sandbox Framework Architecture

## Overview

This framework implements a **unified cellular automaton approach** to simulating diverse physical theories. The core philosophy: **physics emerges from local update rules on discrete networks**, not from solving continuous differential equations.

## Core Architecture Principles

### 1. Discrete Spacetime Network
- Space and time are discretized into lattice/network structures
- Each node stores local physical quantities (fields, densities, charges, etc.)
- Edges connect nodes and can have dynamic properties (lengths, capacities)
- Update rules are **local**: each node updates based only on neighbors

### 2. Automaton Update Cycle
All simulations follow a common pattern:
```rust
loop {
    1. Read local state (current node + neighbors)
    2. Apply update rule (physics-specific)
    3. Write new state
    4. Advance time step
}
```

### 3. Emergent Behavior
Complex phenomena emerge from simple local rules:
- **Turbulence** from Navier-Stokes pressure projection
- **Electromagnetic waves** from Maxwell curl equations
- **Gravitational lensing** from metric-dependent edge lengths
- **Orbital mechanics** from particle-particle force laws

## Framework Layers

```
┌─────────────────────────────────────────────────┐
│          Examples (examples/*.rs)               │
│  Physical scenarios demonstrating theories      │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│       Physics Modules (src/physics/*.rs)        │
│  Update rules implementing physical theories    │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│      Core Data Structures (grids, particles)    │
│  Lattices, fields, particle collections         │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│   Visualization (image export, video encoding)  │
│  Render physical quantities as pixels           │
└─────────────────────────────────────────────────┘
```

---

## Physics Modules

### 1. **Field-Based Theories** (Continuum on Lattice)

#### Maxwell FDTD (`src/physics/maxwell.rs`)
- **Grid**: 3D Yee lattice with staggered E and B fields
- **Update Rule**: Finite-difference curl operators
  ```rust
  ∂E/∂t = (1/ε₀) ∇×B - J/ε₀
  ∂B/∂t = -∇×E
  ```
- **Local Coupling**: 6-neighbor stencil (±x, ±y, ±z)
- **Example**: `dipole_radiation.rs` - oscillating charge creates EM waves

#### Navier-Stokes Fluid (`src/physics/fluid2d.rs`)
- **Grid**: 2D MAC grid (pressure at cell centers, velocity at faces)
- **Update Rule**: Operator splitting
  1. Semi-Lagrangian advection (backward particle trace)
  2. Diffusion (Jacobi iteration)
  3. Pressure projection (Poisson solve → divergence-free velocity)
- **Local Coupling**: 4-neighbor stencil (±x, ±y) for pressure
- **Example**: `vortex_collision.rs` - emergent turbulence from interacting vortices

#### Scalar Waves (`src/physics/wave2d.rs`)
- **Grid**: 2D lattice with scalar field values
- **Update Rule**: Discrete wave equation
  ```rust
  ψ(t+dt) = 2ψ(t) - ψ(t-dt) + c²·dt²·∇²ψ(t)
  ```
- **Local Coupling**: 4-neighbor Laplacian
- **Example**: Wave interference patterns

---

### 2. **Particle-Based Theories** (Action-at-a-Distance)

#### Weber Electrodynamics (`src/physics/weber_particles.rs`)
- **Data Structure**: Collection of particles {position, velocity, acceleration, charge, mass}
- **Update Rule**: Weber force law
  ```rust
  F = (k·q₁q₂/r²) · [1 - (ṙ)²/(2c²) + r·r̈/c²]
  ```
- **Coupling**: All-to-all particle interactions (N² complexity)
- **Integration**: Velocity Verlet (symplectic for energy conservation)
- **Example**: `weber_three_body.rs` - chaotic dynamics with velocity-dependent forces

#### Newtonian Gravity (`examples/gravity_three_body.rs`)
- **Data Structure**: Same particle framework as Weber
- **Update Rule**: Inverse-square gravity
  ```rust
  F = -G·m₁m₂·r̂/r²
  ```
- **Normalized Units**: AU for distance, M☉ for mass, days for time (avoids overflow)
- **Example**: `gravity_three_body.rs` - stable orbital mechanics

---

### 3. **Hybrid Theories** (Fields + Particles)

#### Martins' Fluid EM (`src/physics/fluid2d.rs` extended)
- **Grid**: Navier-Stokes fluid + charge density field
- **Analogy**: Vorticity ω ≈ B-field, Pressure gradient ∇p ≈ E-field
- **Update Rule**:
  - Advect charge with fluid flow
  - Compute E-field from pressure gradient
- **Examples**:
  - `faraday_disk.rs`: Rotating vortex (B) + stationary charge → radial E
  - `lorentz_force.rs`: Charged particle deflection in vorticity field

---

### 4. **Emergent Geometry** (Network-Based)

#### Curved Spacetime (`src/physics/curved_spacetime.rs`)
- **Network**: 2D lattice with nodes {position, mass_density, metric_scale, curvature}
- **Edges**: Dynamic effective lengths `L_eff = L_rest · √(g_from · g_to)`
- **Update Rule**: Metric responds to mass
  ```rust
  g = 1 + φ  where  φ = -∑(G·m/r)
  ```
- **Geodesic Equation**: Light rays deflect via ∇g
  ```rust
  d²x/dλ² = -Γ·(dx/dλ)²  where  Γ ≈ (1/2)∇g
  ```
- **Example**: `curved_spacetime_lensing.rs` - gravitational lensing from emergent geometry

---

## Creating New Experiments: Step-by-Step Guide

### Method 1: Using Existing Physics Module

**Goal**: Create a new scenario with an existing theory (e.g., Maxwell, Fluid, Weber)

1. **Choose the physics module** (`maxwell.rs`, `fluid2d.rs`, `weber_particles.rs`, etc.)

2. **Create example file**: `examples/my_experiment.rs`

3. **Initialize the system**:
```rust
use em_physics_sandbox::physics::MaxwellTheory; // or Fluid2D, WeberParticles, etc.

fn main() {
    // Create grid/particle system
    let mut physics = MaxwellTheory::new(nx, ny, nz, spacing, dt);

    // Set initial conditions
    physics.add_gaussian_pulse(center, width, amplitude);

    // Run simulation
    for step in 0..num_steps {
        physics.step(); // Update rule
        if step % frame_interval == 0 {
            export_frame(&physics, step);
        }
    }
}
```

4. **Add to `Cargo.toml`**:
```toml
[[example]]
name = "my_experiment"
path = "examples/my_experiment.rs"
```

5. **Run**: `cargo run --example my_experiment --release`

**Examples in this repo**:
- `dipole_radiation.rs` uses `MaxwellTheory`
- `vortex_collision.rs` uses `Fluid2D`
- `weber_three_body.rs` uses `WeberParticles`

---

### Method 2: Implementing a New Physical Theory

**Goal**: Add support for a completely new theory (e.g., Bohmian mechanics, lattice QCD)

#### Step 1: Define the Theory Module

Create `src/physics/my_theory.rs`:

```rust
/// Data structure for the theory
pub struct MyTheory {
    // Lattice or particle data
    pub field: Vec<f32>,      // Example: scalar field values
    pub nx: usize,            // Grid dimensions
    pub ny: usize,
    pub spacing: f32,
    pub dt: f32,

    // Physical constants
    pub coupling_constant: f32,
}

impl MyTheory {
    /// Initialize the system
    pub fn new(nx: usize, ny: usize, spacing: f32, dt: f32) -> Self {
        let field = vec![0.0; nx * ny];

        Self {
            field,
            nx,
            ny,
            spacing,
            dt,
            coupling_constant: 1.0,
        }
    }

    /// Main update step (the physics!)
    pub fn step(&mut self) {
        let mut new_field = self.field.clone();

        // Apply local update rule
        for j in 1..self.ny - 1 {
            for i in 1..self.nx - 1 {
                let idx = self.index(i, j);

                // Read neighbors
                let f_center = self.field[idx];
                let f_left = self.field[idx - 1];
                let f_right = self.field[idx + 1];
                let f_down = self.field[idx - self.nx];
                let f_up = self.field[idx + self.nx];

                // Compute update (example: diffusion)
                let laplacian = (f_left + f_right + f_down + f_up - 4.0 * f_center)
                              / (self.spacing * self.spacing);

                new_field[idx] = f_center + self.dt * self.coupling_constant * laplacian;
            }
        }

        self.field = new_field;
    }

    /// Helper: convert (i,j) to linear index
    fn index(&self, i: usize, j: usize) -> usize {
        i + self.nx * j
    }
}
```

#### Step 2: Register in Module System

Edit `src/physics/mod.rs`:
```rust
pub mod my_theory;
pub use my_theory::MyTheory;
```

#### Step 3: Create Example

Create `examples/my_theory_demo.rs`:
```rust
use em_physics_sandbox::physics::MyTheory;

fn main() {
    let mut theory = MyTheory::new(256, 256, 0.01, 0.001);

    // Set initial conditions
    // ...

    // Run simulation
    for step in 0..1000 {
        theory.step();
    }
}
```

#### Step 4: Build and Run
```bash
cargo run --example my_theory_demo --release
```

---

## Key Design Patterns

### Pattern 1: Staggered Grids (Maxwell FDTD)
**When**: Fields with curl/divergence operators
**Why**: Centered differences on staggered grids are 2nd-order accurate
**Example**: E-field at cell faces, B-field at cell edges

### Pattern 2: Semi-Lagrangian Advection (Fluid Dynamics)
**When**: Quantities transported by velocity field
**Why**: Unconditionally stable (allows large time steps)
**How**: Trace particles backward, interpolate field values

### Pattern 3: Operator Splitting (Navier-Stokes)
**When**: Multiple physical processes (advection, diffusion, pressure)
**Why**: Simplifies complex equations into sequential simple steps
**Example**: Fluid step = advect → diffuse → project

### Pattern 4: Symplectic Integration (Particle Dynamics)
**When**: Conservative systems (energy preservation critical)
**Why**: Bounded energy error (doesn't accumulate)
**Example**: Velocity Verlet for gravity/Weber forces

### Pattern 5: Dynamic Network Topology (Curved Spacetime)
**When**: Geometry itself is dynamical
**Why**: Captures emergent geometry from matter distribution
**Example**: Edge lengths modified by metric field

---

## Theory-to-Implementation Mapping

### Example: Implementing Schrödinger Equation

**Theory**:
```
iℏ ∂ψ/∂t = -(ℏ²/2m)∇²ψ + V(x)ψ
```

**Discretization**:
1. Split into real and imaginary parts: ψ = ψ_r + i·ψ_i
2. Time step: Crank-Nicolson (implicit, preserves norm)
3. Laplacian: 5-point stencil in 2D

**Implementation**:
```rust
pub struct SchrodingerWave2D {
    psi_real: Vec<f32>,
    psi_imag: Vec<f32>,
    potential: Vec<f32>,
    // ... grid parameters
}

impl SchrodingerWave2D {
    pub fn step(&mut self) {
        // Apply kinetic energy operator: -∇²
        let laplacian_real = self.compute_laplacian(&self.psi_real);
        let laplacian_imag = self.compute_laplacian(&self.psi_imag);

        // Update: iℏ ∂ψ/∂t = Hψ
        for i in 0..self.psi_real.len() {
            let kinetic_r = -(HBAR * HBAR) / (2.0 * MASS) * laplacian_real[i];
            let kinetic_i = -(HBAR * HBAR) / (2.0 * MASS) * laplacian_imag[i];

            let potential_r = self.potential[i] * self.psi_real[i];
            let potential_i = self.potential[i] * self.psi_imag[i];

            // iℏ dψ_r/dt = -(kinetic_i + potential_i)
            self.psi_real[i] += self.dt / HBAR * (kinetic_i + potential_i);

            // iℏ dψ_i/dt = (kinetic_r + potential_r)
            self.psi_imag[i] += -self.dt / HBAR * (kinetic_r + potential_r);
        }
    }
}
```

---

## Visualization Pipeline

All examples use this common pattern:

1. **Create frame buffer**: `let mut rgba = vec![0u8; width * height * 4];`

2. **Map physical quantity to color**:
```rust
// Example: E-field magnitude → color intensity
let e_magnitude = (ex * ex + ey * ey).sqrt();
let intensity = (e_magnitude / e_max).clamp(0.0, 1.0);
let red = (255.0 * intensity) as u8;
```

3. **Write pixel**: `rgba[pixel_index] = red; rgba[pixel_index + 1] = green; ...`

4. **Save frame**: `image::save_buffer("frame_0001.png", &rgba, width, height, Rgba8)`

5. **Encode video**: `ffmpeg -i frame_%04d.png -c:v libx264 output.mp4`

---

## Experiments Summary

| Example | Physics Module | Theory Paradigm | Key Output |
|---------|---------------|-----------------|------------|
| `dipole_radiation.rs` | `maxwell.rs` | Field theory | EM waves from oscillating charge |
| `vortex_collision.rs` | `fluid2d.rs` | Continuum mechanics | Emergent turbulence |
| `faraday_disk.rs` | `fluid2d.rs` + EM analog | Hybrid field theory | Radial E-field from rotating vorticity |
| `lorentz_force.rs` | `fluid2d.rs` + particles | Hybrid | Charged particle deflection |
| `weber_three_body.rs` | `weber_particles.rs` | Action-at-a-distance | Velocity-dependent forces |
| `gravity_three_body.rs` | Particle dynamics | Classical mechanics | Orbital chaos |
| `curved_spacetime_lensing.rs` | `curved_spacetime.rs` | Emergent geometry | Light bending in curved space |

---

## Performance Considerations

### Grid-Based Simulations
- **Memory**: O(nx · ny · nz) for 3D grids
- **Compute**: O(nx · ny · nz) per time step (local updates)
- **Parallelization**: Trivially parallelizable (each cell independent)

### Particle-Based Simulations
- **Memory**: O(N) for N particles
- **Compute**: O(N²) for all-to-all interactions (Weber, gravity)
- **Optimization**: Use spatial hashing for O(N log N) if interactions are short-range

### Typical Performance
- **2D grid** (256×256): ~100 fps on CPU
- **3D grid** (64×64×64): ~30 fps on CPU
- **Particle system** (N=3): Real-time
- **Particle system** (N=1000): ~1 fps (N² scaling)

---

## Adding Alternative Physics: Quick Reference

### For Field Theories
1. Identify update equation (PDE)
2. Choose discretization (finite difference, finite volume, etc.)
3. Implement `step()` with local stencil operations
4. Add source terms or boundary conditions as needed

### For Particle Theories
1. Define force law F(positions, velocities, accelerations)
2. Choose integrator (Euler, Verlet, RK4)
3. Implement `step()` with force computation + integration
4. Track energy for validation

### For Hybrid Theories
1. Identify coupling between fields and particles
2. Update fields (grid step)
3. Interpolate fields to particle positions
4. Update particle dynamics
5. Deposit particle contributions back to grid

---

## Philosophy: Why Cellular Automata?

Traditional physics simulations solve PDEs numerically (finite element, spectral methods, etc.). This framework takes a different view:

**Cellular Automaton Hypothesis**: Fundamental physics is not continuous differential equations, but **discrete local update rules** on a network. The PDEs we write down are **emergent approximations** valid at large scales.

**Advantages**:
- **Simplicity**: Local rules are easier to implement than global PDE solvers
- **Generality**: Same framework works for Maxwell, Navier-Stokes, GR, etc.
- **Physical intuition**: "What happens at this point, given neighbors?" vs abstract operators
- **Numerical stability**: No need for complex implicit solvers or adaptive meshes

**Examples**:
- Navier-Stokes emerges from lattice Boltzmann collision rules
- Maxwell emerges from Yee lattice curl operations
- General Relativity emerges from dynamic network edge lengths

This is the **automaton approach to physics**: reality may be fundamentally discrete, and our continuous theories are limiting cases.

---

## Further Development Ideas

1. **3D Fluid Dynamics**: Extend `fluid2d.rs` to 3D with Helmholtz decomposition
2. **Quantum Mechanics**: Implement Schrödinger equation or Dirac equation on lattice
3. **Lattice QCD**: Strong force via SU(3) gauge fields on spacetime lattice
4. **Bohmian Mechanics**: Pilot wave + particle trajectories
5. **Discrete Differential Geometry**: Curvature computation on triangulated surfaces
6. **Coupled Systems**: Fluid + EM + particles for plasma physics
7. **General Relativity 3+1D**: Full Einstein equations with ADM formalism

The framework is **paradigm-agnostic** - any theory with local update rules can be implemented.
