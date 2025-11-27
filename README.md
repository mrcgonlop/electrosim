# Electromagnetic Physics Sandbox

A modular, GPU-accelerated electromagnetic physics simulation framework for testing alternative EM theories (Maxwell, Weber, fluid models, etc.). Built in Rust with a focus on testability, performance, and extensibility.

## Features

- **Modular Theory System**: Swap between different EM theories (Maxwell, Weber, etc.) via trait-based architecture
- **2D Wave Equation**: Simplified starting point for testing numerical methods
- **3D FDTD Maxwell Solver**: Full 3D finite-difference time-domain implementation on Yee lattice
- **Visualization**: PNG frame export and MP4 video generation with multiple colormaps
- **GPU Acceleration**: WGPU compute shader pipeline (in progress)
- **Historical Experiments**: Pre-configured setups (Faraday cage, etc.) with automated validation
- **Comprehensive Testing**: Unit tests, integration tests, physics validation tests, and benchmarks
- **Performance**: Optimized for speed with optional CPU/GPU execution paths

## Quick Start

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Optional: ffmpeg (for MP4 video export)
  - Windows: `winget install ffmpeg`
  - Mac: `brew install ffmpeg`
  - Linux: `sudo apt install ffmpeg`
- Optional: GPU with Vulkan/Metal/DX12 support for GPU acceleration

### Build and Run

```bash
# Clone the repository
git clone <repository-url>
cd em-physics-sandbox

# Build the project
cargo build --release

# Run the demo
cargo run --release

# Run tests
cargo test --all

# Run benchmarks
cargo bench
```

### Visualization

```bash
# Export PNG frames (fast preview)
cargo run --example export_frames --release

# Generate MP4 video (requires ffmpeg)
cargo run --example export_video --release

# Text-based real-time viewer
cargo run --example simple_viewer --release
```

See [VISUALIZATION_GUIDE.md](VISUALIZATION_GUIDE.md) for detailed visualization instructions.

## Project Structure

```
em-physics-sandbox/
├── src/
│   ├── physics/           # EM theory implementations
│   │   ├── em_theory.rs   # Core trait definition
│   │   ├── field.rs       # Field data structures
│   │   ├── maxwell.rs     # Maxwell's equations (FDTD)
│   │   ├── weber.rs       # Weber electrodynamics (stub)
│   │   └── wave2d.rs      # 2D wave equation
│   ├── simulation/        # Simulation infrastructure
│   │   ├── voxel_grid.rs  # 3D voxel grid
│   │   ├── integrator.rs  # Time-stepping methods
│   │   └── boundary.rs    # Boundary conditions
│   ├── gpu/               # GPU compute pipeline
│   │   ├── compute.rs     # Compute setup
│   │   └── shaders/       # WGSL shaders
│   ├── experiments/       # Pre-configured experiments
│   │   └── faraday_cage.rs
│   ├── visualization/     # 3D rendering (in progress)
│   └── utils/             # Math utilities
├── tests/                 # Integration and validation tests
├── benches/              # Performance benchmarks
└── Cargo.toml
```

## Usage Examples

### Running a 2D Wave Simulation

```rust
use em_physics_sandbox::physics::{Wave2D, ScalarWaveTheory};

let wave = Wave2D::new(1.0); // wave speed = 1.0 m/s
let nx = 128;
let ny = 128;
let mut field = vec![0.0; nx * ny];
let mut velocity = vec![0.0; nx * ny];

// Set Gaussian pulse
wave.set_gaussian_pulse(&mut field, (64, 64), 1.0, 8.0, nx, ny);

// Simulate
for _ in 0..100 {
    wave.update_scalar_field(&mut field, &mut velocity, nx, ny, 0.1, 0.01);
}
```

### Running a 3D Maxwell FDTD Simulation

```rust
use em_physics_sandbox::physics::{MaxwellTheory, EMTheory};
use em_physics_sandbox::simulation::VoxelGrid;

let maxwell = MaxwellTheory::new();
let mut grid = VoxelGrid::new(64, 64, 64, 0.01); // 64³ grid, 1cm spacing

// Initialize with Gaussian pulse
maxwell.initialize(&mut grid);

// Simulate
let dt = 1e-12; // 1 picosecond
for _ in 0..100 {
    maxwell.update_fields(&mut grid, dt);
}

// Calculate total energy
let energy = maxwell.total_energy(&grid);
println!("Total energy: {:.6e} J", energy);
```

### Running the Faraday Cage Experiment

```rust
use em_physics_sandbox::{
    experiments::{FaradayCage, Experiment},
    physics::{MaxwellTheory, EMTheory},
    simulation::VoxelGrid,
};

let maxwell = MaxwellTheory::new();
let cage = FaradayCage::new(0.8, 100.0); // 80% cage, 100 V/m external field
let mut grid = VoxelGrid::new(32, 32, 32, 0.01);

// Setup experiment
cage.setup(&mut grid);

// Run simulation
for _ in 0..200 {
    maxwell.update_fields(&mut grid, 1e-12);
}

// Check results
let result = cage.check_result(&grid);
println!("Experiment result: {:?}", result);
```

## Adding a New EM Theory

To implement a new electromagnetic theory:

1. Create a new file in `src/physics/` (e.g., `my_theory.rs`)

2. Implement the `EMTheory` trait:

