//! 3D voxel grid for storing electromagnetic field data

use crate::physics::EMField;
use glam::Vec3;

/// A 3D voxel grid for storing electromagnetic field data.
///
/// The grid uses a staggered Yee lattice for FDTD simulations:
/// - E fields are stored at cell edges
/// - B fields are stored at cell faces
///
/// # Example
///
/// ```
/// use em_physics_sandbox::simulation::VoxelGrid;
///
/// let grid = VoxelGrid::new(64, 64, 64, 0.01);
/// assert_eq!(grid.nx, 64);
/// ```
#[derive(Debug, Clone)]
pub struct VoxelGrid {
    /// Number of voxels in x direction
    pub nx: usize,
    /// Number of voxels in y direction
    pub ny: usize,
    /// Number of voxels in z direction
    pub nz: usize,
    /// Physical spacing between voxels (meters)
    pub spacing: f32,

    /// Electric field components (staggered grid)
    pub e_field: Vec<Vec3>,
    /// Magnetic field components (staggered grid)
    pub b_field: Vec<Vec3>,

    /// Material properties per voxel
    pub epsilon: Vec<f32>,  // Permittivity
    pub mu: Vec<f32>,       // Permeability
    pub sigma: Vec<f32>,    // Conductivity
}

impl VoxelGrid {
    /// Create a new voxel grid with the specified dimensions and spacing.
    ///
    /// # Arguments
    ///
    /// * `nx`, `ny`, `nz` - Grid dimensions in voxels
    /// * `spacing` - Physical distance between voxel centers (meters)
    ///
    /// # Example
    ///
    /// ```
    /// use em_physics_sandbox::simulation::VoxelGrid;
    ///
    /// // Create a 32³ grid with 1cm spacing
    /// let grid = VoxelGrid::new(32, 32, 32, 0.01);
    /// ```
    pub fn new(nx: usize, ny: usize, nz: usize, spacing: f32) -> Self {
        let size = nx * ny * nz;

        Self {
            nx,
            ny,
            nz,
            spacing,
            e_field: vec![Vec3::ZERO; size],
            b_field: vec![Vec3::ZERO; size],
            epsilon: vec![8.854e-12; size],  // Vacuum permittivity
            mu: vec![1.257e-6; size],        // Vacuum permeability
            sigma: vec![0.0; size],          // No conductivity by default
        }
    }

    /// Get the linear index for a 3D coordinate.
    ///
    /// # Arguments
    ///
    /// * `i`, `j`, `k` - 3D voxel coordinates
    ///
    /// # Returns
    ///
    /// Linear index into the field arrays
    #[inline]
    pub fn index(&self, i: usize, j: usize, k: usize) -> usize {
        debug_assert!(i < self.nx, "i index out of bounds");
        debug_assert!(j < self.ny, "j index out of bounds");
        debug_assert!(k < self.nz, "k index out of bounds");
        i + self.nx * (j + self.ny * k)
    }

    /// Get 3D coordinates from a linear index.
    ///
    /// # Arguments
    ///
    /// * `idx` - Linear index
    ///
    /// # Returns
    ///
    /// Tuple of (i, j, k) coordinates
    #[inline]
    pub fn coords(&self, idx: usize) -> (usize, usize, usize) {
        let k = idx / (self.nx * self.ny);
        let remainder = idx % (self.nx * self.ny);
        let j = remainder / self.nx;
        let i = remainder % self.nx;
        (i, j, k)
    }

    /// Get the physical position of a voxel center.
    ///
    /// # Arguments
    ///
    /// * `i`, `j`, `k` - Voxel coordinates
    ///
    /// # Returns
    ///
    /// Position in meters
    pub fn position(&self, i: usize, j: usize, k: usize) -> Vec3 {
        Vec3::new(
            i as f32 * self.spacing,
            j as f32 * self.spacing,
            k as f32 * self.spacing,
        )
    }

    /// Get the EM field at a specific grid point.
    pub fn get_em_field(&self, idx: usize) -> Option<EMField> {
        if idx < self.e_field.len() {
            Some(EMField::new(self.e_field[idx], self.b_field[idx]))
        } else {
            None
        }
    }

    /// Set the electric field at a grid point.
    pub fn set_e_field(&mut self, idx: usize, e: Vec3) {
        if idx < self.e_field.len() {
            self.e_field[idx] = e;
        }
    }

    /// Set the magnetic field at a grid point.
    pub fn set_b_field(&mut self, idx: usize, b: Vec3) {
        if idx < self.b_field.len() {
            self.b_field[idx] = b;
        }
    }

    /// Get the EM field at a physical position using trilinear interpolation.
    ///
    /// # Arguments
    ///
    /// * `pos` - Position in space (meters)
    ///
    /// # Returns
    ///
    /// Interpolated EM field
    pub fn interpolate_field(&self, pos: Vec3) -> EMField {
        // Convert position to grid coordinates
        let grid_pos = pos / self.spacing;

        // Get the integer grid cell
        let i = grid_pos.x.floor().max(0.0).min((self.nx - 2) as f32) as usize;
        let j = grid_pos.y.floor().max(0.0).min((self.ny - 2) as f32) as usize;
        let k = grid_pos.z.floor().max(0.0).min((self.nz - 2) as f32) as usize;

        // Get fractional part for interpolation
        let fx = grid_pos.x - i as f32;
        let fy = grid_pos.y - j as f32;
        let fz = grid_pos.z - k as f32;

        // Trilinear interpolation weights
        let w000 = (1.0 - fx) * (1.0 - fy) * (1.0 - fz);
        let w100 = fx * (1.0 - fy) * (1.0 - fz);
        let w010 = (1.0 - fx) * fy * (1.0 - fz);
        let w110 = fx * fy * (1.0 - fz);
        let w001 = (1.0 - fx) * (1.0 - fy) * fz;
        let w101 = fx * (1.0 - fy) * fz;
        let w011 = (1.0 - fx) * fy * fz;
        let w111 = fx * fy * fz;

        // Sample the 8 corners
        let mut e = Vec3::ZERO;
        let mut b = Vec3::ZERO;

        for (di, dj, dk, weight) in [
            (0, 0, 0, w000), (1, 0, 0, w100), (0, 1, 0, w010), (1, 1, 0, w110),
            (0, 0, 1, w001), (1, 0, 1, w101), (0, 1, 1, w011), (1, 1, 1, w111),
        ] {
            let idx = self.index(i + di, j + dj, k + dk);
            e += self.e_field[idx] * weight;
            b += self.b_field[idx] * weight;
        }

        EMField::new(e, b)
    }

