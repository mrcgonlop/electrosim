//! Simulation infrastructure: grids, integrators, and boundary conditions

pub mod voxel_grid;
pub mod integrator;
pub mod boundary;
pub mod spacetime_cell;
pub mod spacetime_network;

pub use voxel_grid::VoxelGrid;
pub use integrator::{Integrator, LeapfrogIntegrator};
pub use boundary::{BoundaryCondition, PerfectConductor};
pub use spacetime_cell::SpacetimeCell;
pub use spacetime_network::{SpacetimeNetwork, NonLocalEdge, InteractionKind, NetworkStats};
