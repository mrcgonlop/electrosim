//! Maxwell's equations implementation using FDTD (Finite-Difference Time-Domain)
//!
//! Implements Maxwell's equations on a Yee lattice:
//! ∇ × E = -∂B/∂t
//! ∇ × B = μ₀ε₀ ∂E/∂t + μ₀J

use crate::physics::{EMField, EMTheory};
use crate::simulation::VoxelGrid;
use glam::Vec3;

/// Maxwell's equations solver using FDTD with Yee lattice.
///
/// The Yee lattice staggers E and B fields in space for numerical stability:
/// - E components are at cell edges
/// - B components are at cell faces
///
/// Time integration uses leapfrog:
/// - E and B are offset by half a time step
///
/// # Example
///
/// ```
/// use em_physics_sandbox::physics::MaxwellTheory;
/// use em_physics_sandbox::simulation::VoxelGrid;
///
/// let theory = MaxwellTheory::new();
/// let mut grid = VoxelGrid::new(64, 64, 64, 0.01);
///
/// // Set initial plane wave
/// theory.initialize_plane_wave(&mut grid, 1e8, glam::Vec3::X);
///
/// // Simulate
/// for _ in 0..100 {
///     theory.update_fields(&mut grid, 1e-10);
/// }
/// ```
pub struct MaxwellTheory {
    /// Speed of light in vacuum (m/s)
    pub c: f32,
    /// Vacuum permittivity (F/m)
    pub epsilon_0: f32,
    /// Vacuum permeability (H/m)
    pub mu_0: f32,
}

impl MaxwellTheory {
    /// Create a new Maxwell theory solver with physical constants.
    ///
    /// # Example
    ///
    /// ```
    /// use em_physics_sandbox::physics::MaxwellTheory;
    ///
    /// let maxwell = MaxwellTheory::new();
    /// assert_eq!(maxwell.name(), "Maxwell's Equations");
    /// ```
    pub fn new() -> Self {
        const C: f32 = 2.998e8; // m/s
        const EPSILON_0: f32 = 8.854e-12; // F/m
        const MU_0: f32 = 1.257e-6; // H/m

        Self {
            c: C,
            epsilon_0: EPSILON_0,
            mu_0: MU_0,
        }
    }

    /// Calculate the curl of E field using finite differences.
    ///
    /// ∇ × E = (∂Ez/∂y - ∂Ey/∂z, ∂Ex/∂z - ∂Ez/∂x, ∂Ey/∂x - ∂Ex/∂y)
    fn curl_e(&self, grid: &VoxelGrid, i: usize, j: usize, k: usize) -> Vec3 {
        let dx = grid.spacing;

        // Get E field at neighboring points
        let idx = grid.index(i, j, k);
        let e_center = grid.e_field[idx];

        // x-component: ∂Ez/∂y - ∂Ey/∂z
        let curl_x = if j < grid.ny - 1 && k < grid.nz - 1 {
            let ez_y = grid.e_field[grid.index(i, j + 1, k)].z - e_center.z;
            let ey_z = grid.e_field[grid.index(i, j, k + 1)].y - e_center.y;
            (ez_y - ey_z) / dx
        } else {
            0.0
        };

        // y-component: ∂Ex/∂z - ∂Ez/∂x
        let curl_y = if i < grid.nx - 1 && k < grid.nz - 1 {
            let ex_z = grid.e_field[grid.index(i, j, k + 1)].x - e_center.x;
            let ez_x = grid.e_field[grid.index(i + 1, j, k)].z - e_center.z;
            (ex_z - ez_x) / dx
        } else {
            0.0
        };

        // z-component: ∂Ey/∂x - ∂Ex/∂y
        let curl_z = if i < grid.nx - 1 && j < grid.ny - 1 {
            let ey_x = grid.e_field[grid.index(i + 1, j, k)].y - e_center.y;
            let ex_y = grid.e_field[grid.index(i, j + 1, k)].x - e_center.x;
            (ey_x - ex_y) / dx
        } else {
            0.0
        };

        Vec3::new(curl_x, curl_y, curl_z)
    }

    /// Calculate the curl of B field using finite differences.
    ///
    /// ∇ × B = (∂Bz/∂y - ∂By/∂z, ∂Bx/∂z - ∂Bz/∂x, ∂By/∂x - ∂Bx/∂y)
    fn curl_b(&self, grid: &VoxelGrid, i: usize, j: usize, k: usize) -> Vec3 {
        let dx = grid.spacing;

        let idx = grid.index(i, j, k);
        let b_center = grid.b_field[idx];

        // Similar structure to curl_e
        let curl_x = if j < grid.ny - 1 && k < grid.nz - 1 {
            let bz_y = grid.b_field[grid.index(i, j + 1, k)].z - b_center.z;
            let by_z = grid.b_field[grid.index(i, j, k + 1)].y - b_center.y;
            (bz_y - by_z) / dx
        } else {
            0.0
        };

        let curl_y = if i < grid.nx - 1 && k < grid.nz - 1 {
            let bx_z = grid.b_field[grid.index(i, j, k + 1)].x - b_center.x;
            let bz_x = grid.b_field[grid.index(i + 1, j, k)].z - b_center.z;
            (bx_z - bz_x) / dx
        } else {
            0.0
        };

        let curl_z = if i < grid.nx - 1 && j < grid.ny - 1 {
            let by_x = grid.b_field[grid.index(i + 1, j, k)].y - b_center.y;
            let bx_y = grid.b_field[grid.index(i, j + 1, k)].x - b_center.x;
            (by_x - bx_y) / dx
        } else {
            0.0
        };

        Vec3::new(curl_x, curl_y, curl_z)
    }

