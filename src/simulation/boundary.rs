//! Boundary condition implementations

use crate::simulation::VoxelGrid;
use glam::Vec3;

/// Trait for boundary conditions
pub trait BoundaryCondition: Send + Sync {
    /// Apply this boundary condition to the grid
    fn apply(&self, grid: &mut VoxelGrid);

    /// Get the name of this boundary condition
    fn name(&self) -> &str;
}

/// Perfect Electric Conductor (PEC) boundary condition
///
/// Enforces E_tangential = 0 at boundaries (conducting walls)
pub struct PerfectConductor {
    /// Which boundaries to apply PEC to (left, right, bottom, top, front, back)
    pub boundaries: [bool; 6],
}

impl PerfectConductor {
    /// Create PEC boundary on all sides
    pub fn all() -> Self {
        Self {
            boundaries: [true; 6],
        }
    }

    /// Create PEC boundary with custom configuration
    ///
    /// # Arguments
    ///
    /// * `boundaries` - [left, right, bottom, top, front, back]
    pub fn custom(boundaries: [bool; 6]) -> Self {
        Self { boundaries }
    }

    /// No PEC boundaries (open space)
    pub fn none() -> Self {
        Self {
            boundaries: [false; 6],
        }
    }
}

impl Default for PerfectConductor {
    fn default() -> Self {
        Self::all()
    }
}

impl BoundaryCondition for PerfectConductor {
    fn apply(&self, grid: &mut VoxelGrid) {
        // Left boundary (x = 0)
        if self.boundaries[0] {
            for j in 0..grid.ny {
                for k in 0..grid.nz {
                    let idx = grid.index(0, j, k);
                    grid.e_field[idx] = Vec3::ZERO;
                }
            }
        }

        // Right boundary (x = nx-1)
        if self.boundaries[1] {
            for j in 0..grid.ny {
                for k in 0..grid.nz {
                    let idx = grid.index(grid.nx - 1, j, k);
                    grid.e_field[idx] = Vec3::ZERO;
                }
            }
        }

        // Bottom boundary (y = 0)
        if self.boundaries[2] {
            for i in 0..grid.nx {
                for k in 0..grid.nz {
                    let idx = grid.index(i, 0, k);
                    grid.e_field[idx] = Vec3::ZERO;
                }
            }
        }

        // Top boundary (y = ny-1)
        if self.boundaries[3] {
            for i in 0..grid.nx {
                for k in 0..grid.nz {
                    let idx = grid.index(i, grid.ny - 1, k);
                    grid.e_field[idx] = Vec3::ZERO;
                }
            }
        }

        // Front boundary (z = 0)
        if self.boundaries[4] {
            for i in 0..grid.nx {
                for j in 0..grid.ny {
                    let idx = grid.index(i, j, 0);
                    grid.e_field[idx] = Vec3::ZERO;
                }
            }
        }

        // Back boundary (z = nz-1)
        if self.boundaries[5] {
            for i in 0..grid.nx {
                for j in 0..grid.ny {
                    let idx = grid.index(i, j, grid.nz - 1);
                    grid.e_field[idx] = Vec3::ZERO;
                }
            }
        }
    }

    fn name(&self) -> &str {
        "Perfect Electric Conductor"
    }
}

/// Periodic boundary conditions
///
/// Fields wrap around at boundaries (useful for infinite space simulations)
pub struct PeriodicBoundary;

