//! Interactive 2D field viewer with egui UI
//!
//! Displays electromagnetic field slices with real-time updates and interactive controls.

use crate::simulation::VoxelGrid;
use crate::visualization::{FieldComponent, FieldSlice, SlicePlane, slice2d::colormaps};

/// 2D field viewer with interactive controls
pub struct Viewer2D {
    /// Current slice plane
    pub plane: SlicePlane,
    /// Position along perpendicular axis (0.0 to 1.0)
    pub slice_position: f32,
    /// Field component to display
    pub component: FieldComponent,
    /// Whether to show conductor regions
    pub show_conductors: bool,
    /// Current colormap
    pub colormap: ColormapType,
    /// Whether simulation is paused
    pub paused: bool,
    /// Field value scale (auto or manual)
    pub value_scale: ValueScale,
}

/// Type of colormap to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColormapType {
    Viridis,
    Hot,
    Cool,
    Gray,
    RedBlue,
}

impl ColormapType {
    pub fn apply(&self, t: f32) -> [u8; 3] {
        match self {
            Self::Viridis => colormaps::viridis(t),
            Self::Hot => colormaps::hot(t),
            Self::Cool => colormaps::cool(t),
            Self::Gray => colormaps::gray(t),
            Self::RedBlue => colormaps::red_blue(t),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Viridis => "Viridis",
            Self::Hot => "Hot",
            Self::Cool => "Cool",
            Self::Gray => "Grayscale",
            Self::RedBlue => "Red-Blue",
        }
    }

    pub fn all() -> &'static [ColormapType] {
        &[
            ColormapType::Viridis,
            ColormapType::Hot,
            ColormapType::Cool,
            ColormapType::Gray,
            ColormapType::RedBlue,
        ]
    }
}

/// Field value scaling mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueScale {
    /// Automatically scale to min/max of current slice
    Auto,
    /// Use fixed min/max values
    Fixed { min: f32, max: f32 },
}

impl Default for Viewer2D {
    fn default() -> Self {
        Self::new()
    }
}

impl Viewer2D {
    /// Create a new 2D viewer with default settings
    pub fn new() -> Self {
        Self {
            plane: SlicePlane::XY,
            slice_position: 0.5,
            component: FieldComponent::EMagnitude,
            show_conductors: true,
            colormap: ColormapType::Viridis,
            paused: false,
            value_scale: ValueScale::Auto,
        }
    }

    /// Extract the current slice from the grid
    pub fn get_slice(&self, grid: &VoxelGrid) -> FieldSlice {
        FieldSlice::from_grid(grid, self.plane, self.slice_position, self.component)
    }

    /// Render the slice to RGBA8 image data
    pub fn render_slice(&self, grid: &VoxelGrid) -> Vec<u8> {
        let slice = self.get_slice(grid);
        let colormap = self.colormap;
        slice.to_rgba8(move |t| colormap.apply(t))
    }
}

/// Viewer application state
pub struct ViewerApp {
    /// Viewer configuration
    pub viewer: Viewer2D,
    /// Current grid being visualized
    pub grid: VoxelGrid,
    /// Texture for displaying the field
    texture: Option<wgpu::Texture>,
    /// Texture view
    texture_view: Option<wgpu::TextureView>,
    /// Current simulation time
    pub time: f32,
    /// Time step
    pub dt: f32,
    /// Frame counter
    pub frame: usize,
}

impl ViewerApp {
    /// Create a new viewer application
    pub fn new(grid: VoxelGrid) -> Self {
        Self {
            viewer: Viewer2D::new(),
            grid,
            texture: None,
            texture_view: None,
            time: 0.0,
            dt: 1e-12,
            frame: 0,
        }
    }

    /// Update the field texture from current grid state
    pub fn update_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let slice = self.viewer.get_slice(&self.grid);
        let rgba = self.viewer.render_slice(&self.grid);

        let size = wgpu::Extent3d {
            width: slice.width as u32,
            height: slice.height as u32,
            depth_or_array_layers: 1,
        };

