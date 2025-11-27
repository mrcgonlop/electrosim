//! Core trait definition for electromagnetic theories

use crate::physics::EMField;
use crate::simulation::VoxelGrid;
use glam::Vec3;

/// Trait that all electromagnetic theories must implement.
///
/// This trait allows different EM theories (Maxwell, Weber, etc.) to be
/// swapped in and out while maintaining a consistent interface.
///
/// # Example
///
/// ```
/// use em_physics_sandbox::physics::{EMTheory, MaxwellTheory};
/// use em_physics_sandbox::simulation::VoxelGrid;
///
/// let theory = MaxwellTheory::new();
/// let mut grid = VoxelGrid::new(32, 32, 32, 0.01);
///
/// // Update the fields
/// theory.update_fields(&mut grid, 0.001);
/// ```
pub trait EMTheory: Send + Sync {
    /// Update electromagnetic fields for one time step using this theory's equations.
    ///
    /// # Arguments
    ///
    /// * `grid` - Mutable reference to the voxel grid containing field data
    /// * `dt` - Time step size in seconds
    fn update_fields(&self, grid: &mut VoxelGrid, dt: f32);

    /// Get the electromagnetic field values at a specific position.
    ///
    /// For positions between grid points, this should interpolate.
    ///
    /// # Arguments
    ///
    /// * `grid` - Reference to the voxel grid
    /// * `pos` - Position in space (meters)
    ///
    /// # Returns
    ///
    /// The interpolated EM field at the given position
    fn get_field_at(&self, grid: &VoxelGrid, pos: Vec3) -> EMField;

    /// Get the name of this theory for display and logging.
    ///
    /// # Example
    ///
    /// ```
    /// use em_physics_sandbox::physics::{EMTheory, MaxwellTheory};
    ///
    /// let theory = MaxwellTheory::new();
    /// assert_eq!(theory.name(), "Maxwell's Equations");
    /// ```
    fn name(&self) -> &str;

    /// Initialize the grid with theory-specific initial conditions.
    ///
    /// Default implementation does nothing. Override to set up
    /// specific field configurations.
    fn initialize(&self, _grid: &mut VoxelGrid) {
        // Default: do nothing
    }

    /// Calculate the total energy in the simulation.
    ///
    /// This can be used for energy conservation tests.
    fn total_energy(&self, grid: &VoxelGrid) -> f32 {
        let mut energy = 0.0;
        let volume = grid.spacing.powi(3);

        for i in 0..grid.nx {
            for j in 0..grid.ny {
                for k in 0..grid.nz {
                    let idx = grid.index(i, j, k);
                    if let Some(em_field) = grid.get_em_field(idx) {
                        energy += em_field.energy_density() * volume;
                    }
                }
            }
        }

        energy
    }

    /// Check if this theory conserves energy (approximately).
    ///
    /// # Arguments
    ///
    /// * `grid` - The voxel grid
    /// * `tolerance` - Relative tolerance for energy conservation
    ///
    /// # Returns
    ///
    /// True if energy is conserved within tolerance
    fn conserves_energy(&self, grid: &VoxelGrid, initial_energy: f32, tolerance: f32) -> bool {
        let current_energy = self.total_energy(grid);
        let relative_change = ((current_energy - initial_energy) / initial_energy).abs();
        relative_change < tolerance
    }
}

/// Trait for theories that support scalar wave equations (2D wave equation).
///
/// This is used for the simplified 2D wave simulator.
pub trait ScalarWaveTheory: Send + Sync {
    /// Update the scalar field for one time step
    fn update_scalar_field(&self, field: &mut [f32], velocity: &mut [f32], nx: usize, ny: usize, dx: f32, dt: f32);

    /// Get the name of this theory
    fn name(&self) -> &str;

    /// Calculate total energy in the scalar field
    fn total_energy(&self, field: &[f32], velocity: &[f32]) -> f32 {
        field.iter().zip(velocity.iter())
            .map(|(u, v)| 0.5 * (u.powi(2) + v.powi(2)))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock theory for testing
    struct MockTheory;

    impl EMTheory for MockTheory {
        fn update_fields(&self, _grid: &mut VoxelGrid, _dt: f32) {}

        fn get_field_at(&self, _grid: &VoxelGrid, _pos: Vec3) -> EMField {
            EMField::zero()
        }

        fn name(&self) -> &str {
            "Mock Theory"
        }
    }

    #[test]
    fn test_mock_theory() {
        let theory = MockTheory;
        assert_eq!(theory.name(), "Mock Theory");

        let mut grid = VoxelGrid::new(10, 10, 10, 0.1);
        theory.update_fields(&mut grid, 0.01);

        let field = theory.get_field_at(&grid, Vec3::ZERO);
        assert_eq!(field, EMField::zero());
    }
}
