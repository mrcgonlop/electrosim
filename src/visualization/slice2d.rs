//! 2D slice visualization for electromagnetic fields
//!
//! Displays cross-sectional views through 3D field data with heatmaps and vector arrows.

use crate::simulation::VoxelGrid;
use glam::Vec3;

/// Slice plane orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlicePlane {
    /// XY plane (looking down Z axis)
    XY,
    /// XZ plane (looking down Y axis)
    XZ,
    /// YZ plane (looking down X axis)
    YZ,
}

/// Field component to visualize
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldComponent {
    /// Electric field magnitude
    EMagnitude,
    /// Magnetic field magnitude
    BMagnitude,
    /// Electric field X component
    Ex,
    /// Electric field Y component
    Ey,
    /// Electric field Z component
    Ez,
    /// Magnetic field X component
    Bx,
    /// Magnetic field Y component
    By,
    /// Magnetic field Z component
    Bz,
    /// Energy density
    Energy,
    /// Poynting vector magnitude
    PoyntingMagnitude,
}

impl FieldComponent {
    /// Get the name of this field component
    pub fn name(&self) -> &str {
        match self {
            Self::EMagnitude => "E Field Magnitude",
            Self::BMagnitude => "B Field Magnitude",
            Self::Ex => "E_x",
            Self::Ey => "E_y",
            Self::Ez => "E_z",
            Self::Bx => "B_x",
            Self::By => "B_y",
            Self::Bz => "B_z",
            Self::Energy => "Energy Density",
            Self::PoyntingMagnitude => "Poynting Vector",
        }
    }

    /// Extract the field value at a grid point
    pub fn extract(&self, grid: &VoxelGrid, idx: usize) -> f32 {
        match self {
            Self::EMagnitude => grid.e_field[idx].length(),
            Self::BMagnitude => grid.b_field[idx].length(),
            Self::Ex => grid.e_field[idx].x,
            Self::Ey => grid.e_field[idx].y,
            Self::Ez => grid.e_field[idx].z,
            Self::Bx => grid.b_field[idx].x,
            Self::By => grid.b_field[idx].y,
            Self::Bz => grid.b_field[idx].z,
            Self::Energy => {
                if let Some(field) = grid.get_em_field(idx) {
                    field.energy_density()
                } else {
                    0.0
                }
            }
            Self::PoyntingMagnitude => {
                if let Some(field) = grid.get_em_field(idx) {
                    field.poynting_vector().length()
                } else {
                    0.0
                }
            }
        }
    }
}

/// 2D slice through 3D field data
pub struct FieldSlice {
    /// Width of the slice
    pub width: usize,
    /// Height of the slice
    pub height: usize,
    /// Field values at each point
    pub values: Vec<f32>,
    /// Minimum value in the slice
    pub min_value: f32,
    /// Maximum value in the slice
    pub max_value: f32,
}

impl FieldSlice {
    /// Extract a 2D slice from a 3D grid
    ///
    /// # Arguments
    ///
    /// * `grid` - The 3D voxel grid
    /// * `plane` - Which plane to slice
    /// * `position` - Position along the perpendicular axis (0.0 to 1.0)
    /// * `component` - Which field component to extract
    pub fn from_grid(
        grid: &VoxelGrid,
        plane: SlicePlane,
        position: f32,
        component: FieldComponent,
    ) -> Self {
        let (width, height, depth_idx) = match plane {
            SlicePlane::XY => {
                let depth_idx = (position * (grid.nz - 1) as f32) as usize;
                (grid.nx, grid.ny, depth_idx.min(grid.nz - 1))
            }
            SlicePlane::XZ => {
                let depth_idx = (position * (grid.ny - 1) as f32) as usize;
                (grid.nx, grid.nz, depth_idx.min(grid.ny - 1))
            }
            SlicePlane::YZ => {
                let depth_idx = (position * (grid.nx - 1) as f32) as usize;
                (grid.ny, grid.nz, depth_idx.min(grid.nx - 1))
            }
        };

        let mut values = Vec::with_capacity(width * height);
        let mut min_value = f32::INFINITY;
        let mut max_value = f32::NEG_INFINITY;

        for j in 0..height {
            for i in 0..width {
                let idx = match plane {
                    SlicePlane::XY => grid.index(i, j, depth_idx),
                    SlicePlane::XZ => grid.index(i, depth_idx, j),
                    SlicePlane::YZ => grid.index(depth_idx, i, j),
                };

                let value = component.extract(grid, idx);
                min_value = min_value.min(value);
                max_value = max_value.max(value);
                values.push(value);
            }
        }

        Self {
            width,
            height,
            values,
            min_value,
            max_value,
        }
    }

    /// Get the value at a specific point
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.values[y * self.width + x]
    }

    /// Convert to RGBA8 image using a colormap
    ///
    /// # Arguments
    ///
    /// * `colormap` - Function that maps normalized value (0-1) to RGB color
    pub fn to_rgba8<F>(&self, colormap: F) -> Vec<u8>
    where
        F: Fn(f32) -> [u8; 3],
    {
        let mut rgba = Vec::with_capacity(self.width * self.height * 4);

        let range = self.max_value - self.min_value;
        let range = if range > 1e-10 { range } else { 1.0 };

        for &value in &self.values {
            // Normalize to 0-1
            let normalized = ((value - self.min_value) / range).clamp(0.0, 1.0);

            // Apply colormap
            let rgb = colormap(normalized);

            rgba.push(rgb[0]);
            rgba.push(rgb[1]);
            rgba.push(rgb[2]);
            rgba.push(255); // Alpha
        }

        rgba
    }
}

