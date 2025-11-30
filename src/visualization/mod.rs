//! Visualization and UI components

pub mod renderer;
pub mod slice2d;
pub mod viewer;
pub mod dimensional_slice;
pub mod video_renderer;

pub use renderer::Renderer;
pub use slice2d::{FieldSlice, SlicePlane, FieldComponent};
pub use viewer::{Viewer2D, ViewerApp};
pub use dimensional_slice::{DimensionalSlice, MultiSliceView, DimensionMode};
pub use video_renderer::{VideoRenderer, VideoSequence, create_rotation_video, create_evolution_video};