    /// Initialize a plane wave in the grid.
    ///
    /// # Arguments
    ///
    /// * `grid` - The voxel grid to initialize
    /// * `frequency` - Wave frequency (Hz)
    /// * `direction` - Normalized propagation direction
    pub fn initialize_plane_wave(&self, grid: &mut VoxelGrid, frequency: f32, direction: Vec3) {
        let k = 2.0 * std::f32::consts::PI * frequency / self.c;
        let omega = 2.0 * std::f32::consts::PI * frequency;

        let k_vec = direction.normalize() * k;

        // E perpendicular to k
        let e_dir = if direction.x.abs() < 0.9 {
            Vec3::X.cross(direction).normalize()
        } else {
            Vec3::Y.cross(direction).normalize()
        };

        // B perpendicular to both k and E
        let b_dir = direction.cross(e_dir).normalize();

        for i in 0..grid.nx {
            for j in 0..grid.ny {
                for k in 0..grid.nz {
                    let pos = grid.position(i, j, k);
                    let phase = k_vec.dot(pos);

                    let idx = grid.index(i, j, k);

                    grid.e_field[idx] = e_dir * phase.sin();
                    grid.b_field[idx] = b_dir * phase.sin() / self.c;
                }
            }
        }
    }

    /// Initialize a Gaussian pulse.
    ///
    /// # Arguments
    ///
    /// * `grid` - The voxel grid to initialize
    /// * `center` - Center position of pulse
    /// * `width` - Width of Gaussian (meters)
    /// * `amplitude` - Peak electric field amplitude (V/m)
    pub fn initialize_gaussian_pulse(
        &self,
        grid: &mut VoxelGrid,
        center: Vec3,
        width: f32,
        amplitude: f32,
    ) {
        for i in 0..grid.nx {
            for j in 0..grid.ny {
                for k in 0..grid.nz {
                    let pos = grid.position(i, j, k);
                    let r = (pos - center).length();
                    let gaussian = amplitude * (-(r * r) / (2.0 * width * width)).exp();

                    let idx = grid.index(i, j, k);

                    // E field in x direction
                    grid.e_field[idx] = Vec3::new(gaussian, 0.0, 0.0);
                    // B field initially zero
                    grid.b_field[idx] = Vec3::ZERO;
                }
            }
        }
    }
}

impl Default for MaxwellTheory {
    fn default() -> Self {
        Self::new()
    }
}

impl EMTheory for MaxwellTheory {
    fn update_fields(&self, grid: &mut VoxelGrid, dt: f32) {
        // CFL stability condition: c * dt < dx / √3
        let cfl = self.c * dt / grid.spacing;
        assert!(
            cfl < 0.577, // 1/√3 ≈ 0.577
            "CFL condition violated: c*dt/dx = {} >= 1/√3. Reduce dt or increase spacing.",
            cfl
        );

        // Create temporary buffers for the new fields
        let mut new_e = grid.e_field.clone();
        let mut new_b = grid.b_field.clone();

        // FDTD update using leapfrog integration
        // Half-step B update: B^(n+1/2) = B^(n-1/2) - dt * ∇×E^n
        for i in 1..grid.nx - 1 {
            for j in 1..grid.ny - 1 {
                for k in 1..grid.nz - 1 {
                    let idx = grid.index(i, j, k);

                    // Check if this is a conductor
                    if grid.sigma[idx] > 1e6 {
                        // High conductivity -> zero fields
                        new_e[idx] = Vec3::ZERO;
                        new_b[idx] = Vec3::ZERO;
                        continue;
                    }

                    let curl_e = self.curl_e(grid, i, j, k);
                    new_b[idx] = grid.b_field[idx] - dt * curl_e;
                }
            }
        }

        // Copy back B field for next step
        grid.b_field.copy_from_slice(&new_b);

        // Full-step E update: E^(n+1) = E^n + (dt/ε) * ∇×B^(n+1/2)
        for i in 1..grid.nx - 1 {
            for j in 1..grid.ny - 1 {
                for k in 1..grid.nz - 1 {
                    let idx = grid.index(i, j, k);

                    // Skip conductors
                    if grid.sigma[idx] > 1e6 {
                        continue;
                    }

                    let curl_b = self.curl_b(grid, i, j, k);
                    let epsilon = grid.epsilon[idx];
                    let mu = grid.mu[idx];

                    // E update with material properties
                    new_e[idx] = grid.e_field[idx] + (dt / (epsilon * mu)) * curl_b;
                }
            }
        }

        // Copy back E field
        grid.e_field.copy_from_slice(&new_e);
    }