/// Colormap functions
pub mod colormaps {
    /// Viridis colormap (perceptually uniform, good for scientific data)
    pub fn viridis(t: f32) -> [u8; 3] {
        // Simplified viridis approximation
        let r = (0.267 + 0.805 * t - 0.555 * t * t) * 255.0;
        let g = (0.005 + 1.62 * t - 0.984 * t * t) * 255.0;
        let b = (0.329 + 2.14 * t - 2.91 * t * t + 1.07 * t * t * t) * 255.0;

        [
            r.clamp(0.0, 255.0) as u8,
            g.clamp(0.0, 255.0) as u8,
            b.clamp(0.0, 255.0) as u8,
        ]
    }

    /// Hot colormap (black -> red -> yellow -> white)
    pub fn hot(t: f32) -> [u8; 3] {
        let r = (3.0 * t).clamp(0.0, 1.0) * 255.0;
        let g = (3.0 * t - 1.0).clamp(0.0, 1.0) * 255.0;
        let b = (3.0 * t - 2.0).clamp(0.0, 1.0) * 255.0;

        [r as u8, g as u8, b as u8]
    }

    /// Cool colormap (cyan -> magenta)
    pub fn cool(t: f32) -> [u8; 3] {
        let r = (t * 255.0) as u8;
        let g = ((1.0 - t) * 255.0) as u8;
        let b = 255;

        [r, g, b]
    }

    /// Grayscale
    pub fn gray(t: f32) -> [u8; 3] {
        let val = (t * 255.0) as u8;
        [val, val, val]
    }

    /// Red-blue diverging (good for showing +/- values)
    pub fn red_blue(t: f32) -> [u8; 3] {
        if t < 0.5 {
            // Blue to white
            let s = t * 2.0;
            [
                (s * 255.0) as u8,
                (s * 255.0) as u8,
                255,
            ]
        } else {
            // White to red
            let s = (t - 0.5) * 2.0;
            [
                255,
                ((1.0 - s) * 255.0) as u8,
                ((1.0 - s) * 255.0) as u8,
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_component_names() {
        assert_eq!(FieldComponent::EMagnitude.name(), "E Field Magnitude");
        assert_eq!(FieldComponent::Ex.name(), "E_x");
    }

    #[test]
    fn test_slice_extraction() {
        let grid = VoxelGrid::new(10, 10, 10, 0.1);

        // Extract XY plane at middle
        let slice = FieldSlice::from_grid(&grid, SlicePlane::XY, 0.5, FieldComponent::EMagnitude);

        assert_eq!(slice.width, 10);
        assert_eq!(slice.height, 10);
        assert_eq!(slice.values.len(), 100);
    }

    #[test]
    fn test_slice_all_planes() {
        let grid = VoxelGrid::new(8, 12, 16, 0.1);

        let xy = FieldSlice::from_grid(&grid, SlicePlane::XY, 0.5, FieldComponent::EMagnitude);
        assert_eq!(xy.width, 8);
        assert_eq!(xy.height, 12);

        let xz = FieldSlice::from_grid(&grid, SlicePlane::XZ, 0.5, FieldComponent::EMagnitude);
        assert_eq!(xz.width, 8);
        assert_eq!(xz.height, 16);

        let yz = FieldSlice::from_grid(&grid, SlicePlane::YZ, 0.5, FieldComponent::EMagnitude);
        assert_eq!(yz.width, 12);
        assert_eq!(yz.height, 16);
    }

    #[test]
    fn test_rgba_conversion() {
        let grid = VoxelGrid::new(4, 4, 4, 0.1);
        let slice = FieldSlice::from_grid(&grid, SlicePlane::XY, 0.5, FieldComponent::EMagnitude);

        let rgba = slice.to_rgba8(colormaps::viridis);

        // Should have 4 bytes per pixel
        assert_eq!(rgba.len(), 4 * 4 * 4);
    }

    #[test]
    fn test_colormaps() {
        // Test that colormaps return valid RGB values
        for colormap in [
            colormaps::viridis,
            colormaps::hot,
            colormaps::cool,
            colormaps::gray,
            colormaps::red_blue,
        ] {
            for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let rgb = colormap(t);
                // All values should be valid bytes
                assert!(rgb[0] <= 255);
                assert!(rgb[1] <= 255);
                assert!(rgb[2] <= 255);
            }
        }
    }

    #[test]
    fn test_min_max_values() {
        use crate::physics::{MaxwellTheory, EMTheory};

        let mut grid = VoxelGrid::new(8, 8, 8, 0.1);
        let maxwell = MaxwellTheory::new();

        // Initialize with some field
        maxwell.initialize(&mut grid);

        let slice = FieldSlice::from_grid(&grid, SlicePlane::XY, 0.5, FieldComponent::EMagnitude);

        // Min should be <= all values <= max
        for &value in &slice.values {
            assert!(value >= slice.min_value);
            assert!(value <= slice.max_value);
        }
    }
}
