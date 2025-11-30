//! Dimensional Slice Visualization
//!
//! Renders 2D slices through 3D adaptive automata, showing local dimension
//! as color, height, or opacity.
//!
//! Visualization strategies:
//! 1. Color-coded dimension (blue=1D, green=2D, yellow=3D, red=4D+)
//! 2. Height map (dimension → z-height)
//! 3. Transparency (lower dimension → more transparent)
//! 4. Gradient visualization (show dimensional forces)

use crate::physics::adaptive_automata::{AdaptiveAutomata, Cell};
use image::{ImageBuffer, Rgb, Rgba};
use glam::Vec3;

/// Dimensional visualization mode
#[derive(Clone, Copy, Debug)]
pub enum DimensionMode {
    /// Color code by dimension
    ColorCoded,
    /// Show as height map
    HeightMap,
    /// Dimension as transparency
    Transparency,
    /// Show dimensional gradients (forces)
    Gradient,
}

/// 2D slice through 3D automata
pub struct DimensionalSlice {
    /// Which plane to slice (0=XY, 1=XZ, 2=YZ)
    pub plane: usize,
    /// Position along perpendicular axis
    pub slice_pos: usize,
    /// Visualization mode
    pub mode: DimensionMode,
    /// Color mapping parameters
    pub dim_min: f32,
    pub dim_max: f32,
}

impl DimensionalSlice {
    pub fn new(plane: usize, slice_pos: usize) -> Self {
        Self {
            plane,
            slice_pos,
            mode: DimensionMode::ColorCoded,
            dim_min: 0.0,
            dim_max: 4.0,
        }
    }

    /// Render slice as RGB image
    pub fn render(&self, automata: &AdaptiveAutomata, width: u32, height: u32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let mut img = ImageBuffer::new(width, height);

        let (nx, ny) = match self.plane {
            0 => (automata.nx, automata.ny),  // XY plane
            1 => (automata.nx, automata.nz),  // XZ plane
            _ => (automata.ny, automata.nz),  // YZ plane
        };

        for y in 0..height {
            for x in 0..width {
                // Map pixel to grid coordinates
                let i = (x as f32 / width as f32 * nx as f32) as usize;
                let j = (y as f32 / height as f32 * ny as f32) as usize;

                // Get cell index based on slice plane
                let cell_idx = match self.plane {
                    0 => automata.cell_index(i, j, self.slice_pos),  // XY
                    1 => automata.cell_index(i, self.slice_pos, j),  // XZ
                    _ => automata.cell_index(self.slice_pos, i, j),  // YZ
                };

                if cell_idx < automata.cells.len() {
                    let cell = &automata.cells[cell_idx];
                    let color = self.cell_color(cell, automata, cell_idx);
                    img.put_pixel(x, y, color);
                }
            }
        }

        img
    }

    /// Render slice with transparency (RGBA)
    pub fn render_rgba(&self, automata: &AdaptiveAutomata, width: u32, height: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut img = ImageBuffer::new(width, height);

        let (nx, ny) = match self.plane {
            0 => (automata.nx, automata.ny),
            1 => (automata.nx, automata.nz),
            _ => (automata.ny, automata.nz),
        };

        for y in 0..height {
            for x in 0..width {
                let i = (x as f32 / width as f32 * nx as f32) as usize;
                let j = (y as f32 / height as f32 * ny as f32) as usize;

                let cell_idx = match self.plane {
                    0 => automata.cell_index(i, j, self.slice_pos),
                    1 => automata.cell_index(i, self.slice_pos, j),
                    _ => automata.cell_index(self.slice_pos, i, j),
                };

                if cell_idx < automata.cells.len() {
                    let cell = &automata.cells[cell_idx];
                    let color = self.cell_color_rgba(cell, automata, cell_idx);
                    img.put_pixel(x, y, color);
                }
            }
        }

        img
    }

    /// Get RGB color for cell based on dimension
    fn cell_color(&self, cell: &Cell, automata: &AdaptiveAutomata, cell_idx: usize) -> Rgb<u8> {
        match self.mode {
            DimensionMode::ColorCoded => self.dimension_to_color(cell.dimension),
            DimensionMode::HeightMap => self.height_map_color(cell.dimension),
            DimensionMode::Gradient => self.gradient_color(automata, cell_idx),
            DimensionMode::Transparency => self.dimension_to_color(cell.dimension),
        }
    }

    /// Get RGBA color with transparency
    fn cell_color_rgba(&self, cell: &Cell, automata: &AdaptiveAutomata, cell_idx: usize) -> Rgba<u8> {
        let rgb = self.cell_color(cell, automata, cell_idx);

        // Alpha based on dimension (lower d → more transparent)
        let alpha = match self.mode {
            DimensionMode::Transparency => {
                let normalized = (cell.dimension - self.dim_min) / (self.dim_max - self.dim_min);
                (normalized.clamp(0.0, 1.0) * 255.0) as u8
            }
            _ => 255,
        };

        Rgba([rgb[0], rgb[1], rgb[2], alpha])
    }

