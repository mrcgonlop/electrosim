//! Video Renderer for 3D Animated Visualizations
//!
//! Creates video sequences showing:
//! - Graph evolution over time (emergent causality)
//! - 3D rotation (see structure from all angles)
//! - Dimensional changes (color-coded)
//! - Multiway branching (quantum mechanics!)

use crate::physics::{Hypergraph, MultiwayGraph, GraphEmbedding};
use crate::physics::adaptive_automata::AdaptiveAutomata;
use glam::{Vec3, Mat4};
use image::{ImageBuffer, Rgb};
use std::f32::consts::PI;

/// Video frame renderer
pub struct VideoRenderer {
    pub width: u32,
    pub height: u32,
    pub camera_distance: f32,
    pub fov: f32,
}

impl VideoRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            camera_distance: 50.0,
            fov: 60.0,
        }
    }

    /// Render graph in 3D with rotation
    pub fn render_graph_3d(
        &self,
        embedding: &GraphEmbedding,
        rotation_angle: f32,
        tilt_angle: f32,
    ) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let mut img = ImageBuffer::new(self.width, self.height);

        // Fill background (black)
        for pixel in img.pixels_mut() {
            *pixel = Rgb([0, 0, 0]);
        }

        // Camera transform
        let camera_pos = Vec3::new(
            self.camera_distance * rotation_angle.cos() * tilt_angle.cos(),
            self.camera_distance * tilt_angle.sin(),
            self.camera_distance * rotation_angle.sin() * tilt_angle.cos(),
        );

        let view_matrix = Mat4::look_at_rh(
            camera_pos,
            Vec3::ZERO,
            Vec3::Y,
        );

        let aspect = self.width as f32 / self.height as f32;
        let proj_matrix = Mat4::perspective_rh(
            self.fov.to_radians(),
            aspect,
            0.1,
            1000.0,
        );

        let vp_matrix = proj_matrix * view_matrix;

        // Find center and scale
        let positions: Vec<_> = embedding.positions.values().copied().collect();
        if positions.is_empty() {
            return img;
        }

        let center = positions.iter().sum::<Vec3>() / positions.len() as f32;
        let max_dist = positions.iter()
            .map(|p| (*p - center).length())
            .fold(0.0f32, f32::max);

        let scale = if max_dist > 0.0 { 20.0 / max_dist } else { 1.0 };

        // Draw edges first (so nodes appear on top)
        // This requires edge information which we'll approximate from neighbors
        // (Simplified: just draw all node-node connections)

        // Draw nodes
        for (&node_id, &pos) in &embedding.positions {
            let world_pos = (pos - center) * scale;
            let clip_pos = vp_matrix.project_point3(world_pos);

            // Check if in front of camera
            if clip_pos.z > 0.0 && clip_pos.z < 1.0 {
                // Convert to screen space
                let screen_x = ((clip_pos.x + 1.0) * 0.5 * self.width as f32) as i32;
                let screen_y = ((1.0 - clip_pos.y) * 0.5 * self.height as f32) as i32;

                // Get dimension for coloring
                let dim = embedding.dimensions.get(&node_id).copied().unwrap_or(3.0);
                let color = self.dimension_to_color(dim);

                // Draw node as small circle
                let radius = 3;
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        if dx*dx + dy*dy <= radius*radius {
                            let px = screen_x + dx;
                            let py = screen_y + dy;

                            if px >= 0 && px < self.width as i32 &&
                               py >= 0 && py < self.height as i32 {
                                img.put_pixel(px as u32, py as u32, color);
                            }
                        }
                    }
                }
            }
        }

        img
    }

    /// Render adaptive automata in 3D
    pub fn render_automata_3d(
        &self,
        automata: &AdaptiveAutomata,
        rotation_angle: f32,
        tilt_angle: f32,
        slice_z: usize,
    ) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let mut img = ImageBuffer::new(self.width, self.height);

        // Fill background
        for pixel in img.pixels_mut() {
            *pixel = Rgb([10, 10, 20]);
        }

        // Camera setup
        let camera_pos = Vec3::new(
            self.camera_distance * rotation_angle.cos() * tilt_angle.cos(),
            self.camera_distance * tilt_angle.sin(),
            self.camera_distance * rotation_angle.sin() * tilt_angle.cos(),
        );

        let center = Vec3::new(
            automata.nx as f32 * automata.spacing * 0.5,
            automata.ny as f32 * automata.spacing * 0.5,
            automata.nz as f32 * automata.spacing * 0.5,
        );

        let view_matrix = Mat4::look_at_rh(camera_pos, center, Vec3::Y);

        let aspect = self.width as f32 / self.height as f32;
        let proj_matrix = Mat4::perspective_rh(
            self.fov.to_radians(),
            aspect,
            0.1,
            1000.0,
        );

        let vp_matrix = proj_matrix * view_matrix;

        // Draw cells in the slice
        for k in 0..automata.nz {
            // Only draw near the slice plane
            if (k as i32 - slice_z as i32).abs() > 2 {
                continue;
            }

            for j in 0..automata.ny {
                for i in 0..automata.nx {
                    let idx = automata.cell_index(i, j, k);
                    if idx >= automata.cells.len() {
                        continue;
                    }

                    let cell = &automata.cells[idx];
                    let world_pos = cell.position;

                    let clip_pos = vp_matrix.project_point3(world_pos);

                    if clip_pos.z > 0.0 && clip_pos.z < 1.0 {
                        let screen_x = ((clip_pos.x + 1.0) * 0.5 * self.width as f32) as i32;
                        let screen_y = ((1.0 - clip_pos.y) * 0.5 * self.height as f32) as i32;

                        let color = self.dimension_to_color(cell.dimension);

                        // Draw cell
                        let size = if k == slice_z { 4 } else { 2 };
                        for dy in -size..=size {
                            for dx in -size..=size {
                                let px = screen_x + dx;
                                let py = screen_y + dy;

                                if px >= 0 && px < self.width as i32 &&
                                   py >= 0 && py < self.height as i32 {
                                    // Blend with existing pixel for depth effect
                                    let existing = img.get_pixel(px as u32, py as u32);
                                    let alpha = if k == slice_z { 1.0 } else { 0.3 };

                                    let blended = Rgb([
                                        (existing[0] as f32 * (1.0 - alpha) + color[0] as f32 * alpha) as u8,
                                        (existing[1] as f32 * (1.0 - alpha) + color[1] as f32 * alpha) as u8,
                                        (existing[2] as f32 * (1.0 - alpha) + color[2] as f32 * alpha) as u8,
                                    ]);

                                    img.put_pixel(px as u32, py as u32, blended);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Add text overlay showing rotation angle
        self.draw_text(&mut img, 10, 10, &format!("Rotation: {:.0}°", rotation_angle.to_degrees()));
        self.draw_text(&mut img, 10, 30, &format!("Avg Dim: {:.2}", automata.avg_dimension));
        self.draw_text(&mut img, 10, 50, &format!("Slice Z: {}/{}", slice_z, automata.nz));

        img
    }

    /// Map dimension to color
    fn dimension_to_color(&self, dimension: f32) -> Rgb<u8> {
        let t = (dimension / 4.0).clamp(0.0, 1.0);

        if t < 0.25 {
            // Blue → Cyan (d=0 to d=1)
            let s = t * 4.0;
            Rgb([0, (128.0 * s) as u8, 255])
        } else if t < 0.5 {
            // Cyan → Green (d=1 to d=2)
            let s = (t - 0.25) * 4.0;
            Rgb([0, 128 + (127.0 * s) as u8, (255.0 * (1.0 - s)) as u8])
        } else if t < 0.75 {
            // Green → Yellow (d=2 to d=3)
            let s = (t - 0.5) * 4.0;
            Rgb([(255.0 * s) as u8, 255, 0])
        } else {
            // Yellow → Red (d=3 to d=4+)
            let s = (t - 0.75) * 4.0;
            Rgb([255, (255.0 * (1.0 - s)) as u8, 0])
        }
    }

    /// Simple text rendering (bitmap-style)
    fn draw_text(&self, img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, x: u32, y: u32, text: &str) {
        // Very simple: just draw white pixels for now
        // (Full font rendering would require a font library)
        let color = Rgb([255, 255, 255]);

        for (i, _c) in text.chars().enumerate() {
            let px = x + (i as u32 * 8);
            if px < self.width && y < self.height {
                // Draw a simple rectangle for each character
                for dy in 0..10 {
                    for dx in 0..6 {
                        if px + dx < self.width && y + dy < self.height {
                            img.put_pixel(px + dx, y + dy, color);
                        }
                    }
                }
            }
        }
    }
}

/// Video sequence generator
pub struct VideoSequence {
    pub frames: Vec<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    pub fps: u32,
}

impl VideoSequence {
    pub fn new(fps: u32) -> Self {
        Self {
            frames: Vec::new(),
            fps,
        }
    }

    /// Add a frame
    pub fn add_frame(&mut self, frame: ImageBuffer<Rgb<u8>, Vec<u8>>) {
        self.frames.push(frame);
    }

    /// Save as individual PNG frames (for ffmpeg encoding)
    pub fn save_frames(&self, output_dir: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(output_dir)?;

        for (i, frame) in self.frames.iter().enumerate() {
            let filename = format!("{}/frame_{:05}.png", output_dir, i);
            frame.save(&filename).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            })?;
        }

        println!("Saved {} frames to {}/", self.frames.len(), output_dir);
        println!("\nTo create video, run:");
        println!("  ffmpeg -r {} -i {}frame_%05d.png -c:v libx264 -pix_fmt yuv420p output.mp4",
                 self.fps, output_dir);

        Ok(())
    }
}

/// Generate rotating view video
pub fn create_rotation_video(
    automata: &AdaptiveAutomata,
    duration_seconds: f32,
    fps: u32,
    width: u32,
    height: u32,
) -> VideoSequence {
    let mut sequence = VideoSequence::new(fps);
    let renderer = VideoRenderer::new(width, height);

    let total_frames = (duration_seconds * fps as f32) as usize;
    let rotation_cycles = 2.0; // Complete rotations

    for frame_idx in 0..total_frames {
        let t = frame_idx as f32 / total_frames as f32;

        // Rotation angle (0 to 2π * cycles)
        let rotation = t * 2.0 * PI * rotation_cycles;

        // Tilt angle (oscillate between -30° and +30°)
        let tilt = (t * 4.0 * PI).sin() * 0.3;

        // Varying slice position
        let slice_z = ((t * 3.0).sin() * 0.5 + 0.5) * (automata.nz - 1) as f32;

        let frame = renderer.render_automata_3d(
            automata,
            rotation,
            tilt,
            slice_z as usize,
        );

        sequence.add_frame(frame);

        if frame_idx % (fps as usize) == 0 {
            println!("Generated frame {}/{}", frame_idx, total_frames);
        }
    }

    sequence
}

/// Generate evolution video (multiway graph)
pub fn create_evolution_video(
    multiway: &MultiwayGraph,
    frame_skip: usize,
    width: u32,
    height: u32,
) -> VideoSequence {
    let mut sequence = VideoSequence::new(30);
    let renderer = VideoRenderer::new(width, height);

    // For each state in multiway graph, create an embedding and render
    for (state_idx, graph) in multiway.states.iter().enumerate().step_by(frame_skip) {
        if graph.nodes.is_empty() {
            continue;
        }

        // Embed graph
        let embedding = GraphEmbedding::from_hypergraph(graph, 100);

        // Render from multiple angles
        for angle_idx in 0..8 {
            let rotation = (angle_idx as f32 / 8.0) * 2.0 * PI;
            let tilt = 0.3;

            let frame = renderer.render_graph_3d(&embedding, rotation, tilt);
            sequence.add_frame(frame);
        }

        if state_idx % 10 == 0 {
            println!("Processed state {}/{}", state_idx, multiway.states.len());
        }
    }

    sequence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = VideoRenderer::new(640, 480);
        assert_eq!(renderer.width, 640);
        assert_eq!(renderer.height, 480);
    }

    #[test]
    fn test_sequence_creation() {
        let sequence = VideoSequence::new(30);
        assert_eq!(sequence.fps, 30);
        assert_eq!(sequence.frames.len(), 0);
    }
}
