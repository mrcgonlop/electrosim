//! Physics module containing EM theory traits and implementations

pub mod em_theory;
pub mod field;
// Obsolete modules (commented out - use em_emergence_tests instead)
// pub mod maxwell;
// pub mod weber;
// pub mod weber_particles;
// pub mod wave2d;
// pub mod fluid2d;
pub mod curved_spacetime;
pub mod hypergraph;
pub mod experimental_tests;
pub mod adaptive_automata;
pub mod graph_embedding;
pub mod simple_rules;
pub mod em_emergence_tests;
pub mod graph_optimization;
pub mod dimensional_dynamics;
pub mod quaternion_field;

pub use em_theory::{EMTheory, ScalarWaveTheory};
pub use field::{EMField, ScalarField};
// pub use maxwell::MaxwellTheory;
// pub use weber::WeberTheory;
// pub use weber_particles::{WeberParticles, Particle};
// pub use wave2d::Wave2D;
// pub use fluid2d::Fluid2D;
pub use curved_spacetime::CurvedSpacetime2D;
pub use hypergraph::{Hypergraph, HyperEdge, EdgeType};
pub use experimental_tests::{ExperimentalTestSuite, PhysicsTest, TestResult};
pub use adaptive_automata::{AdaptiveAutomata, Cell};
pub use graph_embedding::GraphEmbedding;
pub use simple_rules::MultiwayGraph;
