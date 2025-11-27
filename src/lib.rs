//! # Electromagnetic Physics Sandbox
//!
//! A modular, GPU-accelerated voxel-based electromagnetic physics simulator
//! for testing alternative EM theories (Maxwell, Weber, fluid models, etc.).
//!
//! ## Core Modules
//!
//! - `physics`: EM theory traits and implementations
//! - `simulation`: Voxel grids, integrators, and boundary conditions
//! - `gpu`: GPU compute pipeline and shaders
//! - `experiments`: Predefined experimental setups
//! - `visualization`: Field rendering and UI
//! - `utils`: Mathematical utilities

pub mod physics;
pub mod simulation;
pub mod gpu;
pub mod experiments;
pub mod visualization;
pub mod utils;

// Re-export commonly used types
pub use physics::{EMField, EMTheory};
pub use simulation::VoxelGrid;

/// Result type for the library
pub type Result<T> = anyhow::Result<T>;

/// Application state and configuration
pub struct App {
    /// Active electromagnetic theory
    pub theory: Box<dyn EMTheory>,
    /// Simulation grid
    pub grid: VoxelGrid,
    /// Time step size
    pub dt: f32,
    /// Current simulation time
    pub time: f32,
}

impl App {
    /// Create a new application with the given theory and grid dimensions
    pub fn new(theory: Box<dyn EMTheory>, grid_size: (usize, usize, usize), spacing: f32) -> Self {
        Self {
            theory,
            grid: VoxelGrid::new(grid_size.0, grid_size.1, grid_size.2, spacing),
            dt: 0.001,
            time: 0.0,
        }
    }

    /// Step the simulation forward by one time step
    pub fn step(&mut self) {
        self.theory.update_fields(&mut self.grid, self.dt);
        self.time += self.dt;
    }

    /// Run the simulation for a specified number of steps
    pub fn run_steps(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        // This will be implemented once we have a concrete theory
        // Just ensuring the structure compiles
    }
}
