//! Visualization and UI components

pub mod renderer;
pub mod slice2d;
pub mod viewer;

pub use renderer::Renderer;
pub use slice2d::{FieldSlice, SlicePlane, FieldComponent};
pub use viewer::{Viewer2D, ViewerApp};