impl PeriodicBoundary {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PeriodicBoundary {
    fn default() -> Self {
        Self::new()
    }
}

impl BoundaryCondition for PeriodicBoundary {
    fn apply(&self, grid: &mut VoxelGrid) {
        // Copy x boundaries
        for j in 0..grid.ny {
            for k in 0..grid.nz {
                let left = grid.index(0, j, k);
                let right = grid.index(grid.nx - 1, j, k);

                // Wrap around
                grid.e_field[left] = grid.e_field[right];
                grid.b_field[left] = grid.b_field[right];
            }
        }

        // Copy y boundaries
        for i in 0..grid.nx {
            for k in 0..grid.nz {
                let bottom = grid.index(i, 0, k);
                let top = grid.index(i, grid.ny - 1, k);

                grid.e_field[bottom] = grid.e_field[top];
                grid.b_field[bottom] = grid.b_field[top];
            }
        }

        // Copy z boundaries
        for i in 0..grid.nx {
            for j in 0..grid.ny {
                let front = grid.index(i, j, 0);
                let back = grid.index(i, j, grid.nz - 1);

                grid.e_field[front] = grid.e_field[back];
                grid.b_field[front] = grid.b_field[back];
            }
        }
    }

    fn name(&self) -> &str {
        "Periodic"
    }
}

/// Absorbing boundary condition (Mur's ABC, first order)
///
/// Attempts to absorb outgoing waves without reflection
pub struct AbsorbingBoundary {
    /// Speed of light
    pub c: f32,
}

impl AbsorbingBoundary {
    pub fn new(c: f32) -> Self {
        Self { c }
    }
}

impl Default for AbsorbingBoundary {
    fn default() -> Self {
        Self::new(2.998e8)
    }
}

impl BoundaryCondition for AbsorbingBoundary {
    fn apply(&self, _grid: &mut VoxelGrid) {
        // TODO: Implement Mur's ABC
        // For now, this is a stub
        log::warn!("Absorbing boundary not yet implemented");
    }

    fn name(&self) -> &str {
        "Absorbing (Mur's ABC - Stub)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_pec_all() {
        let pec = PerfectConductor::all();
        assert_eq!(pec.name(), "Perfect Electric Conductor");
        assert!(pec.boundaries.iter().all(|&b| b));
    }

    #[test]
    fn test_pec_none() {
        let pec = PerfectConductor::none();
        assert!(pec.boundaries.iter().all(|&b| !b));
    }

    #[test]
    fn test_pec_application() {
        let mut grid = VoxelGrid::new(10, 10, 10, 0.1);

        // Set some fields
        for i in 0..grid.nx {
            for j in 0..grid.ny {
                for k in 0..grid.nz {
                    let idx = grid.index(i, j, k);
                    grid.e_field[idx] = Vec3::ONE;
                }
            }
        }

        // Apply PEC on all boundaries
        let pec = PerfectConductor::all();
        pec.apply(&mut grid);

        // Check that boundary fields are zero
        for j in 0..grid.ny {
            for k in 0..grid.nz {
                let left = grid.index(0, j, k);
                let right = grid.index(grid.nx - 1, j, k);
                assert_eq!(grid.e_field[left], Vec3::ZERO);
                assert_eq!(grid.e_field[right], Vec3::ZERO);
            }
        }

        // Interior should still have fields
        let interior = grid.index(5, 5, 5);
        assert_eq!(grid.e_field[interior], Vec3::ONE);
    }

    #[test]
    fn test_periodic_boundary() {
        let mut grid = VoxelGrid::new(10, 10, 10, 0.1);

        // Set field at right boundary
        let right_val = Vec3::new(1.0, 2.0, 3.0);
        for j in 0..grid.ny {
            for k in 0..grid.nz {
                let idx = grid.index(grid.nx - 1, j, k);
                grid.e_field[idx] = right_val;
            }
        }

        // Apply periodic boundary
        let periodic = PeriodicBoundary::new();
        periodic.apply(&mut grid);

        // Left boundary should now match right boundary
        for j in 0..grid.ny {
            for k in 0..grid.nz {
                let left = grid.index(0, j, k);
                assert_eq!(grid.e_field[left], right_val);
            }
        }
    }

    #[test]
    fn test_absorbing_boundary() {
        let abc = AbsorbingBoundary::new(3e8);
        assert_eq!(abc.name(), "Absorbing (Mur's ABC - Stub)");

        let mut grid = VoxelGrid::new(10, 10, 10, 0.1);
        // Should not crash (even though not implemented)
        abc.apply(&mut grid);
    }
}
