//! Faraday cage experiment
//!
//! Tests electromagnetic shielding by a conducting enclosure.
//! Expected result: External E field should not penetrate into the cage interior.

use crate::experiments::{Experiment, ExperimentResult};
use crate::simulation::VoxelGrid;
use glam::Vec3;

/// Faraday cage shielding experiment
///
/// Sets up a conducting box and applies an external electric field.
/// Verifies that the interior field is significantly reduced (shielded).
///
/// # Example
///
/// ```
/// use em_physics_sandbox::experiments::{FaradayCage, Experiment};
/// use em_physics_sandbox::simulation::VoxelGrid;
///
/// let experiment = FaradayCage::new(0.8, 100.0);
/// let mut grid = VoxelGrid::new(32, 32, 32, 0.01);
///
/// experiment.setup(&mut grid);
/// // ... run simulation ...
/// // let result = experiment.check_result(&grid);
/// ```
pub struct FaradayCage {
    /// Thickness of conducting walls (as fraction of grid)
    pub wall_thickness: f32,
    /// External E field strength (V/m)
    pub external_field: f32,
    /// Required shielding effectiveness (dB)
    pub required_shielding_db: f32,
}

impl FaradayCage {
    /// Create a new Faraday cage experiment
    ///
    /// # Arguments
    ///
    /// * `wall_thickness` - Thickness of walls as fraction of grid size (0.0-1.0)
    /// * `external_field` - Strength of external E field (V/m)
    ///
    /// # Example
    ///
    /// ```
    /// use em_physics_sandbox::experiments::FaradayCage;
    ///
    /// // 80% of grid is cage, 20% is walls
    /// let cage = FaradayCage::new(0.8, 100.0);
    /// ```
    pub fn new(wall_thickness: f32, external_field: f32) -> Self {
        Self {
            wall_thickness,
            external_field,
            required_shielding_db: 20.0, // Default: 20 dB shielding
        }
    }

    /// Set required shielding effectiveness in dB
    pub fn with_shielding(mut self, shielding_db: f32) -> Self {
        self.required_shielding_db = shielding_db;
        self
    }

    /// Calculate the average E field magnitude in the interior region
    fn interior_field_strength(&self, grid: &VoxelGrid) -> f32 {
        let wall_voxels = ((1.0 - self.wall_thickness) * grid.nx as f32 / 2.0) as usize;

        let i_min = wall_voxels;
        let i_max = grid.nx - wall_voxels;
        let j_min = wall_voxels;
        let j_max = grid.ny - wall_voxels;
        let k_min = wall_voxels;
        let k_max = grid.nz - wall_voxels;

        let mut sum = 0.0;
        let mut count = 0;

        for i in i_min..i_max {
            for j in j_min..j_max {
                for k in k_min..k_max {
                    let idx = grid.index(i, j, k);
                    sum += grid.e_field[idx].length();
                    count += 1;
                }
            }
        }

        if count > 0 {
            sum / count as f32
        } else {
            0.0
        }
    }

    /// Calculate the average E field magnitude in the exterior region
    fn exterior_field_strength(&self, grid: &VoxelGrid) -> f32 {
        let wall_voxels = ((1.0 - self.wall_thickness) * grid.nx as f32 / 2.0) as usize;

        let mut sum = 0.0;
        let mut count = 0;

        // Sample from corners (exterior)
        for i in 0..wall_voxels {
            for j in 0..wall_voxels {
                for k in 0..wall_voxels {
                    let idx = grid.index(i, j, k);
                    sum += grid.e_field[idx].length();
                    count += 1;
                }
            }
        }

        if count > 0 {
            sum / count as f32
        } else {
            0.0
        }
    }
}

impl Experiment for FaradayCage {
    fn name(&self) -> &str {
        "Faraday Cage Shielding"
    }

