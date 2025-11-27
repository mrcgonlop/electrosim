//! Time integration schemes for the simulation

use crate::physics::EMTheory;
use crate::simulation::VoxelGrid;

/// Trait for time integration schemes
pub trait Integrator {
    /// Advance the simulation by one time step
    fn step(&self, theory: &dyn EMTheory, grid: &mut VoxelGrid, dt: f32);

    /// Get the name of this integrator
    fn name(&self) -> &str;
}

/// Leapfrog (Verlet) integrator for EM fields
///
/// This integrator is symplectic and preserves energy well.
/// E and B fields are offset by half a time step.
pub struct LeapfrogIntegrator;

impl LeapfrogIntegrator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LeapfrogIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Integrator for LeapfrogIntegrator {
    fn step(&self, theory: &dyn EMTheory, grid: &mut VoxelGrid, dt: f32) {
        // The theory's update_fields method handles the leapfrog integration
        theory.update_fields(grid, dt);
    }

    fn name(&self) -> &str {
        "Leapfrog"
    }
}

/// Runge-Kutta 4th order integrator (for future use with particle-based theories)
pub struct RK4Integrator;

impl RK4Integrator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RK4Integrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Integrator for RK4Integrator {
    fn step(&self, theory: &dyn EMTheory, grid: &mut VoxelGrid, dt: f32) {
        // TODO: Implement RK4 for higher accuracy
        // For now, fall back to single step
        theory.update_fields(grid, dt);
    }

    fn name(&self) -> &str {
        "RK4 (Stub)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::MaxwellTheory;

    #[test]
    fn test_leapfrog_integrator() {
        let integrator = LeapfrogIntegrator::new();
        assert_eq!(integrator.name(), "Leapfrog");

        let theory = MaxwellTheory::new();
        let mut grid = VoxelGrid::new(16, 16, 16, 0.01);

        // Should not crash
        integrator.step(&theory, &mut grid, 1e-12);
    }

    #[test]
    fn test_rk4_integrator() {
        let integrator = RK4Integrator::new();
        assert_eq!(integrator.name(), "RK4 (Stub)");
    }
}