    fn get_field_at(&self, grid: &VoxelGrid, pos: Vec3) -> EMField {
        grid.interpolate_field(pos)
    }

    fn name(&self) -> &str {
        "Maxwell's Equations"
    }

    fn initialize(&self, grid: &mut VoxelGrid) {
        // Initialize with a simple Gaussian pulse in the center
        let center = grid.dimensions() / 2.0;
        let width = grid.spacing * 5.0;
        self.initialize_gaussian_pulse(grid, center, width, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_maxwell_creation() {
        let maxwell = MaxwellTheory::new();
        assert_eq!(maxwell.name(), "Maxwell's Equations");
        assert!(maxwell.c > 0.0);
        assert!(maxwell.epsilon_0 > 0.0);
        assert!(maxwell.mu_0 > 0.0);
    }

    #[test]
    fn test_plane_wave_initialization() {
        let maxwell = MaxwellTheory::new();
        let mut grid = VoxelGrid::new(32, 32, 32, 0.01);

        maxwell.initialize_plane_wave(&mut grid, 1e9, Vec3::Z);

        // Check that fields are non-zero
        let mut has_nonzero_e = false;
        let mut has_nonzero_b = false;

        for i in 0..grid.size() {
            if grid.e_field[i].length() > 0.1 {
                has_nonzero_e = true;
            }
            if grid.b_field[i].length() > 0.0 {
                has_nonzero_b = true;
            }
        }

        assert!(has_nonzero_e, "E field should be initialized");
        assert!(has_nonzero_b, "B field should be initialized");
    }

    #[test]
    fn test_gaussian_pulse_initialization() {
        let maxwell = MaxwellTheory::new();
        let mut grid = VoxelGrid::new(32, 32, 32, 0.01);

        let center = Vec3::new(0.16, 0.16, 0.16); // Center of grid
        maxwell.initialize_gaussian_pulse(&mut grid, center, 0.05, 100.0);

        // Center should have maximum field
        let center_idx = grid.index(16, 16, 16);
        let center_field = grid.e_field[center_idx].length();

        // Corner should have much smaller field
        let corner_idx = grid.index(0, 0, 0);
        let corner_field = grid.e_field[corner_idx].length();

        assert!(center_field > 10.0 * corner_field);
    }

    #[test]
    fn test_static_field_stability() {
        let maxwell = MaxwellTheory::new();
        let mut grid = VoxelGrid::new(16, 16, 16, 0.001);

        // Start with zero fields
        grid.clear();

        let initial_energy = maxwell.total_energy(&grid);

        // Simulate - should remain zero
        let dt = 1e-12;
        for _ in 0..10 {
            maxwell.update_fields(&mut grid, dt);
        }

        let final_energy = maxwell.total_energy(&grid);

        assert_relative_eq!(initial_energy, 0.0, epsilon = 1e-10);
        assert_relative_eq!(final_energy, 0.0, epsilon = 1e-10);
    }

    #[test]
    #[should_panic(expected = "CFL condition violated")]
    fn test_cfl_violation() {
        let maxwell = MaxwellTheory::new();
        let mut grid = VoxelGrid::new(16, 16, 16, 0.01);

        // dt too large for stability
        let dt = 1e-9; // Way too large
        maxwell.update_fields(&mut grid, dt);
    }

    #[test]
    fn test_conductor_zeroes_fields() {
        let maxwell = MaxwellTheory::new();
        let mut grid = VoxelGrid::new(16, 16, 16, 0.01);

        // Initialize with some field
        maxwell.initialize_gaussian_pulse(&mut grid, Vec3::splat(0.08), 0.02, 10.0);

        // Make center region a conductor
        grid.set_conductor_region((6, 6, 6), (10, 10, 10));

        // Update
        let dt = 1e-12;
        for _ in 0..5 {
            maxwell.update_fields(&mut grid, dt);
        }

        // Check that conductor region has zero fields
        for i in 6..=10 {
            for j in 6..=10 {
                for k in 6..=10 {
                    let idx = grid.index(i, j, k);
                    assert_relative_eq!(grid.e_field[idx].length(), 0.0, epsilon = 1e-6);
                    assert_relative_eq!(grid.b_field[idx].length(), 0.0, epsilon = 1e-6);
                }
            }
        }
    }

    #[test]
    fn test_energy_calculation() {
        let maxwell = MaxwellTheory::new();
        let mut grid = VoxelGrid::new(16, 16, 16, 0.01);

        // Zero energy initially
        let energy = maxwell.total_energy(&grid);
        assert_relative_eq!(energy, 0.0, epsilon = 1e-10);

        // Add some field
        maxwell.initialize_gaussian_pulse(&mut grid, Vec3::splat(0.08), 0.02, 100.0);

        // Should have positive energy
        let energy = maxwell.total_energy(&grid);
        assert!(energy > 0.0);
    }
}