        // Create or recreate texture if size changed
        if self.texture.is_none()
            || self.texture.as_ref().unwrap().width() != size.width
            || self.texture.as_ref().unwrap().height() != size.height
        {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Field Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            self.texture_view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));
            self.texture = Some(texture);
        }

        // Upload texture data
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: self.texture.as_ref().unwrap(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );
    }

    /// Render UI with egui
    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.heading("EM Field Viewer");

            ui.separator();

            ui.label("Slice Plane:");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.viewer.plane, SlicePlane::XY, "XY");
                ui.selectable_value(&mut self.viewer.plane, SlicePlane::XZ, "XZ");
                ui.selectable_value(&mut self.viewer.plane, SlicePlane::YZ, "YZ");
            });

            ui.add(egui::Slider::new(&mut self.viewer.slice_position, 0.0..=1.0).text("Position"));

            ui.separator();

            ui.label("Field Component:");
            egui::ComboBox::from_label("Component")
                .selected_text(self.viewer.component.name())
                .show_ui(ui, |ui| {
                    for component in [
                        FieldComponent::EMagnitude,
                        FieldComponent::BMagnitude,
                        FieldComponent::Ex,
                        FieldComponent::Ey,
                        FieldComponent::Ez,
                        FieldComponent::Bx,
                        FieldComponent::By,
                        FieldComponent::Bz,
                        FieldComponent::Energy,
                        FieldComponent::PoyntingMagnitude,
                    ] {
                        ui.selectable_value(&mut self.viewer.component, component, component.name());
                    }
                });

            ui.separator();

            ui.label("Colormap:");
            egui::ComboBox::from_label("Colormap")
                .selected_text(self.viewer.colormap.name())
                .show_ui(ui, |ui| {
                    for &colormap in ColormapType::all() {
                        ui.selectable_value(&mut self.viewer.colormap, colormap, colormap.name());
                    }
                });

            ui.separator();

            ui.checkbox(&mut self.viewer.show_conductors, "Show Conductors");

            ui.separator();

            ui.label("Simulation:");
            ui.horizontal(|ui| {
                if ui.button(if self.viewer.paused { "▶ Play" } else { "⏸ Pause" }).clicked() {
                    self.viewer.paused = !self.viewer.paused;
                }
                if ui.button("⏭ Step").clicked() {
                    // Single step will be handled by caller
                }
            });

            ui.add(egui::Slider::new(&mut self.dt, 1e-14..=1e-11).text("Time Step").logarithmic(true));

            ui.separator();

            ui.label(format!("Frame: {}", self.frame));
            ui.label(format!("Time: {:.2e} s", self.time));

            let slice = self.viewer.get_slice(&self.grid);
            ui.label(format!("Min: {:.2e}", slice.min_value));
            ui.label(format!("Max: {:.2e}", slice.max_value));

            ui.separator();

            if ui.button("Reset View").clicked() {
                self.viewer.slice_position = 0.5;
                self.viewer.plane = SlicePlane::XY;
            }
        });

        // Central panel would show the actual field visualization
        // This will be rendered via wgpu texture, egui just provides the UI
    }

    /// Get the texture view for rendering
    pub fn texture_view(&self) -> Option<&wgpu::TextureView> {
        self.texture_view.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewer_creation() {
        let viewer = Viewer2D::new();
        assert_eq!(viewer.plane, SlicePlane::XY);
        assert_eq!(viewer.slice_position, 0.5);
        assert_eq!(viewer.component, FieldComponent::EMagnitude);
    }

    #[test]
    fn test_viewer_slice_extraction() {
        let viewer = Viewer2D::new();
        let grid = VoxelGrid::new(16, 16, 16, 0.1);

        let slice = viewer.get_slice(&grid);
        assert_eq!(slice.width, 16);
        assert_eq!(slice.height, 16);
    }

    #[test]
    fn test_viewer_render() {
        let viewer = Viewer2D::new();
        let grid = VoxelGrid::new(8, 8, 8, 0.1);

        let rgba = viewer.render_slice(&grid);
        assert_eq!(rgba.len(), 8 * 8 * 4); // RGBA
    }

    #[test]
    fn test_colormap_names() {
        assert_eq!(ColormapType::Viridis.name(), "Viridis");
        assert_eq!(ColormapType::Hot.name(), "Hot");
    }

    #[test]
    fn test_app_creation() {
        let grid = VoxelGrid::new(16, 16, 16, 0.1);
        let app = ViewerApp::new(grid);

        assert_eq!(app.frame, 0);
        assert_eq!(app.time, 0.0);
    }
}