```rust
use crate::physics::{EMTheory, EMField};
use crate::simulation::VoxelGrid;
use glam::Vec3;

pub struct MyTheory {
    // Your theory's parameters
}

impl EMTheory for MyTheory {
    fn update_fields(&self, grid: &mut VoxelGrid, dt: f32) {
        // Implement your field update equations
    }

    fn get_field_at(&self, grid: &VoxelGrid, pos: Vec3) -> EMField {
        // Return field at position
        grid.interpolate_field(pos)
    }

    fn name(&self) -> &str {
        "My Theory"
    }
}
```

3. Add tests in the same file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_theory() {
        let theory = MyTheory::new();
        // Test your theory
    }
}
```

4. Export from `src/physics/mod.rs`:

```rust
pub mod my_theory;
pub use my_theory::MyTheory;
```

## Testing

The project includes comprehensive testing at multiple levels:

### Unit Tests

Test individual components in isolation:

```bash
cargo test
```

### Physics Validation Tests

Verify physical laws are obeyed:

```bash
cargo test --test physics_validation
```

Key validation tests:
- Energy conservation
- Wave speed matches c
- E and B perpendicularity in plane waves
- Gauss's law (∇·E = 0 in vacuum)
- Conductor boundary conditions

### Integration Tests

Test complete workflows:

```bash
cargo test --test integration_tests
```

### Benchmarks

Measure performance:

```bash
cargo bench
```

Benchmarks track:
- Field update speed (voxels/second)
- Energy calculation time
- Memory access patterns
- Scaling with grid size

## Configuration

Key simulation parameters and their stability constraints:

### CFL Condition

For stability, the Courant-Friedrichs-Lewy (CFL) condition must be satisfied:

**2D Wave Equation**: `c * dt / dx ≤ 1/√2 ≈ 0.707`

**3D Maxwell FDTD**: `c * dt / dx ≤ 1/√3 ≈ 0.577`

Where:
- `c` = wave speed (m/s)
- `dt` = time step (s)
- `dx` = grid spacing (m)

### Grid Resolution

For accurate wave representation:
- Minimum: 10 points per wavelength
- Recommended: 20+ points per wavelength

### Conductor Representation

Conductors are modeled by high conductivity (σ > 10^9 S/m) which zeros out fields.

## Performance

Typical performance on modern hardware (3.5 GHz CPU):

| Grid Size | Voxels | Updates/sec | Mvoxel-updates/s |
|-----------|--------|-------------|------------------|
| 16³       | 4K     | ~5000       | ~20              |
| 32³       | 32K    | ~600        | ~20              |
| 64³       | 262K   | ~75         | ~20              |
| 128³      | 2M     | ~10         | ~20              |

GPU acceleration (TODO) should provide 10-100× speedup.

## Roadmap

### Near-term (Current Focus)
- ✅ 2D wave equation with tests
- ✅ 3D Maxwell FDTD solver
- ✅ Faraday cage experiment
- ✅ Comprehensive test suite
- ✅ Performance benchmarks

### Medium-term
- [ ] Full GPU compute pipeline
- [ ] 3D field visualization
- [ ] Absorbing boundary conditions (Mur's ABC)
- [ ] Current sources and antennas
- [ ] More historical experiments (Hertz, Tesla coil)

### Long-term
- [ ] Weber electrodynamics implementation
- [ ] Fluid EM models
- [ ] Particle-field hybrid methods
- [ ] Real-time interactive visualization
- [ ] Distributed/parallel execution

## Contributing

Contributions are welcome! Areas of particular interest:

1. **Physics**: Implement alternative EM theories
2. **Performance**: Optimize hot loops, GPU kernels
3. **Visualization**: 3D rendering, real-time updates
4. **Experiments**: Historical experiment setups
5. **Testing**: More validation tests, edge cases

## Architecture Decisions

### Why Rust?

- **Performance**: Zero-cost abstractions, no GC pauses
- **Safety**: Memory safety without runtime overhead
- **Concurrency**: Fearless parallelism for multi-core
- **GPU**: Native wgpu support for compute shaders

### Why Trait-Based Architecture?

The `EMTheory` trait allows:
- Clean separation of theory from infrastructure
- Easy comparison between theories on same setup
- Testing theories independently
- Runtime theory switching

### Why FDTD for Maxwell?

Finite-Difference Time-Domain:
- Simple and robust
- Explicit time-stepping (no matrix inversion)
- Natural parallelization
- Well-studied stability conditions
- Direct E and B field evolution

## Known Limitations

1. **Numerical Dissipation**: FDTD introduces some energy loss over time
2. **Boundary Reflections**: Perfect conductor boundaries cause reflections (absorbing BC needed)
3. **Dispersion**: Numerical dispersion affects high-frequency waves
4. **Grid Resolution**: Fine features require fine grids (memory intensive)

## References

- Taflove & Hagness, "Computational Electrodynamics: The Finite-Difference Time-Domain Method"
- Yee, K.S., "Numerical solution of initial boundary value problems" (1966)
- Weber, W., "Elektrodynamische Maassbestimmungen" (1846)
- Jackson, "Classical Electrodynamics"

## License

MIT OR Apache-2.0

## Citation

If you use this in research, please cite:

```bibtex
@software{em_physics_sandbox,
  title={Electromagnetic Physics Sandbox},
  author={Your Name},
  year={2024},
  url={https://github.com/yourusername/em-physics-sandbox}
}
```

## Contact

For questions, issues, or contributions:
- GitHub Issues: [link]
- Email: [your-email]
