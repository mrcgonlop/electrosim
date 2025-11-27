//! Physics module containing EM theory traits and implementations

pub mod em_theory;
pub mod field;
pub mod maxwell;
pub mod weber;
pub mod weber_particles;
pub mod wave2d;
pub mod fluid2d;
pub mod curved_spacetime;

pub use em_theory::{EMTheory, ScalarWaveTheory};
pub use field::{EMField, ScalarField};
pub use maxwell::MaxwellTheory;
pub use weber::WeberTheory;
pub use weber_particles::{WeberParticles, Particle};
pub use wave2d::Wave2D;
pub use fluid2d::Fluid2D;
pub use curved_spacetime::CurvedSpacetime2D;
