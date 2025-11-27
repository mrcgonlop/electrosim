//! Pre-configured experimental setups

pub mod faraday_cage;

pub use faraday_cage::FaradayCage;

use crate::simulation::VoxelGrid;

/// Trait for experimental setups
pub trait Experiment {
    /// Get the name of this experiment
    fn name(&self) -> &str;

    /// Set up the experiment in the given grid
    fn setup(&self, grid: &mut VoxelGrid);

    /// Check if the experiment passed its success criteria
    fn check_result(&self, grid: &VoxelGrid) -> ExperimentResult;
}

/// Result of an experiment
#[derive(Debug, Clone, PartialEq)]
pub enum ExperimentResult {
    /// Experiment passed all checks
    Pass,
    /// Experiment failed with reason
    Fail(String),
    /// Experiment is still running
    Running,
}

impl ExperimentResult {
    /// Check if the experiment passed
    pub fn is_pass(&self) -> bool {
        matches!(self, ExperimentResult::Pass)
    }

    /// Check if the experiment failed
    pub fn is_fail(&self) -> bool {
        matches!(self, ExperimentResult::Fail(_))
    }
}