    fn setup(&self, grid: &mut VoxelGrid) {
        // Calculate wall thickness in voxels
        let wall_voxels = ((1.0 - self.wall_thickness) * grid.nx as f32 / 2.0) as usize;

        // Define conducting walls (hollow box)
        let i_min = wall_voxels;
        let i_max = grid.nx - wall_voxels - 1;
        let j_min = wall_voxels;
        let j_max = grid.ny - wall_voxels - 1;
        let k_min = wall_voxels;
        let k_max = grid.nz - wall_voxels - 1;

        // Set up conducting walls
        for i in 0..grid.nx {
            for j in 0..grid.ny {
                for k in 0..grid.nz {
                    let is_wall = i < i_min
                        || i > i_max
                        || j < j_min
                        || j > j_max
                        || k < k_min
                        || k > k_max;

                    if is_wall {
                        let idx = grid.index(i, j, k);
                        grid.sigma[idx] = 1e10; // High conductivity (conductor)
                    }
                }
            }
        }

        // Apply external uniform E field everywhere initially
        for i in 0..grid.nx {
            for j in 0..grid.ny {
                for k in 0..grid.nz {
                    let idx = grid.index(i, j, k);
                    grid.e_field[idx] = Vec3::new(self.external_field, 0.0, 0.0);
                }
            }
        }

        log::info!(
            "Faraday cage setup complete: wall_thickness={:.1}%, external_field={:.1} V/m",
            self.wall_thickness * 100.0,
            self.external_field
        );
    }

    fn check_result(&self, grid: &VoxelGrid) -> ExperimentResult {
        let interior = self.interior_field_strength(grid);
        let exterior = self.exterior_field_strength(grid);

        if exterior < 1e-6 {
            return ExperimentResult::Running;
        }

        // Calculate shielding effectiveness in dB
        let shielding_db = 20.0 * (exterior / interior.max(1e-10)).log10();

        log::info!(
            "Faraday cage results: interior={:.2e} V/m, exterior={:.2e} V/m, shielding={:.1} dB",
            interior,
            exterior,
            shielding_db
        );

        if shielding_db >= self.required_shielding_db {
            ExperimentResult::Pass
        } else {
            ExperimentResult::Fail(format!(
                "Insufficient shielding: {:.1} dB < {:.1} dB required",
                shielding_db, self.required_shielding_db
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faraday_cage_creation() {
        let cage = FaradayCage::new(0.8, 100.0);
        assert_eq!(cage.name(), "Faraday Cage Shielding");
        assert_eq!(cage.wall_thickness, 0.8);
        assert_eq!(cage.external_field, 100.0);
    }

    #[test]
    fn test_faraday_cage_setup() {
        let cage = FaradayCage::new(0.8, 100.0);
        let mut grid = VoxelGrid::new(32, 32, 32, 0.01);

        cage.setup(&mut grid);

        // Check that walls are conductive
        let wall_idx = grid.index(0, 0, 0);
        assert!(grid.sigma[wall_idx] > 1e9);

        // Check that exterior has E field
        assert!(grid.e_field[wall_idx].length() > 0.0);
    }

    #[test]
    fn test_shielding_measurement() {
        let cage = FaradayCage::new(0.6, 100.0);
        let mut grid = VoxelGrid::new(20, 20, 20, 0.01);

        cage.setup(&mut grid);

        // Initially, interior and exterior should be similar
        let interior_before = cage.interior_field_strength(&grid);
        let exterior_before = cage.exterior_field_strength(&grid);

        assert!(interior_before > 50.0);
        assert!(exterior_before > 50.0);

        // After simulation, interior should be much lower
        // (we're not running a full sim here, just testing the measurement)
    }

    #[test]
    fn test_experiment_result() {
        let result = ExperimentResult::Pass;
        assert!(result.is_pass());
        assert!(!result.is_fail());

        let result = ExperimentResult::Fail("test".to_string());
        assert!(!result.is_pass());
        assert!(result.is_fail());

        let result = ExperimentResult::Running;
        assert!(!result.is_pass());
        assert!(!result.is_fail());
    }
}
