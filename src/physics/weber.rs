//! Weber electrodynamics implementation (stub for future development)
//!
//! Weber's electrodynamics is an action-at-a-distance theory that differs
//! from Maxwell's field theory. This is a placeholder for future implementation.

use crate::physics::{EMField, EMTheory};
use crate::simulation::VoxelGrid;
use glam::Vec3;

/// Weber electrodynamics theory (stub).
///
/// TODO: Implement Weber's force law:
/// F = qq'/r² [1 - (dr/dt)²/2c² + r(d²r/dt²)/c²]
///
/// This requires tracking particle positions and velocities rather than fields.
pub struct WeberTheory {
    /// Speed of light
    pub c: f32,
}

impl WeberTheory {
    /// Create a new Weber theory (not yet implemented)
    pub fn new() -> Self {
        Self { c: 2.998e8 }
    }
}

impl Default for WeberTheory {
    fn default() -> Self {
        Self::new()
    }
}

impl EMTheory for WeberTheory {
    fn update_fields(&self, _grid: &mut VoxelGrid, _dt: f32) {
        // TODO: Implement Weber's equations
        // For now, this is a no-op
        log::warn!("Weber theory not yet implemented");
    }

    fn get_field_at(&self, _grid: &VoxelGrid, _pos: Vec3) -> EMField {
        // TODO: Calculate field from particle positions
        EMField::zero()
    }

    fn name(&self) -> &str {
        "Weber Electrodynamics (Stub)"
    }

    fn initialize(&self, _grid: &mut VoxelGrid) {
        log::info!("Weber theory initialization not yet implemented");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weber_creation() {
        let weber = WeberTheory::new();
        assert_eq!(weber.name(), "Weber Electrodynamics (Stub)");
    }

    #[test]
    fn test_weber_stub() {
        let weber = WeberTheory::new();
        let mut grid = VoxelGrid::new(10, 10, 10, 0.1);

        // Should not crash, but won't do anything
        weber.update_fields(&mut grid, 0.01);

        let field = weber.get_field_at(&grid, Vec3::ZERO);
        assert_eq!(field, EMField::zero());
    }
}