    /// Map dimension to color (blue → green → yellow → red)
    fn dimension_to_color(&self, dimension: f32) -> Rgb<u8> {
        let normalized = (dimension - self.dim_min) / (self.dim_max - self.dim_min);
        let t = normalized.clamp(0.0, 1.0);

        // Color gradient:
        // d=0-1: Blue (1D strings/defects)
        // d=1-2: Cyan → Green (2D sheets)
        // d=2-3: Green → Yellow (approaching 3D)
        // d=3: Yellow (our 3D space)
        // d>3: Orange → Red (4D+ exotic regions)

        if t < 0.25 {
            // Blue → Cyan
            let s = t * 4.0;
            Rgb([0, (128.0 * s) as u8, 255])
        } else if t < 0.5 {
            // Cyan → Green
            let s = (t - 0.25) * 4.0;
            Rgb([0, 128 + (127.0 * s) as u8, (255.0 * (1.0 - s)) as u8])
        } else if t < 0.75 {
            // Green → Yellow
            let s = (t - 0.5) * 4.0;
            Rgb([(255.0 * s) as u8, 255, 0])
        } else {
            // Yellow → Red
            let s = (t - 0.75) * 4.0;
            Rgb([255, (255.0 * (1.0 - s)) as u8, 0])
        }
    }

    /// Height map coloring (grayscale based on dimension)
    fn height_map_color(&self, dimension: f32) -> Rgb<u8> {
        let normalized = (dimension - self.dim_min) / (self.dim_max - self.dim_min);
        let intensity = (normalized.clamp(0.0, 1.0) * 255.0) as u8;
        Rgb([intensity, intensity, intensity])
    }

    /// Color based on dimensional gradient magnitude
    fn gradient_color(&self, automata: &AdaptiveAutomata, cell_idx: usize) -> Rgb<u8> {
        let grad = automata.dimensional_gradient(cell_idx);
        let magnitude = grad.length();

        // Map gradient magnitude to hot colormap
        let t = (magnitude * 10.0).clamp(0.0, 1.0);  // Scale for visibility

        if t < 0.5 {
            // Black → Red
            let s = t * 2.0;
            Rgb([(255.0 * s) as u8, 0, 0])
        } else {
            // Red → Yellow
            let s = (t - 0.5) * 2.0;
            Rgb([255, (255.0 * s) as u8, 0])
        }
    }
}

/// Render multiple slices to show 3D structure
pub struct MultiSliceView {
    pub num_slices: usize,
    pub plane: usize,
    pub mode: DimensionMode,
}

impl MultiSliceView {
    pub fn new(plane: usize, num_slices: usize) -> Self {
        Self {
            num_slices,
            plane,
            mode: DimensionMode::ColorCoded,
        }
    }

    /// Render grid of slices
    pub fn render(&self, automata: &AdaptiveAutomata, slice_width: u32, slice_height: u32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let cols = (self.num_slices as f32).sqrt().ceil() as u32;
        let rows = (self.num_slices as f32 / cols as f32).ceil() as u32;

        let total_width = cols * slice_width;
        let total_height = rows * slice_height;

        let mut img = ImageBuffer::new(total_width, total_height);

        let depth = match self.plane {
            0 => automata.nz,  // XY slices through Z
            1 => automata.ny,  // XZ slices through Y
            _ => automata.nx,  // YZ slices through X
        };

        for slice_idx in 0..self.num_slices {
            let slice_pos = (slice_idx * depth) / self.num_slices;

            let mut slicer = DimensionalSlice::new(self.plane, slice_pos);
            slicer.mode = self.mode;
            slicer.dim_min = automata.min_dimension;
            slicer.dim_max = automata.max_dimension;

            let slice_img = slicer.render(automata, slice_width, slice_height);

            // Copy to grid position
            let grid_x = (slice_idx as u32 % cols) * slice_width;
            let grid_y = (slice_idx as u32 / cols) * slice_height;

            for y in 0..slice_height {
                for x in 0..slice_width {
                    if let Some(pixel) = slice_img.get_pixel_checked(x, y) {
                        let target_x = grid_x + x;
                        let target_y = grid_y + y;
                        if target_x < total_width && target_y < total_height {
                            img.put_pixel(target_x, target_y, *pixel);
                        }
                    }
                }
            }
        }

        img
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_creation() {
        let slice = DimensionalSlice::new(0, 5);
        assert_eq!(slice.plane, 0);
        assert_eq!(slice.slice_pos, 5);
    }

    #[test]
    fn test_color_mapping() {
        let slice = DimensionalSlice::new(0, 0);

        // Test color gradient
        let color_1d = slice.dimension_to_color(1.0);
        let color_2d = slice.dimension_to_color(2.0);
        let color_3d = slice.dimension_to_color(3.0);
        let color_4d = slice.dimension_to_color(4.0);

        // 1D should be bluish
        assert!(color_1d[2] > 200, "1D should be blue");

        // 3D should be yellowish
        assert!(color_3d[0] > 200 && color_3d[1] > 200, "3D should be yellow");

        // 4D should be reddish
        assert!(color_4d[0] > 200, "4D should be red");
    }
}