    /// Set a region to be a perfect conductor (zero fields inside).
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum corner of the region (grid coordinates)
    /// * `max` - Maximum corner of the region (grid coordinates)
    pub fn set_conductor_region(&mut self, min: (usize, usize, usize), max: (usize, usize, usize)) {
        for i in min.0..=max.0.min(self.nx - 1) {
            for j in min.1..=max.1.min(self.ny - 1) {
                for k in min.2..=max.2.min(self.nz - 1) {
                    let idx = self.index(i, j, k);
                    self.sigma[idx] = 1e10; // Very high conductivity
                }
            }
        }
    }

    /// Clear all fields to zero.
    pub fn clear(&mut self) {
        for i in 0..self.e_field.len() {
            self.e_field[i] = Vec3::ZERO;
            self.b_field[i] = Vec3::ZERO;
        }
    }

    /// Get total number of voxels.
    pub fn size(&self) -> usize {
        self.nx * self.ny * self.nz
    }

    /// Get the physical dimensions of the grid (meters).
    pub fn dimensions(&self) -> Vec3 {
        Vec3::new(
            self.nx as f32 * self.spacing,
            self.ny as f32 * self.spacing,
            self.nz as f32 * self.spacing,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_voxel_grid_creation() {
        let grid = VoxelGrid::new(10, 20, 30, 0.1);
        assert_eq!(grid.nx, 10);
        assert_eq!(grid.ny, 20);
        assert_eq!(grid.nz, 30);
        assert_eq!(grid.spacing, 0.1);
        assert_eq!(grid.size(), 10 * 20 * 30);
    }

    #[test]
    fn test_voxel_grid_indexing() {
        let grid = VoxelGrid::new(10, 10, 10, 0.1);

        // Test index -> coords -> index round trip
        for i in 0..10 {
            for j in 0..10 {
                for k in 0..10 {
                    let idx = grid.index(i, j, k);
                    let (i2, j2, k2) = grid.coords(idx);
                    assert_eq!((i, j, k), (i2, j2, k2));
                }
            }
        }
    }

    #[test]
    fn test_position_calculation() {
        let grid = VoxelGrid::new(10, 10, 10, 0.5);
        let pos = grid.position(2, 3, 4);

        assert_relative_eq!(pos.x, 1.0, epsilon = 1e-6);
        assert_relative_eq!(pos.y, 1.5, epsilon = 1e-6);
        assert_relative_eq!(pos.z, 2.0, epsilon = 1e-6);
    }

    #[test]
    fn test_field_get_set() {
        let mut grid = VoxelGrid::new(10, 10, 10, 0.1);

        let test_e = Vec3::new(1.0, 2.0, 3.0);
        let test_b = Vec3::new(4.0, 5.0, 6.0);

        let idx = grid.index(5, 5, 5);
        grid.set_e_field(idx, test_e);
        grid.set_b_field(idx, test_b);

        let field = grid.get_em_field(idx).unwrap();
        assert_eq!(field.electric, test_e);
        assert_eq!(field.magnetic, test_b);
    }

    #[test]
    fn test_clear() {
        let mut grid = VoxelGrid::new(5, 5, 5, 0.1);

        // Set some fields
        for i in 0..5 {
            let idx = grid.index(i, i, i);
            grid.set_e_field(idx, Vec3::ONE);
            grid.set_b_field(idx, Vec3::ONE);
        }

        // Clear
        grid.clear();

        // Verify all are zero
        for i in 0..grid.size() {
            assert_eq!(grid.e_field[i], Vec3::ZERO);
            assert_eq!(grid.b_field[i], Vec3::ZERO);
        }
    }

    #[test]
    fn test_interpolation_at_grid_point() {
        let mut grid = VoxelGrid::new(10, 10, 10, 1.0);

        let test_field = Vec3::new(1.0, 2.0, 3.0);
        let idx = grid.index(5, 5, 5);
        grid.set_e_field(idx, test_field);

        // Interpolate at exact grid point
        let pos = grid.position(5, 5, 5);
        let interpolated = grid.interpolate_field(pos);

        // Should be close to the original field
        assert!((interpolated.electric - test_field).length() < 0.5);
    }

    #[test]
    fn test_conductor_region() {
        let mut grid = VoxelGrid::new(10, 10, 10, 0.1);

        grid.set_conductor_region((2, 2, 2), (5, 5, 5));

        // Check that conductivity is high in the region
        for i in 2..=5 {
            for j in 2..=5 {
                for k in 2..=5 {
                    let idx = grid.index(i, j, k);
                    assert!(grid.sigma[idx] > 1e9);
                }
            }
        }

        // Check that conductivity is zero outside
        let idx = grid.index(0, 0, 0);
        assert_eq!(grid.sigma[idx], 0.0);
    }
}
